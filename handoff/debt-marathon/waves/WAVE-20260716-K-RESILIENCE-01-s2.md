# WAVE-20260716-K-RESILIENCE-01-s2

> 计划书：[`../long-plans/K-RESILIENCE-01.md`](../long-plans/K-RESILIENCE-01.md) · 前序：[s1](./WAVE-20260716-K-RESILIENCE-01-s1.md)

## 摘要

| 字段 | 值 |
|------|-----|
| **债 ID** | K-RESILIENCE-01 |
| **Stage** | 2 · One representative wiring |
| **分支** | `debt/fix-marathon-stop-hook` |
| **日期** | 2026-07-16 |
| **执行面** | [oclive-debt-stage](48619809-2353-4e65-bdbb-c548f85f3be6) |
| **状态三态** | Locally verified |
| **Claim** | `d1efb9e0-8b59-4364-b317-04d9d2c72997` · attempt 1 |

## 证据

| 项 | 值 |
|----|-----|
| Base / HEAD | `6d55a007415784225b0d64f448562873a68f378d` |
| Changed | `prompt_http.rs` · `adapter.rs`（测）· `reply_post_process_http.rs`（mock id）· `config.rs`（ENV_LOCK） |
| `cargo test -p oclive_kernel_host remote_plugin` | **PASS**（25） |
| `check-domain-layering` | **PASS** |

## 做了什么

- `prompt_http::build_prompt` → `RemotePluginAdapterBlocking::call_with_builtin_fallback`
- 新增 adapter 单测：fallback on / fallback disabled
- 测试夹具连带：reply mock 回显 JSON-RPC id；config ENV_LOCK 串行化

## 刻意没做

- 未改 `memory_http` · 未发明 retry 层 · 未合 main · 未升 TECHNICAL_DEBT Done

## 下一跳

父 Agent Stage 3：Partial 台账 + Wave + commit/push/open-pr（不合 main）

## GATES §6

- [x] 文件范围仅 remote_plugin/（+ policy 未改）
- [x] GATES 已读
- [x] 验收 PASS
- [x] 未升错误 Done / 未合 main
- [x] 父更新 QUEUE/checkpoint
