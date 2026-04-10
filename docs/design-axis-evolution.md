# 设计轴心：思路变化记录

本文记录 **性格与人设相关设计思路的演进**，方便后来者与协作者理解「为什么现在这样设计」。

**原则**：旧交接、路线图、`handoff/` 等材料 **不删**；若与当前产品语义不一致，以 **[personality-archive-notes.md](./personality-archive-notes.md)**、**[roles/README_MANIFEST.md](../roles/README_MANIFEST.md)** 与 **源码** 为准，旧文仅作历史参考。

---

## 以前（简化理解）

- 性格呈现主要围绕 **manifest 七维**（`default_personality`）与 **事件 / 演化数值**；**`core_personality.txt`** 作为长文补充，边界不如现在清晰。
- 未统一区分 **包内锁定的基底** 与 **运行时可由模型维护的状态**；缺少跨编写器 / 运行时 / 启动器的同一套词汇（核心档案、可变档案、视图）。

## 现在（设计轴心）

- **人设档案** = **核心性格档案**（包内 **`core_personality.txt`**，运行时 **不得**由模型改写）+ **可变性格档案**（仅存运行时 DB，在 **`evolution.personality_source: profile`** 下由模型在对话后维护）。
- **七维**：在 **`profile`** 下多为从「核心 + 可变」正文归纳的 **视图**，包内数组仍作默认与参考；在 **`vector`**（经典，默认）下仍以 **数值演化路径** 为主。
- 创作者 **不能**手写运行时可变档案正文，只能通过 **`evolution`**（如 **`max_change_per_event`**、`event_impact_factor`）调强弱。
- 远期「全 AI 情感 / 成长引擎」**只备忘、不立项实现**（见 `personality-archive-notes.md` 末节）。

## 推进方式（后续工作）

- **文档**：`creator-docs/`、外仓 README / CHANGELOG 与本文、`personality-archive-notes` 保持互链；无害的旧表述可保留。
- **契约与模板**：`settings.template.json` 等显式写出 **`personality_source`**，降低「只有七维」的误解。
- **产品 UI**：提示文案区分 **用户关系身份**（`user_relations`）与 **性格档案**，避免「人设」一词多义。

## 相关入口

- [personality-archive-notes.md](./personality-archive-notes.md) — 轴心定义与三应用分工  
- [roles/README_MANIFEST.md](../roles/README_MANIFEST.md) — `default_personality`、`evolution` 字段说明  
- [creator-docs/getting-started/CREATOR_WORKFLOW.md](../creator-docs/getting-started/CREATOR_WORKFLOW.md) — 创作者工作流中的档案说明  
