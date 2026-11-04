//! A seekable `GInputStream` over HTTP, so GTK's media player can stream
//! videos and GIFs straight from the site.
//!
//! GTK opens media through GIO, which only reads `https://` addresses
//! when a gvfs HTTP backend happens to be running. This stream fetches
//! with the app's own HTTP client instead, using `Range` requests: reads
//! continue the current response, and a new request is only made when
//! the player seeks somewhere else, and data already received (up to 8 MB
//! per video) is kept in memory so the demuxer's repeated reads of the
//! header and index don't each cost a request - which matters on CDNs
//! that rate-limit. Nothing is written to disk.
//!
//! GStreamer reads from its own threads, and stops a pipeline by
//! cancelling the read's `GCancellable` and then waiting for the read to
//! return - so every network wait here also watches that cancellable and
//! gives up at once when it fires (otherwise closing or switching a video
//! hangs the app until the network answers). The read position is atomic
//! so seeks and `tell()` never wait behind a read that's blocked on the
//! network.

use std::collections::BTreeMap;
use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use channex::data::http::HttpClient;
use gtk::gio;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::runtime;

/// Seeking forward by up to this much reads and discards instead of
/// opening a new request (fewer requests, gentler on rate limits).
const SKIP_LIMIT: u64 = 512 * 1024;
/// Retries for rate-limited (429) or flaky (5xx) answers. Kept low:
/// retrying a rate limit quickly only extends it.
const RETRIES: u32 = 2;
/// Longest wait honoured from a `Retry-After` header.
const MAX_RETRY_AFTER: Duration = Duration::from_secs(10);
/// What a rate-limited request reports, so the viewer can explain it.
pub const RATE_LIMITED: &str = "rate-limited (HTTP 429)";
/// Received data kept in memory per stream, so the demuxer's repeated
/// reads of the header and index don't each become a new request.
const MEMORY_CACHE: usize = 8 * 1024 * 1024;
/// A new request starts this far before the wanted position, so the
/// demuxer's small backward steps (e.g. while reading the index at the
/// end of a file) are already in memory.
const BACK_FETCH: u64 = 64 * 1024;
/// How often a network wait checks whether it's been cancelled.
const CANCEL_POLL: Duration = Duration::from_millis(25);

/// The open response and what's been received but not read.
#[derive(Default)]
struct Io {
    response: Option<reqwest::Response>,
    /// Stream position the open response will deliver next.
    response_pos: u64,
    pending: Vec<u8>,
    pending_offset: usize,
    /// Chunks already received, by file offset (up to `MEMORY_CACHE`).
    received: BTreeMap<u64, Vec<u8>>,
    received_bytes: usize,
}

impl Io {
    fn pending_len(&self) -> usize {
        self.pending.len() - self.pending_offset
    }

    fn reset(&mut self) {
        self.response = None;
        self.pending.clear();
        self.pending_offset = 0;
    }

    /// Copies from already-received data at `pos`, if there is any.
    fn read_received(&self, pos: u64, buffer: &mut [u8]) -> Option<usize> {
        let (start, chunk) = self.received.range(..=pos).next_back()?;
        let offset = usize::try_from(pos - start).ok()?;
        if offset >= chunk.len() {
            return None;
        }
        let n = buffer.len().min(chunk.len() - offset);
        buffer[..n].copy_from_slice(&chunk[offset..offset + n]);
        Some(n)
    }

