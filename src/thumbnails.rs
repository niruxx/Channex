//! Loads remote images (thumbnails, favicons) as textures, with an
//! in-memory cache of recent textures and an on-disk cache of the raw
//! files in `<user cache dir>/channex/thumbnails/` (cleared on the
//! schedule and size limit set in Preferences; see `cache.rs`). At most 8
//! downloads run at once so a 150-thread catalog doesn't open 150
//! connections.

use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use std::time::SystemTime;

use gtk::{gdk, glib};
use tokio::sync::Semaphore;

use crate::runtime;
use crate::state::state;

/// Decoded textures kept in memory. A 4chan thumbnail is up to
/// 250x250 (~250 KB decoded), so this is roughly 60 MB at most.
const MEMORY_ENTRIES: usize = 250;
const MAX_PARALLEL: usize = 8;

#[derive(Default)]
struct MemoryCache {
    textures: HashMap<String, gdk::Texture>,
    order: VecDeque<String>,
}

impl MemoryCache {
    fn get(&self, url: &str) -> Option<gdk::Texture> {
        self.textures.get(url).cloned()
    }

    fn insert(&mut self, url: String, texture: gdk::Texture) {
        if self.textures.insert(url.clone(), texture).is_none() {
            self.order.push_back(url);
        }
        while self.order.len() > MEMORY_ENTRIES {
            if let Some(old) = self.order.pop_front() {
                self.textures.remove(&old);
            }
        }
    }
}

thread_local! {
    static MEMORY: RefCell<MemoryCache> = RefCell::default();
}

fn semaphore() -> Arc<Semaphore> {
    static SEM: OnceLock<Arc<Semaphore>> = OnceLock::new();
    SEM.get_or_init(|| Arc::new(Semaphore::new(MAX_PARALLEL)))
        .clone()
}

pub fn cache_dir() -> PathBuf {
    glib::user_cache_dir().join("channex").join("thumbnails")
}

/// FNV-1a: a stable file name per URL (std's hasher isn't guaranteed
/// stable across Rust versions).
fn cache_name(url: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in url.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    format!("{hash:016x}")
}

/// A texture for `url`, or `None` if it couldn't be fetched or decoded.
pub async fn load(url: &str) -> Option<gdk::Texture> {
    if url.is_empty() {
        return None;
    }
    if let Some(hit) = MEMORY.with(|m| m.borrow().get(url)) {
        return Some(hit);
    }

    let client = state().http();
    let owned = url.to_owned();
    let bytes = runtime::spawn(async move {
        let path = cache_dir().join(cache_name(&owned));
        if let Ok(bytes) = tokio::fs::read(&path).await {
            // Mark it as recently used: the size limit evicts the least
            // recently used images first (see cache.rs).
            if let Ok(file) = std::fs::File::options().append(true).open(&path) {
                let _ = file.set_modified(SystemTime::now());
            }
            return Some(bytes);
        }
        let _permit = semaphore().acquire_owned().await.ok()?;
        let bytes = client.get_bytes(&owned).await.ok()?;
        let _ = tokio::fs::create_dir_all(cache_dir()).await;
        let _ = tokio::fs::write(&path, &bytes).await;
        Some(bytes)
    })
    .await?;

    let texture = gdk::Texture::from_bytes(&glib::Bytes::from_owned(bytes)).ok()?;
    MEMORY.with(|m| m.borrow_mut().insert(url.to_owned(), texture.clone()));
    Some(texture)
}

/// Forgets every texture held in memory (after the disk cache is
/// cleared, so nothing stale is shown).
pub fn clear_memory() {
    MEMORY.with(|m| *m.borrow_mut() = MemoryCache::default());
}
