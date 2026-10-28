use std::future::Future;

/// Run a future from a synchronous caller, borrowing the multi-thread Tokio runtime if present.
///
/// A current-thread runtime cannot be synchronously driven from inside its own task.
/// Fallible production adapters should use the checked bridge instead of this legacy helper.
///
/// # Panics
///
/// Panics on a current-thread/unsupported runtime, or when a standalone runtime cannot be built.
pub fn block_on<F>(future: F) -> F::Output
where
    F: Future,
{
    try_block_on(future).unwrap_or_else(|error| {
        tracing::error!(target: "oclive_runtime", %error, "block_on failed");
        panic!("synchronous async bridge: {error}");
    })
}

/// Reject an unsupported runtime before polling the future; no Send or 'static bounds are added.
pub(crate) fn try_block_on<F: Future>(future: F) -> Result<F::Output, String> {
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        return match handle.runtime_flavor() {
            tokio::runtime::RuntimeFlavor::MultiThread => {
                Ok(tokio::task::block_in_place(|| handle.block_on(future)))
            }
            _ => Err("synchronous RPC requires a multi-thread Tokio runtime or no active runtime; use the async API on a current-thread runtime".into()),
        };
    }
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| format!("cannot build synchronous async bridge runtime: {error}"))?;
    Ok(runtime.block_on(future))
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn checked_bridge_without_runtime_preserves_borrowed_non_send_future_and_output() {
        let value = Rc::new(Cell::new(4));
        let result = try_block_on(async {
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
            value.set(5);
            &value
        })
        .unwrap();
        assert!(Rc::ptr_eq(result, &value));
        assert_eq!(value.get(), 5);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn checked_bridge_on_multi_thread_runtime_drives_timer_without_moving_future() {
        let value = Rc::new(Cell::new(1));
        let result = try_block_on(async {
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
            value.set(2);
            &value
        })
        .unwrap();
        assert_eq!(result.get(), 2);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn checked_bridge_rejects_current_thread_before_polling() {
        let polled = Cell::new(false);
        let error = try_block_on(async {
            polled.set(true);
        })
        .unwrap_err();
        assert!(error.contains("async API"));
        assert!(!polled.get());
    }
}
