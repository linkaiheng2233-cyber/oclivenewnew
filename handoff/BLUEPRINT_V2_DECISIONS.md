# 蓝图 v2 维护者决议记录

| 项 | 值 |
|----|-----|
| 状态 | **已确认**（2026-05-20；开工口令：**可以落实**） |
| RFC | [RFC_ROLE_BLUEPRINT_V2.md](RFC_ROLE_BLUEPRINT_V2.md) |
| 实施计划 | [BLUEPRINT_V2_IMPLEMENTATION_PLAN.md](BLUEPRINT_V2_IMPLEMENTATION_PLAN.md) |

与「建议」不同的选项已加粗。

---

## G1–G5（总确认）

| ID | 决议 |
|----|------|
| G1 | P1 **仅** Schema + validation，不改 `co_present` / 加载链 |
| G2 | P2∥P6 → P3 → P4 → P5 → P7 → P8 |
| G3 | P5 下线手拖连线，工具栏写蓝图 |
| G4 | 先 **P1 PR**，P2+ 二次确认 |
| G5 | 开工口令：**可以落实** |

---

## R1–R4

| ID | 决议 |
|----|------|
| R1 | 会话覆盖按 **`slot_registry` 实例键** |
| R2 | 架构图改槽位 **立即写盘** `pipeline.ocblueprint` |
| R3 | 保留 **内核 + 设施总线** 示意节点 |
| R4 | `meta.id` ≠ 目录名 → **ERROR** |

---

## B1–B5（P1 Schema）

| ID | 决议 |
|----|------|
| B1 | 固定文件名 **`pipeline.ocblueprint`** |
| B2 | `schema_version` **仅整数 `2`** |
| B3 | **`module_relations` 禁止出现在文件中**（仅运行时派生） |
| B4 | `slot_registry` 键：**任意非空字符串** |
| B5 | 同 `type` 下重复 **`position` → ERROR** |

---

## M1–M7

| ID | 决议 |
|----|------|
| M1 | 保留可选 **`author.json`** |
| M2 | **`ui.json`** 继续独立 |
| M3 | 保留 **`knowledge/`** 目录 |
| M4 | **`reply_quality_anchor`**：`prompts/` 与 `meta` 均可；**meta 优先** |
| M5 | 迁移时 `suggested_plugin_backends` → 默认 **`slot_registry`** |
| M6 | **`min_runtime_version` 在 `meta`** |
| M7 | P1 校验支持 **7 元 `personality` 数组** |

---

## S1–S8

| ID | 决议 |
|----|------|
| S1 | 不强制默认槽（0 实例则跳过阶段） |
| S2 | `complex_emotion` 允许 0 实例 |
| S3 | agent 非 directory 时不得有 `plugins[]` |
| S4 | 非 agent 不得同时有 `plugin` 与 `plugins` |
| S5 | `remote` 的 `url` 可空 |
| S6 | `memory`+`local` 的 `local_memory_provider_id` 可空 |
| S7 | `position` **仅同 type 内**比较 |
| S8 | 迁移工具自动加默认 **`complex_emotion`** 槽 |

---

## C1–C6 / U1–U5 / T1–T5 / P1–P2

按问卷建议；**U4=B**（固定拓扑 + directory 边）；**U5** 跟 R2（立即写盘）；**T3=A**（本迭代 `roles/*` 迁 v2，与 P6∥P2）。

---

## Git 提交提醒（执行纪律）

每完成一个阶段应 **`git commit`**，避免「计划勾选完成但仓库无提交」：

1. `docs(handoff): blueprint v2 decisions frozen` — 本文件 + RFC/计划状态
2. `feat(validation): pipeline.ocblueprint v2 schema and CLI validate` — Rust + Schema + CLI + 模板 + 测试

发 PR 前：`cargo test -p oclive_validation` 与 `cargo test -p oclive-cli`。勿提交 `src-tauri/Cargo.toml` 仅行尾变更。
