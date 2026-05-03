//! Sync call sites that still need HTTP without enabling workspace `reqwest/blocking`.
//!
//! Uses a small dedicated Tokio runtime so `reqwest::Client` can be driven from sync code
//! (Tauri sync `invoke`, `MemoryRetrieval::rank_memories`, market index sync, etc.).

use once_cell::sync::Lazy;
use std::future::Future;
use tokio::runtime::Runtime;

static HTTP_RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .thread_name("oclive-http")
        .build()
        .expect("oclive http runtime")
});

#[inline]
pub fn block_on<F>(future: F) -> F::Output
where
    F: Future,
{
    HTTP_RUNTIME.block_on(future)
}
