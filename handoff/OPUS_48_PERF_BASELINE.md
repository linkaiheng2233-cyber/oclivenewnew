# Opus 4.8 热路径性能基线（Phase 0）

固定采样与 DB 往返粗估，供 PR-A/B 前后对比。无代码行为变更。

**采样日期**：2026-06-08 · **环境**：Windows x86_64 · Release · `OCLIVE_HTTP_API_MOCK_LLM=1` · 单轮 `POST /chat`（`mumu`）· `RUST_LOG=oclive_turn=debug`

## oclive_turn Top-10（ms，降序）

| Stage | elapsed_ms（约） |
|-------|------------------|
| `build_prompt` | 12.4 |
| `bot_reply_emotion_analyze` | 8.1 |
| `load_memories` | 6.3 |
| `memory_rank` | 4.9 |
| `load_recent_context` | 3.2 |
| `apply_chat_turn_atomic` | 2.8 |
| `ensure_role_loaded` | 2.1 |
| `complex_emotion_resolve_turn` | 1.6 |
| `startup_health` | 1.2 |
| `ensure_role_runtime` | 0.9 |

完整表见 [`creator-docs/getting-started/PERFORMANCE.md`](../creator-docs/getting-started/PERFORMANCE.md) §6。

## DB 往返粗估（优化前 · `ensure_role_runtime` → `pre_llm`）

| 路径 | 调用点 | 每回合约计 |
|------|--------|------------|
| `effective_slot_registry_for_session` | `session_backends` ×3 间接 | **3** |
| `get_current_scene` / `get_interaction_mode` / `get_remote_life_enabled` | `process_message.rs` | **3** |
| `get_event_impact_factor` / `get_mutable_personality` | `pre.rs` prefetch | **2** |
| `load_recent_context` | `agent_context` + `pre.rs` | **2** |
| `resolve_active_user_identity` | `agent_context` + `pre.rs` | **2** |
| `persist_memory_decay_batch` | `pre.rs` ×2 | **2 事务** |

**目标（PR-A 后）**：session 配置 **1** 次；`role_runtime` 热字段快照 **1** 次读；`load_recent_context` **≤1**；decay **1** 事务。

## oclive_turn Top-10（ms，降序 · PR-A/B 后）

**采样日期**：2026-06-08 · **环境**：与 §Before 相同（Release · `OCLIVE_HTTP_API_MOCK_LLM=1` · 单轮 `POST /chat` · `RUST_LOG=oclive_turn=debug`）· **steady-state**（同进程第二轮，角色/DB 已热）

| Stage | elapsed_ms（约） | Before（约） | 备注 |
|-------|------------------|--------------|------|
| `apply_chat_turn_atomic` | 0.5 | 2.8 | K-PERF-01 批处理保留 |
| `virtual_time_ms` | 0.5 | — | 新进 Top-10 |
| `get_role_runtime_snapshot` | 0.2 | — | K-PERF-04 合并 `event_impact_factor` / `mutable_personality` |
| `relation_state_for_identity` | 0.2 | — | |
| `set_user_presence_scene` | 0.1 | — | |
| `current_personality` | 0.1 | — | |
| `idle_personality_decay` | 0.1 | — | |
| `load_memories` | 0.08 | 6.3 | |
| `set_core_delta_personality_json_non_profile` | 0.07 | — | |
| `ensure_identity_stats_row` | 0.07 | — | |

**退出 Top-10（仍采样，未单独列出）**：`build_prompt` **0.05**（Before 12.4）、`bot_reply_emotion_analyze` **0.008**（Before 8.1）、`memory_rank` **0.007**（Before 4.9）。

**不再单独计 stage**：`load_recent_context`（K-PERF-03 TurnPrefetch 一次构建）、`ensure_role_runtime` / `startup_health` / `ensure_role_loaded`（仍执行但未走 `process_message_stage` 计时）。

冷启动首回合 `apply_chat_turn_atomic` 约 **1.0** ms（空 `OCLIVE_APP_DATA`）；interpretation 见 [`PERFORMANCE.md`](../creator-docs/getting-started/PERFORMANCE.md) §6。

## PR 切分（锁定）

| PR | 分支建议 | ID | 依赖 |
|----|----------|-----|------|
| PR-C1 | `discipline/fq-ratchet` | D-LAYER-05a | — |
| PR-A | `perf/turn-prefetch-snapshot` | K-PERF-03~06 | Phase 0 |
| PR-B | `perf/session-cache-sqlite` | K-PERF-07/08/12 | 可与 PR-A 并行 |
| PR-D | `perf/frontend-lazy-i18n` | K-PERF-10/11, K-DOC-06 | — |
| PR-C2 | `discipline/turn-ports` | D-LAYER-05b | PR-C1 |
| PR-E | `hygiene/dto-errors` | D-DTO-01, D-ERR-01 | — |

**推荐合并顺序**：PR-C1 → PR-A → PR-B → PR-D → PR-C2 → PR-E。
