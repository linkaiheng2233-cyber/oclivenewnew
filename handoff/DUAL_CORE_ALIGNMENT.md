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
| P2–P5 调度器 / 宿主 / Monolith | **未开始** |
| v2 交付 / 插件极简 | **已闭环** |

---

## 术语

| 术语 | 层次 | 今天 |
|------|------|------|
| 单核双态**构建** | 编译期 | **有** |
| 双核双态**运行时** | Stable + Experimental | **校验 only**；调度 **未接线** |

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
