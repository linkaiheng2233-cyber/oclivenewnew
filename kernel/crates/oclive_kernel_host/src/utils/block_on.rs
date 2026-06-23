use std::future::Future;

/// Run an async future on the current Tokio runtime from a synchronous caller.
///
/// # Panics
///
/// Panics if called from within an async task without a current runtime when no runtime exists.
pub fn block_on<F>(future: F) -> F::Output
where
    F: Future,
{
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        return tokio::task::block_in_place(|| handle.block_on(future));
    }
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap_or_else(|e| {
            tracing::error!(target: "oclive_runtime", error = %e, "block_on: failed to build runtime");
            panic!("tokio runtime for block_on: {e}");
        });
    rt.block_on(future)
}

/// Run a future on a dedicated thread with its own current-thread runtime.
///
/// Use from **synchronous** Tauri `invoke` handlers (WebView main thread). Calling
/// [`block_on`] there may nest `block_in_place` on the app runtime and panic with
/// "A Tokio 1.x context was found, but it is being shutdown".
///
/// # Panics
///
/// Panics if the isolated thread fails to start, the runtime cannot be built, or the thread join fails.
pub fn block_on_isolated<F, T>(future: F) -> T
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap_or_else(|e| {
                tracing::error!(target: "oclive_runtime", error = %e, "block_on_isolated: failed to build runtime");
                panic!("isolated tokio runtime: {e}");
            });
        rt.block_on(future)
    })
    .join()
    .expect("isolated block_on thread panicked")
}
