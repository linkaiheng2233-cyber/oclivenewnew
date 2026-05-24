# 性能阶段总表（活跃快照）

> 历史阶段报告见 [`archive/13_PERF_BASELINE_2026-04-01.md`](archive/13_PERF_BASELINE_2026-04-01.md)、[`archive/12_BACKEND_PERF_RUNBOOK.md`](archive/12_BACKEND_PERF_RUNBOOK.md)。详细轻量剖面见 [`creator-docs/development/LIGHTWEIGHT_PROFILE.md`](../creator-docs/development/LIGHTWEIGHT_PROFILE.md)。

## Round 14 收尾验证（2026-05-25）

环境：Windows，`cargo build --release -p oclivenewnew-tauri`（`target-dir` → `../oclive-dev-artifacts/oclivenewnew-cargo-target/`）。

| 项 | 结果 |
|----|------|
| Release 可执行文件 | `release/oclivenewnew-tauri.exe` — **14,094,848 B（≈13.44 MiB）**，构建时间 2026-05-25 |
| OOCP S0–S12 | 全部 ✓（`OCLIVE_HTTP_API_MOCK_LLM=1`，`http://127.0.0.1:8420`，`node examples/oocp-test-suite/run.mjs`） |
| `cargo test -p oclivenewnew-tauri --lib`（默认） | 127 passed |
| `cargo test -p oclivenewnew-tauri --lib --features dual_core` | 132 passed |
| `cargo clippy --workspace --all-targets --all-features` | 通过（`-D warnings`） |

说明：默认 feature 下双核 `dual_pipeline*` 不编入产物；包体为第十四轮精简（re-export 合并、`dual_core` gate、移除 Tauri `fs-*`、Cache 去 `lru` 等）后的 release 实测。