    /// Keeps a chunk received at `pos` for later reads, while there's room.
    fn remember(&mut self, pos: u64, chunk: &[u8]) {
        if self.received_bytes + chunk.len() > MEMORY_CACHE
            || self.read_received(pos, &mut [0]).is_some()
        {
            return;
        }
        self.received_bytes += chunk.len();
        self.received.insert(pos, chunk.to_vec());
    }
}

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct HttpStream {
        pub url: OnceLock<String>,
        pub client: OnceLock<HttpClient>,
        /// Where the next read starts.
        pub(super) pos: AtomicU64,
        /// Total size once the server has said (`u64::MAX` = unknown).
        pub(super) size: AtomicU64,
        pub(super) io: Mutex<Io>,
        /// Why the stream is waiting, while it is (shown by the viewer).
        pub(super) status: Mutex<Option<String>>,
        /// Set when a request failed for good (retries used up). Every
        /// later request fails the same way without touching the
        /// network: GStreamer otherwise keeps retrying elsewhere in the
        /// file, which looks like loading forever and hits a
        /// rate-limiting site harder.
        pub(super) failure: Mutex<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for HttpStream {
        const NAME: &'static str = "ChannexHttpStream";
        type Type = super::HttpStream;
        type ParentType = gio::InputStream;
        type Interfaces = (gio::Seekable,);
    }

    impl ObjectImpl for HttpStream {
        fn constructed(&self) {
            self.parent_constructed();
            self.size.store(u64::MAX, Ordering::SeqCst);
        }
    }

    impl InputStreamImpl for HttpStream {
        fn read(
            &self,
            buffer: &mut [u8],
            cancellable: Option<&gio::Cancellable>,
        ) -> Result<usize, glib::Error> {
            debug_off_main_thread("read");
            let mut io = self.io.lock().unwrap();
            let pos = self.pos.load(Ordering::SeqCst);
            if buffer.is_empty() || self.known_size().is_some_and(|size| pos >= size) {
                return Ok(0);
            }
            // Data we've already received needs no network at all.
            if let Some(n) = io.read_received(pos, buffer) {
                self.pos.store(pos + n as u64, Ordering::SeqCst);
                return Ok(n);
            }
            self.position_response(&mut io, pos, cancellable)?;
            if io.pending_len() == 0 && !self.fill(&mut io, cancellable)? {
                return Ok(0); // end of stream
            }
            let n = buffer.len().min(io.pending_len());
            let start = io.pending_offset;
            buffer[..n].copy_from_slice(&io.pending[start..start + n]);
            io.pending_offset += n;
            io.response_pos += n as u64;
            self.pos.store(pos + n as u64, Ordering::SeqCst);
            Ok(n)
        }

        fn close(&self, _cancellable: Option<&gio::Cancellable>) -> Result<(), glib::Error> {
            // A read may be finishing; don't wait for it.
            if let Ok(mut io) = self.io.try_lock() {
                io.reset();
            }
            Ok(())
        }
    }

    impl SeekableImpl for HttpStream {
        fn tell(&self) -> i64 {
            self.pos.load(Ordering::SeqCst) as i64
        }

        fn can_seek(&self) -> bool {
            true
        }

        fn seek(
            &self,
            offset: i64,
            type_: glib::SeekType,
            cancellable: Option<&gio::Cancellable>,
        ) -> Result<(), glib::Error> {
            let base = match type_ {
                glib::SeekType::Cur => self.pos.load(Ordering::SeqCst) as i64,
                glib::SeekType::End => {
                    let size = match self.known_size() {
                        Some(size) => size,
                        None => {
                            // Learn the size from a request at the current spot.
                            debug_off_main_thread("seek to end");
                            let mut io = self.io.lock().unwrap();
                            let pos = self.pos.load(Ordering::SeqCst);
                            self.open(&mut io, pos, cancellable)?;
                            self.known_size().ok_or_else(|| {
                                glib::Error::new(
                                    gio::IOErrorEnum::NotSupported,
                                    "the server didn't say how big the file is",
                                )
                            })?
                        }
                    };
                    size as i64
                }
                _ => 0,
            };
            let target = base + offset;
            if target < 0 {
                return Err(glib::Error::new(
                    gio::IOErrorEnum::InvalidArgument,
                    "seek before the start of the file",
                ));
            }
            // The next read repositions the response if it needs to.
            self.pos.store(target as u64, Ordering::SeqCst);
            Ok(())
        }

        fn can_truncate(&self) -> bool {
            false
        }

        fn truncate(
            &self,
            _offset: i64,
            _cancellable: Option<&gio::Cancellable>,
        ) -> Result<(), glib::Error> {
            Err(glib::Error::new(
                gio::IOErrorEnum::NotSupported,
                "read-only stream",
            ))
        }
    }

    impl HttpStream {
        fn known_size(&self) -> Option<u64> {
            let size = self.size.load(Ordering::SeqCst);
            (size != u64::MAX).then_some(size)
        }

        /// Makes the open response deliver data from `pos`: keeps it if
        /// it's already there, reads ahead for a short forward skip, and
        /// otherwise opens a new range request.
        fn position_response(
            &self,
            io: &mut Io,
            pos: u64,
            cancellable: Option<&gio::Cancellable>,
        ) -> Result<(), glib::Error> {
            if io.response.is_some()
                && io.response_pos <= pos
                && pos - io.response_pos <= SKIP_LIMIT
            {
                self.skip_to(io, pos, cancellable)
            } else {
                self.open(io, pos, cancellable)
            }
        }

        /// Reads and discards until the response is at `pos`.
        fn skip_to(
            &self,
            io: &mut Io,
            pos: u64,
            cancellable: Option<&gio::Cancellable>,
        ) -> Result<(), glib::Error> {
            while io.response_pos < pos {
                if io.pending_len() == 0 && !self.fill(io, cancellable)? {
                    break;
                }
                let skip = (pos - io.response_pos).min(io.pending_len() as u64) as usize;
                io.pending_offset += skip;
                io.response_pos += skip as u64;
            }
            Ok(())
        }

        /// Opens `GET` with `Range: bytes=<start>-`, starting a little
        /// before `pos` (see `BACK_FETCH`) and reading up to it.
        fn open(
            &self,
            io: &mut Io,
            pos: u64,
            cancellable: Option<&gio::Cancellable>,
        ) -> Result<(), glib::Error> {
            io.reset();
            if let Some(failure) = self.failure.lock().unwrap().clone() {
                return Err(glib::Error::new(gio::IOErrorEnum::Failed, &failure));
            }
            let wanted = pos;
            let pos = pos.saturating_sub(BACK_FETCH);
            let client = self.client.get().unwrap().clone();
            let url = self.url.get().unwrap().clone();
            let status = &self.status;
            // Debug builds: act as if the site always answered 429.
            let fake_429 =
                cfg!(debug_assertions) && std::env::var_os("CHANNEX_DEV_FAKE_429").is_some();
            let response = wait(cancellable, async move {
                let mut attempt = 0;
                loop {
                    let result = client
                        .reqwest()
                        .get(&url)
                        .header(reqwest::header::RANGE, format!("bytes={pos}-"))
                        .send()
                        .await;
                    if cfg!(debug_assertions) {
                        match &result {
                            Ok(r) => {
                                eprintln!("HTTPSTREAM: GET bytes={pos}- -> {}", r.status().as_u16())
                            }
                            Err(e) => eprintln!("HTTPSTREAM: GET bytes={pos}- failed: {e}"),
                        }
                    }
                    // Wait as long as the server asks (Retry-After, in
                    // seconds), else back off: 1 s, then 3 s.
                    let mut delay = Duration::from_millis(if attempt == 0 { 1000 } else { 3000 });
                    let waiting_because = match result {
                        Ok(_) if fake_429 && attempt < RETRIES => {
                            "The site is limiting how fast files load".to_owned()
                        }
                        Ok(_) if fake_429 => return Err(RATE_LIMITED.to_owned()),
                        Ok(r) if r.status().is_success() => return Ok(r),
                        Ok(r)
                            if (r.status().as_u16() == 429 || r.status().is_server_error())
                                && attempt < RETRIES =>
                        {
                            let asked = r
                                .headers()
                                .get(reqwest::header::RETRY_AFTER)
                                .and_then(|v| v.to_str().ok())
                                .and_then(|v| v.trim().parse::<u64>().ok());
                            if let Some(secs) = asked {
                                delay = Duration::from_secs(secs).min(MAX_RETRY_AFTER);
                            }
                            if r.status().as_u16() == 429 {
                                "The site is limiting how fast files load".to_owned()
                            } else {
                                format!("The site answered {}", r.status().as_u16())
                            }
                        }
                        Ok(r) if r.status().as_u16() == 429 => return Err(RATE_LIMITED.to_owned()),
                        Ok(r) => {
                            return Err(format!("the server answered {}", r.status().as_u16()));
                        }
                        Err(_) if attempt < RETRIES => "Couldn't reach the site".to_owned(),
                        Err(e) => return Err(format!("couldn't connect: {e}")),
                    };
                    *status.lock().unwrap() = Some(format!(
                        "{waiting_because}. Trying again in {} s\u{2026}",
                        delay.as_secs().max(1)
                    ));
                    attempt += 1;
                    // (Cancelling the read also cuts this wait short.)
                    tokio::time::sleep(delay).await;
                }
            });
            *self.status.lock().unwrap() = None;
            let response = response?.map_err(|e| {
                *self.failure.lock().unwrap() = Some(e.clone());
                glib::Error::new(gio::IOErrorEnum::Failed, &e)
            })?;

            if response.status() == reqwest::StatusCode::PARTIAL_CONTENT {
                // "Content-Range: bytes 100-999/1000"
                let total = response
                    .headers()
                    .get(reqwest::header::CONTENT_RANGE)
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.rsplit('/').next())
                    .and_then(|total| total.parse::<u64>().ok());
                if let Some(total) = total {
                    self.size.store(total, Ordering::SeqCst);
                }
                io.response_pos = pos;
            } else {
                // The server ignored the range and sends the whole file.
                if let Some(total) = response.content_length() {
                    self.size.store(total, Ordering::SeqCst);
                }
                io.response_pos = 0;
            }
            io.response = Some(response);
            // Read up to where we wanted to be (the back-fetched part, or
            // everything before it if the server ignored the range).
            self.skip_to(io, wanted, cancellable)
        }

        /// Fetches the next chunk into `pending`. `false` at end of data.
        fn fill(
            &self,
            io: &mut Io,
            cancellable: Option<&gio::Cancellable>,
        ) -> Result<bool, glib::Error> {
            let Some(mut response) = io.response.take() else {
                return Ok(false);
            };
            let chunk = wait(cancellable, async move {
                response.chunk().await.map(|c| (c, response))
            })?;
            match chunk {
                Ok((Some(bytes), response)) => {
                    let at = io.response_pos;
                    io.remember(at, &bytes);
                    io.pending = bytes.to_vec();
                    io.pending_offset = 0;
                    io.response = Some(response);
                    Ok(true)
                }
                Ok((None, _)) => Ok(false),
                Err(_) => Err(glib::Error::new(
                    gio::IOErrorEnum::BrokenPipe,
                    "the download was interrupted",
                )),
            }
        }
    }
}

