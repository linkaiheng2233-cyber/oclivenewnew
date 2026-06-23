//! AB4：Loom 模型检查（`cargo loom test --test loom_concurrency`）。

#![allow(clippy::unwrap_used)]

#[cfg(loom)]
use loom::sync::atomic::{AtomicUsize, Ordering};
#[cfg(loom)]
use loom::thread;

#[cfg(not(loom))]
#[test]
#[ignore = "run with: cargo loom test --test loom_concurrency (requires cfg(loom))"]
fn loom_tests_require_cfg_loom() {}

/// 模型化 `jsonrpc` 请求 ID 分配（`fetch_add` SeqCst）无数据竞争。
#[cfg(loom)]
#[test]
fn jsonrpc_id_allocator_model() {
    loom::model(|| {
        let next = AtomicUsize::new(1);
        let handles: Vec<_> = (0..3)
            .map(|_| {
                thread::spawn(|| {
                    for _ in 0..8 {
                        let _id = next.fetch_add(1, Ordering::SeqCst);
                    }
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
        assert_eq!(next.load(Ordering::SeqCst), 1 + 3 * 8);
    });
}

/// 模型化会话 `narrative_hint` 缓存：单写者 / 多读者 RwLock 语义。
#[cfg(loom)]
#[test]
fn narrative_hint_cache_model() {
    loom::model(|| {
        let map = loom::sync::RwLock::new(String::new());
        let w = thread::spawn(|| {
            if let Ok(mut g) = map.write() {
                g.push_str("hint");
            }
        });
        let r = thread::spawn(|| {
            if let Ok(g) = map.read() {
                let _ = g.len();
            }
        });
        w.join().unwrap();
        r.join().unwrap();
    });
}
