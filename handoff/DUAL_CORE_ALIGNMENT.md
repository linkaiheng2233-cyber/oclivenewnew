# 双核双态 · 对齐速查

**Cursor 主文档**（含进度与仓库计划关系）：[DUAL_CORE_CURSOR_HANDOFF.md](DUAL_CORE_CURSOR_HANDOFF.md)  
**RFC 全文**：[creator-docs/rfc/RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md](../creator-docs/rfc/RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md)

---

## 进度（2026-05）

| 阶段 | 状态 |
|------|------|
| P0 文档 + Q1–Q14 已决 | **已完成** |
| 第二轮 Q15–Q20 | **待答复** |
| P1–P5 实现 | **未开始** |
| v2 交付 / 插件极简 | **已闭环** |

---

## 术语

| 术语 | 层次 | 今天 |
|------|------|------|
| 单核双态**构建** | 编译期 PluginHost vs Monolith | **有** |
| 双核双态**运行时** | Stable + Experimental + 快照降级 | **无**（RFC only） |

---

## 与 v2 今日差异（一条表）

| 项 | 今天 | 双核开启后 |
|----|------|------------|
| `slot_registry` | 总表，无 `zone` | 总表 + `zone`（可 **同时** stable & experimental） |
| 编排 | 隐式共景阶段表 | `pipeline.stable` / `pipeline.experimental` + **`depends_on` DAG** |
| 入口 | 无 | `oclive init --dual-core` |
| 失败 | Remote→builtin 等 | 实验核失败 → 快照恢复 → Stable |
| 用户 | 极简插件 UI | **不变** |

---

## 已确认决策（摘要）

1. `action` = `slot.<registry_key>.<method>`；`depends_on` → action 字符串。  
2. `complex_emotion` = 第七设施，**不进** pipeline。  
3. Experimental `type` **完全开放**；校验只查 registry 键。  
4. 降级 **静默**；快照 **仅内存**（P2 MVP）。  
5. **`schema_version: 3`**；P4 标准构建，P5 Monolith 另里程碑。  
6. 详表见 [DUAL_CORE_CURSOR_HANDOFF.md §九](DUAL_CORE_CURSOR_HANDOFF.md#九已决事项2026-05-对齐)。

---

[English](../creator-docs-en/rfc/RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md)