glib::wrapper! {
    pub struct HttpStream(ObjectSubclass<imp::HttpStream>)
        @extends gio::InputStream,
        @implements gio::Seekable;
}

impl HttpStream {
    /// `size` is the file's length in bytes when the site's API gave it
    /// (0 = unknown). Knowing it up front matters: GStreamer asks for the
    /// size while starting a pipeline, before it can cancel anything, so
    /// answering from the network there would block - and closing the
    /// video waits on that start.
    pub fn new(client: HttpClient, url: &str, size: u64) -> Self {
        let stream: Self = glib::Object::new();
        let _ = stream.imp().url.set(url.to_owned());
        let _ = stream.imp().client.set(client);
        if size > 0 {
            stream.imp().size.store(size, Ordering::SeqCst);
        }
        stream
    }

    /// What the stream is waiting on, if it's held up (for example,
    /// "The site is limiting how fast files load. Trying again in 3 s…").
    pub fn status(&self) -> Option<String> {
        self.imp().status.lock().unwrap().clone()
    }

    /// Why the stream gave up, once it has (it won't try again).
    pub fn failure(&self) -> Option<String> {
        self.imp().failure.lock().unwrap().clone()
    }
}

/// Runs `future` on the background runtime, giving up with
/// `G_IO_ERROR_CANCELLED` as soon as `cancellable` is cancelled.
fn wait<F: Future>(
    cancellable: Option<&gio::Cancellable>,
    future: F,
) -> Result<F::Output, glib::Error> {
    let cancellable = cancellable.cloned();
    runtime::block_on(async move {
        let cancelled = async {
            loop {
                if cancellable
                    .as_ref()
                    .is_some_and(gio::Cancellable::is_cancelled)
                {
                    return;
                }
                tokio::time::sleep(CANCEL_POLL).await;
            }
        };
        tokio::select! {
            output = future => Ok(output),
            () = cancelled => Err(glib::Error::new(gio::IOErrorEnum::Cancelled, "cancelled")),
        }
    })
}

/// Debug builds: flag a network wait on the UI thread (it would freeze).
fn debug_off_main_thread(what: &str) {
    if cfg!(debug_assertions) && glib::MainContext::default().is_owner() {
        eprintln!("HTTPSTREAM: {what} called on the main thread");
    }
}
