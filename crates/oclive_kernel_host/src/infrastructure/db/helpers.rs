#[macro_export]
macro_rules! txn_step {
    ($role_id:expr, $started:expr, $code:literal, $step_name:literal, $future:expr) => {
        {
            let _step_started = std::time::Instant::now();
            if let Err(e) = $future.await {
                let msg = e.to_string();
                tracing::error!(
                    "tx step failed code={} step={} role_id={} err={} elapsed_ms={}",
                    $code,
                    $step_name,
                    $role_id,
                    msg,
                    $started.elapsed().as_millis()
                );
                return Err($crate::error::AppError::TransactionError {
                    code: $code,
                    message: msg,
                });
            }
            tracing::debug!(
                "tx step ok step={} role_id={} step_elapsed_ms={} tx_elapsed_ms={}",
                $step_name,
                $role_id,
                _step_started.elapsed().as_millis(),
                $started.elapsed().as_millis()
            );
        }
    };
}
