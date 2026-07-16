# WAVE-20260716-K-SUPPLY-05-Full-s1

> 计划书：[`../long-plans/K-SUPPLY-05-Full.md`](../long-plans/K-SUPPLY-05-Full.md) · 前序：[s0](./WAVE-20260716-K-SUPPLY-05-Full-s0.md)

## 摘要

| 字段 | 值 |
|------|-----|
| **债 ID** | K-SUPPLY-05-Full |
| **Stage** | 1 · Converge one dependency family（toml） |
| **日期** | 2026-07-16 |
| **Claim** | `96c09b16-f3f1-43a9-bfd8-7b4f8feab2af` |
| **Base** | `6a95fae7f5d7b2e8d2a1a70cb1b0b38f578909eb` |
| **执行面** | [oclive-debt-stage](6df9c992-1218-4df6-996e-f7e54c8c19bf) |

## 证据

| 检查 | 结果 |
|------|------|
| `cargo deny check bans` | **PASS** |
| `cargo audit` | **PASS**（8 allowed 预存；无新增） |
| `check-cargo-dedup-ratchet` | **PASS** · **75**（≤80） |

## 做了什么

- Workspace `toml` **0.8→1**；宿主可见 toml majors 3→2；ratchet 80→75
- `deny.toml` **未**删 skip（0.8 via system-deps Linux · 0.9 via cargo_toml←tauri-build 仍在）

## 刻意没做

- 未动 windows/Tauri · 未假零 skip · 未合 main · 未改 KNOWN（无新 advisory）

## 下一跳

Stage 2：尝试删 skip → 预期诚实不可 → `blocked:needs-ecosystem` 或 Partial

## GATES §6

- [x] 仅 Cargo.toml/Cargo.lock · 验收 PASS · 未升 Full Done · 未合 main
