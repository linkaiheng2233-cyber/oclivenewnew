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

## Wave A–D · TTFT / co-present（2026-06）

| Wave | 状态 | 摘要 |
|------|------|------|
| **A** | Done | `host_flags.event_impact_llm` · 规则 event · `event_impact_ai` 分支 |
| **B** | Done | Turn Thinking Auto/Fast/Deep · `co_present` Fast 裁剪 · `measure-ttft.mjs` |
| **C** | Done | Chat Pro UI `/chat/stream` · `sendMessageStream` · 失败回退 `/chat` |
| **D** | T1/T2/T3 Done | Deep capsule · `PersonaSource` · **`prompt_prefix_cache`** · `--deep-multi` prefill bench |
| **D-T3** | Done | `build_prompt_segments` · prefill p50 **28ms** vs 冷启动 **1443ms**（mumu · qwen2.5:7b · 2026-06-26） |
| **E** | Done | `fast_persistence = strong_only` · Fast 闲聊不写 long_term / favor / evolution；强事件仍写 · RFC [`RFC_TURN_THINKING_PERSISTENCE.md`](../creator-docs/rfc/RFC_TURN_THINKING_PERSISTENCE.md) |
| **F** | Done | 包级 `turn_thinking` OR/AND · 本句 rule prepass · Deep latch · ephemeral TTL 局面摘要 · migration `035` |

Bench SSOT：[`TTFT_BENCHMARK.md`](TTFT_BENCHMARK.md) · 设计：[`DEEP_PROMPT_DISTILLATION.md`](DEEP_PROMPT_DISTILLATION.md) · 愿景：[`creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md`](../creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md)。

## Chat Pro 正式启用（2026-06-26）

| 项 | 状态 |
|----|------|
| Profile | `desktop.oclive.toml` · `[turn_thinking]` Auto/Fast/Deep |
| 流式 UI | `sendMessageStream` + 失败回退 `/chat` + `AbortController` 取消 |
| 设置 | `chat.streamEnabled`（localStorage，默认开启） |
| Bench | `measure-ttft.mjs --profile desktop` · OOCP **S15** 文档化于 [`TTFT_BENCHMARK.md`](TTFT_BENCHMARK.md) |
| 技术债 | **K-CHATPRO-01** 流式取消 UX → **Done** |
