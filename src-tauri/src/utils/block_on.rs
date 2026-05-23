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
        .expect("tokio runtime for block_on");
    rt.block_on(future)
}
