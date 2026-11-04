//! A background tokio runtime for network work. UI code awaits the
//! results from the GTK main loop:
//!
//! ```ignore
//! glib::spawn_future_local(async move {
//!     let posts = runtime::spawn(async move { fetch_catalog(&client, &site, &board).await }).await;
//!     // back on the main thread: update widgets
//! });
//! ```

use std::future::Future;
use std::sync::OnceLock;

use tokio::runtime::Runtime;

fn runtime() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .thread_name("channex-net")
            .enable_all()
            .build()
            .expect("couldn't start the background runtime")
    })
}

/// Runs `future` on the background runtime; await the returned future
/// from the main loop to get its output.
pub fn spawn<F, T>(future: F) -> impl Future<Output = T>
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    let handle = runtime().spawn(future);
    async move { handle.await.expect("background task panicked") }
}

/// Runs `future` to completion on the background runtime, blocking the
/// calling thread. For code called synchronously from other threads
/// (GStreamer reading a media stream); never call it from a runtime task.
pub fn block_on<F: Future>(future: F) -> F::Output {
    runtime().block_on(future)
}
