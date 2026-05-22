# 双核双态 · 对齐速查

**Cursor 主文档**：[DUAL_CORE_CURSOR_HANDOFF.md](DUAL_CORE_CURSOR_HANDOFF.md)  
**RFC**：[creator-docs/rfc/RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md](../creator-docs/rfc/RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md)  
**角色包边界**：[ROLE_PACK_BOUNDARY.md](ROLE_PACK_BOUNDARY.md)

---

## 进度（2026-05）

| 阶段 | 状态 |
|------|------|
| P0 文档 + Q1–Q20 | **已完成** |
| 角色包 / 蓝图 / `runtime_config` 文档 | **已完成** |
| P1 `validate_blueprint_v3` | **已完成（crate）** |
| `pack validate --profile creator` | **已完成** |
| P2–P5 执行计划 | [DUAL_CORE_P2_P5_EXECUTION_PLAN.md](DUAL_CORE_P2_P5_EXECUTION_PLAN.md) |
| P2 宿主加载 + `DualPipelineRunner` + `process_message` 门控 | **已完成** |
| P3 `oclive init --dual-core` | **已完成** |
| P4 OOCP S13（`--include-s13` / 可选 job） | **已完成** |
| P5 Monolith `--dual-core`（`monolith.toml` + 生成注释） | **已完成（脚手架层）** |
| 深化：七槽 method、快照扩展、METHOD_REGISTRY、架构图、DEVELOPER_GUIDE、`oclive explain DUAL_CORE` | **已完成** |
| 最终精修：CI 一致性验证、集成测回归、性能解读文档、双核日志 | **已完成**（见 [ARCHITECTURE_LAYERING.md](ARCHITECTURE_LAYERING.md) § 最终精修） |
| Q21–Q29 | **已答复**（见 [DUAL_CORE_CURSOR_HANDOFF.md](DUAL_CORE_CURSOR_HANDOFF.md) §九） |
| v2 交付 / 插件极简 | **已闭环** |

---

## 术语

| 术语 | 层次 | 今天 |
|------|------|------|
| 单核双态**构建** | 编译期 | **有** |
| 双核双态**运行时** | Stable + Experimental | **默认关**；`dual_core.enabled` + 非空 `experimental` 时接线 |

---

## 已确认（Q16–Q20 摘要）

| ID | 决议 |
|----|------|
| Q16 | schema **2/3 分流**校验 |
| Q17 | 只校验 registry **键** |
| Q18 | 迁移工具 **延后** |
| Q19 | 无 `pipeline.stable` → **co_present** |
| Q20 | P4 仅 **七种 PluginHost type** |

---

[English RFC](../creator-docs-en/rfc/RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md)
