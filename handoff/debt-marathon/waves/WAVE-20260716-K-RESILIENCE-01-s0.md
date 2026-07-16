# WAVE-20260716-K-RESILIENCE-01-s0

> 计划书：[`../long-plans/K-RESILIENCE-01.md`](../long-plans/K-RESILIENCE-01.md)

## 摘要

| 字段 | 值 |
|------|-----|
| **债 ID** | K-RESILIENCE-01 |
| **Stage** | 0 · Inventory remote resilience |
| **日期** | 2026-07-16 |
| **Claim** | `c8ddaf46-7f06-4a4c-9ec5-0a8ec801ad8a` |
| **HEAD** | `2c3ce7d7d8c5e5041342528f327ea0315dfc8c56` |
| **执行面** | [oclive-debt-stage](5158e927-081e-47c3-829b-2a80b80bc3b2) |

## 证据

| 检查 | 结果 |
|------|------|
| `rg -n "timeout\|retry\|fallback" …/remote_plugin` | **PASS** |

## 对齐结论（压缩）

- **Canonical fallback**：`remote_plugin/adapter.rs` `call_with_builtin_fallback` (+ async) + `remote_fallback_policy.rs`
- **Timeout SSOT**：`config.rs` → `jsonrpc.rs` `.timeout`
- **Retry**：host 侧 **无**（Minimal 不发明重试层）
- **离群点**：`prompt_http` / `memory_http` 内联 `remote_fallback_load`（Stage 2 候选接线）
- Stage 1+2 **可行** · 无需新 RFC · parentDisposition keep-open

## 下一跳

`claim --debt K-RESILIENCE-01 --stage 1 …`

## GATES §6

- [x] read-only · 未 Done · 未合 main · 父更新 QUEUE/checkpoint
