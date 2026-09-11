# AI 接手包说明

> **读者**：使用 Cursor / Codex 等 Agent 的维护者。  
> **人类开发者**：请先 [`human-docs/`](../README.md)（L0–L2 · **长文、好读**）；**不要**从 `AGENTS.md` 起步。

---

**本页只保留兼容导航，不是另一套 AI 规则。** 人类与 AI 共用专题事实 owner；文档状态与防漂移规则统一见 [handoff/README](../../handoff/README.md#documentation-status)。当前权限、暂停项和任务授权须先核对，不能由旧交接中的命令自动恢复工作。

## 组成

| 部分 | 路径 | 何时读 |
|------|------|--------|
| **AI 深读分类目录** | [`handoff/AI_READING_INDEX.md`](../../handoff/AI_READING_INDEX.md) | **系统了解项目 · 按任务翻 SSOT** |
| **Agent 索引（精简）** | [`AGENTS.md`](../../AGENTS.md) | 每次让 AI 改代码前 |
| **改动 + 文档纪律** | [`AI_CHANGE_BOUNDARIES`](../../handoff/AI_CHANGE_BOUNDARIES.md) G1–G17 | 改代码/改文档 |
| **模块注册表** | [`MODULE_MAP`](../../handoff/MODULE_MAP_AND_HANDOFF.md) | 模块/六槽/设施关系 |
| **文档分责** | [`handoff/README`](../../handoff/README.md#documentation-status) | **新建/大改文档前：owner、状态、镜像、门禁** |
| **Cursor 规则** | [`.cursor/rules/oclivenewnew.mdc`](../../.cursor/rules/oclivenewnew.mdc) | 7 条硬约束；人类版见 [04 工程约束](../04_ENGINEERING_RULES.md) |
| **开发流水线** | 通用 `~/.cursor/skills/dev-pipeline/` · OCLive [`.cursor/skills/oclive-dev-pipeline/`](../../.cursor/skills/oclive-dev-pipeline/SKILL.md) | 通用七阶段框架 + 本仓 G/验收/纪律定制；Plan→实现→审查→**工程纪律**→文档→总审 |
| **契约百科** | [`creator-docs/`](../../creator-docs/) | manifest、六槽、插件、角色包 |
| **英文镜像** | [`creator-docs-en/`](../../creator-docs-en/) | 对外英文；契约以中文 `creator-docs/` 为准 |
| **维护者深读** | [`handoff/`](../../handoff/) | Bus factor、技术债 |

**物理目录不等于逻辑权责**：[`kernel/`](../../kernel/) 包含当前契约和完整参考运行时；[`distros/`](../../distros/) 包含界面与集成代码。不要据此推导 Host 只在 `distros/`，或 `kernel/` 全部属于最小 Kernel；查看 [权责与源码对照](../../handoff/MODULE_MAP_AND_HANDOFF.md#kernel-source-map)。

---

## 与 human-docs 分工

| | human-docs | AI 包（本文 + AGENTS + handoff） |
|--|------------|-----------------------------------|
| **篇幅** | 可长可细 · 阶梯阅读 | 短索引 · SSOT 链出 |
| **阅读 / 状态** | [学习路线](../README.md)；不维护第二份项目进度 | 状态统一查 [TECHNICAL_DEBT](../../handoff/TECHNICAL_DEBT_INVENTORY.md) |
| **文档纪律** | [04 §8 人类版](../04_ENGINEERING_RULES.md#8-文档贡献纪律人类版) | G10–G16 · §文档编写纪律 |

**效率源于限制**：AI **不**在 AGENTS 复制 MODULE_MAP；人类 **不**在 human-docs 复制 PLUGIN_V1 全文。改架构时 **同 PR** 更新 MODULE_MAP + 相关 human-docs 节 + human-docs README 进度日期。

---

## 下一篇

- 人类自学：[human-docs/README.md](../README.md)
- 文档总索引：[DOCUMENTATION_INDEX](../../creator-docs/getting-started/DOCUMENTATION_INDEX.md)
