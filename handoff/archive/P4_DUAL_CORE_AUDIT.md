# Phase 4 · dual_core / expert_routing 审计（2026-06-05）

## 范围

| 模块 | 位置 | 约行数 | 编译门控 |
|------|------|--------|----------|
| `dual_pipeline*` | `kernel/crates/oclive_kernel_host/src/domain/dual_pipeline*.rs` | ~800+ | `#[cfg(feature = "dual_core")]` |
| `expert_routing` | `kernel/crates/oclive_kernel_host/src/domain/expert_routing.rs` | ~200+ | 部分路径随 dual_core |
| 集成测 | `distros/desktop-tauri/tests/dual_core_happy_path.rs` | — | CI `--features dual_core` + OOCP S13/S14 |

## 路线对齐

- **RFC**：[`creator-docs/rfc/RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md`](../creator-docs/rfc/RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md) — Stable / Experimental 双态，**Opt-in Beta，默认关**。
- **Handoff**：[`handoff/DUAL_CORE_CURSOR_HANDOFF.md`](DUAL_CORE_CURSOR_HANDOFF.md) — 与 Monolith 构建态正交，**不阻塞 v0.2 交付**。
- **路线图**：[`creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md`](../creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md) 未将 dual_core 列为当前季度必达用户功能。

## 决策：**保留于主仓，维持 feature 门控**

| 选项 | 结论 |
|------|------|
| 本季度移入独立 crate/分支 | **否** — CI 与 OOCP S13/S14 已依赖；拆出增加发行版矩阵成本 |
| 删除代码 | **否** — 实验场与 pack-editor 双核预览仍引用 |
| 维持现状 | **是** — `dual_core` 默认关；文档与 `process_message` 分支清晰 |

## 携带成本

- 全量 `cargo build` 不含 dual_core 时 **零链接成本**（条件编译）。
- 维护面：API `expert.rs`、蓝图 `runtime_config.dual_core`、OOCP 双核场景。
- 下一 review 触发条件：路线图将「双核默认开」列入里程碑，或 6 个月内无 pack-editor/OOCP 双核用例更新。

## 建议后续（非本 PR）

1. 在 `AGENTS.md` 保持「默认关、CI 开 feature 测」一句，避免 Agent 误改 Stable 路径。
2. 若 Open Lab 延期，可将 `dual_pipeline*` 迁至 `kernel/crates/oclive_dual_core_experimental`（仍 feature 依赖），降低 `domain/mod.rs` 视觉噪音。
