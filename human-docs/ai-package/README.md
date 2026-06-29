# AI 接手包说明

> **读者**：使用 Cursor / Codex 等 Agent 的维护者，或需要契约百科的创作者。  
> **耗时**：按需查阅；日常改代码前读 `AGENTS.md` + `.cursor/rules` 即可。  
> **人类开发者**：请先 [`human-docs/`](../README.md)（L0–L2 约 1 小时）。

---

## 组成

| 部分 | 路径 | 何时读 |
|------|------|--------|
| **Agent 约束** | [`AGENTS.md`](../../AGENTS.md) | 每次让 AI 改本仓代码前 |
| **Cursor 规则** | [`.cursor/rules/oclivenewnew.mdc`](../../.cursor/rules/oclivenewnew.mdc) | 7 条硬约束镜像；人类可读版见 [04 工程约束](../04_ENGINEERING_RULES.md) |
| **创作者契约** | [`creator-docs/`](../../creator-docs/) | manifest、六槽、插件协议、角色包规范 |
| **英文镜像** | [`creator-docs-en/`](../../creator-docs-en/) | 对外英文；契约以中文 `creator-docs/` 为准 |
| **维护者深读** | [`handoff/`](../../handoff/) | Bus factor、发版清单、技术债；新人请先 human-docs L5–L7 |
| **文档纪律** | [`AI_CHANGE_BOUNDARIES` §文档纪律](../../handoff/AI_CHANGE_BOUNDARIES.md#文档纪律精简) | 入口/契约 SSOT、禁止增殖 handoff、archive 非 truth |

**物理分层**：Rust 内核在 [`kernel/`](../../kernel/)（`kernel/crates/`、`kernel/fuzz/`）；桌面 / Chat Pro / Theater 前端与 Tauri 宿主在 [`distros/`](../../distros/)（`shared`、`chat-pro`、`theater`、`desktop-tauri`）。

---

## 与 human-docs 分工

- **human-docs**：窄入口、时间盒、顺序阅读；**不复制**长文 SSOT。
- **AI 包**：完整契约、RFC、handoff 归档；**不搬家**，仅通过 human-docs [08 资料地图](../08_REFERENCE_MAP.md) 反向链接。

需要契约细节时：从 human-docs 链到 AI 包，不要在 human-docs 里维护第二份长文。

---

## 下一篇

- 人类自学：[human-docs/README.md](../README.md)
- 文档总索引：[DOCUMENTATION_INDEX](../../creator-docs/getting-started/DOCUMENTATION_INDEX.md)
