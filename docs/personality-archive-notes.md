# 人设档案：设计轴心与备忘

**思路为何如此演进**：见 **[design-axis-evolution.md](./design-axis-evolution.md)**（旧文档不删，以本页与 `README_MANIFEST` 为准）。

## 设计轴心（当前共识）

- **人设档案**由两部分组成：
  - **核心性格档案**（包内 **`core_personality.txt`**）：由创作者锁定；运行时 **不得**由模型改写该正文。
  - **可变性格档案**（运行时 DB：**`mutable_personality`**）：仅在 **`evolution.personality_source: profile`**（人格来源选「档案」）时，由模型在对话回合后维护；创作者 **不能**在包内手写该正文，只能通过 **`evolution`**（如 `event_impact_factor`、`max_change_per_event`）调强弱。
- **七维**（manifest 的 **`default_personality`** 等）：在 **`profile`** 模式下多为从「核心 + 可变」正文**归纳的视图**，便于 UI 与人理解；**经典**（**`vector`** 或省略）时仍以数值增量等为主驱动。包内七维仍建议填写，作为默认与兜底参考。

契约与字段说明以 **[roles/README_MANIFEST.md](../roles/README_MANIFEST.md)**（`default_personality`、`evolution`）为准；编写器 **oclive-pack-editor** 简单创作已暴露 **`personality_source`** 与 **`max_change_per_event`**。

## 实现与分工（三应用）

| 应用 | 职责 |
|------|------|
| **oclivenewnew** | 加载包、解析 `personality_source`、维护 DB 中的可变档案、组装提示词；源码见 `profile_personality`、`mutable_profile_llm` 等（以代码为准）。 |
| **oclive-pack-editor** | 编辑包内 **`core_personality.txt`**、`manifest` / **`settings.json`**（含 `evolution`）；不接触运行时可变正文。 |
| **oclive-launcher** | 拉起编写器与运行时；不编辑角色内容。界面说明见该仓 README。 |

## 远期议题（未立项）

**完全由模型驱动的情感 / 成长引擎**（备忘，待产品与技术再议）：

- 类比人类成长：经历不同事件，**选择性唤醒**不同内在侧面，再组合成更不可复制的长期表现。
- 与现状差异：当前可变档案仍是「单通道正文 + 配置封顶」；远期若做，可能涉及多槽状态、事件路由、与记忆/关系的更深耦合等，需单独设计与评审。有结论后更新本页与相关实现。
