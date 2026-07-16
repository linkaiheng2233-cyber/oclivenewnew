# WAVE-20260716-K-SUPPLY-05-Full-s2

> 计划书：[`../long-plans/K-SUPPLY-05-Full.md`](../long-plans/K-SUPPLY-05-Full.md) · 前序：[s1](./WAVE-20260716-K-SUPPLY-05-Full-s1.md)

## 摘要

| 字段 | 值 |
|------|-----|
| **债 ID** | K-SUPPLY-05-Full |
| **Stage** | 2 · Remove final skips |
| **日期** | 2026-07-16 |
| **Claim** | `6a24630f-e159-4445-92a3-f7e093ec44d3` |
| **执行面** | 父 controller |
| **结果** | **诚实停** · 未删 skip · **不准假 Full Done** |

## 证据

| 检查 | 结果 |
|------|------|
| `cargo deny check bans` | **PASS**（仍依赖 documented skips） |
| `check-cargo-dedup-ratchet` | **PASS** · 75 ≤ 80 |
| `deny.toml` diff | **none**（rollback：保留 skip） |

## 仍须 skip（摘要）

- **toml 族**：Linux `system-deps`→toml 0.8；`cargo_toml`←tauri-build→0.9；workspace/tauri-utils→1.x
- **windows\*** / WebView2 多代 · hash/getrandom/bitflags 深栈 · reqwest/base64 工具链分叉

## 做了什么

- 按计划 rollback：不删 `[bans.skip]`；Full 零 skip **不可**在本生态安全达成

## 下一跳

Stage 3：台账 Partial（Full）+ 提交 Stage1 toml 收敛 + PR（#126）

## GATES §6

- [x] 未假 Full Done · 未合 main · Wave 写清剩余 skip
