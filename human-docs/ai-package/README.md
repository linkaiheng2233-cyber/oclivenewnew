# AI 接手包说明

> **读者**：使用 Cursor / Codex 等 Agent 的维护者。  
> **人类开发者**：请先 [`human-docs/`](../README.md)（L0–L2 · **长文、好读**）；**不要**从 `AGENTS.md` 起步。

---

## 组成

| 部分 | 路径 | 何时读 |
|------|------|--------|
| **AI 深读分类目录** | [`handoff/AI_READING_INDEX.md`](../../handoff/AI_READING_INDEX.md) | **系统了解项目 · 按任务翻 SSOT** |
| **Agent 索引（精简）** | [`AGENTS.md`](../../AGENTS.md) | 每次让 AI 改代码前 |
| **改动 + 文档纪律** | [`AI_CHANGE_BOUNDARIES`](../../handoff/AI_CHANGE_BOUNDARIES.md) G1–G16 | 改代码/改文档 |
| **模块注册表** | [`MODULE_MAP`](../../handoff/MODULE_MAP_AND_HANDOFF.md) | 模块/六槽/设施关系 |
| **文档分责** | [`handoff/README`](../../handoff/README.md) §文档分责 · §文档分层 | **新建/大改文档前** |
| **Cursor 规则** | [`.cursor/rules/oclivenewnew.mdc`](../../.cursor/rules/oclivenewnew.mdc) | 7 条硬约束；人类版见 [04 工程约束](../04_ENGINEERING_RULES.md) |
| **契约百科** | [`creator-docs/`](../../creator-docs/) | manifest、六槽、插件、角色包 |
| **英文镜像** | [`creator-docs-en/`](../../creator-docs-en/) | 对外英文；契约以中文 `creator-docs/` 为准 |
| **维护者深读** | [`handoff/`](../../handoff/) | Bus factor、技术债 |

**物理分层**：Rust 内核在 [`kernel/`](../../kernel/)（`kernel/crates/`、`kernel/fuzz/`）；桌面 / Chat Pro / Theater 前端与 Tauri 宿主在 [`distros/`](../../distros/)（`shared`、`chat-pro`、`theater`、`desktop-tauri`）。

---

## 与 human-docs 分工

| | human-docs | AI 包（本文 + AGENTS + handoff） |
|--|------------|-----------------------------------|
| **篇幅** | 可长可细 · 阶梯阅读 | 短索引 · SSOT 链出 |
| **进度** | [README §文档包进度](../README.md#文档包进度与-ai-包同步--2026-06-25) | [TECHNICAL_DEBT §1](../../handoff/TECHNICAL_DEBT_INVENTORY.md) |
| **文档纪律** | [04 §8 人类版](../04_ENGINEERING_RULES.md#8-文档贡献纪律人类版) | G10–G16 · §文档编写纪律 |

**效率源于限制**：AI **不**在 AGENTS 复制 MODULE_MAP；人类 **不**在 human-docs 复制 PLUGIN_V1 全文。改架构时 **同 PR** 更新 MODULE_MAP + 相关 human-docs 节 + human-docs README 进度日期。

---

## 下一篇

- 人类自学：[human-docs/README.md](../README.md)
- 文档总索引：[DOCUMENTATION_INDEX](../../creator-docs/getting-started/DOCUMENTATION_INDEX.md)
