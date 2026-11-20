//! Running work off the UI thread.
//!
//! - [`blocking`]: run a short blocking closure (DB query, file stat) on a worker thread.
//! - [`runner`]: run a [`dk_core::Tool`] with progress, cancellation and panic isolation.

pub mod runner;

/// Run `f` on a fresh worker thread and await its result.
/// Returns `None` if the thread could not start or panicked.
pub async fn blocking<T, F>(f: F) -> Option<T>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    let (tx, rx) = futures_channel::oneshot::channel();
    let spawned = std::thread::Builder::new()
        .name("dk-blocking".into())
        .spawn(move || {
            // Receiver gone means the component unmounted; dropping the result is fine.
            let _ = tx.send(f());
        });
    if let Err(e) = spawned {
        tracing::error!("cannot spawn worker thread: {e}");
        return None;
    }
    rx.await.ok()
}
