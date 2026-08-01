//! AB4：Loom 模型检查（`cargo test --features loom-tests --test loom_concurrency`）。

#![allow(clippy::unwrap_used)]

#[cfg(feature = "loom-tests")]
use loom::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
#[cfg(feature = "loom-tests")]
use loom::thread;
#[cfg(feature = "loom-tests")]
use std::time::Duration;

#[cfg(not(feature = "loom-tests"))]
#[test]
fn loom_tests_require_feature() {
    // Full model tests locally: cargo test --release --features loom-tests --test loom_concurrency
}

/// 模型化 `jsonrpc` 请求 ID 分配（`fetch_add` SeqCst）无数据竞争。
#[cfg(feature = "loom-tests")]
#[test]
fn jsonrpc_id_allocator_model() {
    let mut builder = loom::model::Builder::new();
    builder.preemption_bound = Some(2);
    builder.max_permutations = Some(20_000);
    builder.max_duration = Some(Duration::from_secs(30));
    builder.check(|| {
        let next = Arc::new(AtomicUsize::new(1));
        let handles: Vec<_> = (0..3)
            .map(|_| {
                let next = Arc::clone(&next);
                thread::spawn(move || {
                    for _ in 0..2 {
                        let _id = next.fetch_add(1, Ordering::SeqCst);
                    }
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
        assert_eq!(next.load(Ordering::SeqCst), 1 + 3 * 2);
    });
}

/// 模型化会话 `narrative_hint` 缓存：单写者 / 多读者 RwLock 语义。
#[cfg(feature = "loom-tests")]
#[test]
fn narrative_hint_cache_model() {
    loom::model(|| {
        let map = Arc::new(loom::sync::RwLock::new(String::new()));
        let write_map = Arc::clone(&map);
        let w = thread::spawn(move || {
            if let Ok(mut g) = write_map.write() {
                g.push_str("hint");
            }
        });
        let read_map = Arc::clone(&map);
        let r = thread::spawn(move || {
            if let Ok(g) = read_map.read() {
                let _ = g.len();
            }
        });
        w.join().unwrap();
        r.join().unwrap();
    });
}
