# 03 · 术语表

> **读者**：读源码或 PR 前需要统一缩写的工程师。  
> **读完能做什么**：看到 `srid` / `mrid` / `pl` 等缩写知道含义；区分 `slot_registry` 与 `plugin_backends`。  
> **耗时**：约 20 分钟（可边查边用）。  
> **下一篇**：[04 工程约束](04_ENGINEERING_RULES.md)。

---

## 会话与角色 ID

| 缩写 | 全称 | 含义 | 代码锚点 |
|------|------|------|----------|
| **`mrid`** | manifest role id | 角色包 `manifest.json` 里的角色 ID | `SendMessageRequest.role_id` |
| **`srid`** | session-scoped role id | SQLite / 缓存命名空间：默认等于 `mrid`；有 `session_id` 时变为 `{mrid}::{session_id}` | [`conversation_state_role_id`](../crates/oclive_kernel_host/src/domain/chat_engine/mod.rs) |
| **`pl`** | plugin layer / resolved plugins | 本回合解析后的 `ResolvedRolePlugins`（六槽 `Arc<dyn …>` 句柄集） | `process_message` 内 `pl` 变量 |

**示例**：HTTP 试用聊天带 `session_id` 时，记忆与 `role_runtime` 行按 **`srid`** 隔离，不与默认会话混用。

---

## 架构核心

| 术语 | 含义 |
|------|------|
| **`PluginHost`** | 按角色包 + 会话覆盖解析六槽实现；入口 [`plugin_host/mod.rs`](../crates/oclive_kernel_host/src/domain/plugin_host/mod.rs) |
| **`slot_registry`** | v2 蓝图多实例槽位表（`pipeline.ocblueprint`）；权威键 `type`: memory / emotion / … |
| **`plugin_backends`** | legacy 六键折叠结构 + Rust 运行时类型名 `PluginBackends`；**非** v2 新 UI 首选名 |
| **`slot_registry.type`** | 与六槽键同义；**禁止**别名 `memory_backend` 等 |
| **`OOCP`** | OCLive Open Chat Protocol；HTTP 黑盒测试场景 S0–S12 |
| **`co_present`** | Stable 核共景主路径实现模块 |
| **第 3 设施（立绘）** | `portrait_catalog` · AI **表现导演** 选 `visual_state_id`（RFC 草案；v0.3 仍为文件名 + 七 tag） |
| **第 4 设施（视觉表现）** | **角色舞台**：Live2D / 3D / 演算 adapter（RFC 草案；默认关） |

---

## 磁盘与配置

| 术语 | 含义 |
|------|------|
| **角色包** | `roles/{id}/`：身份、人格、`prompts/` |
| **蓝图文件** | `pipeline.ocblueprint`（文件名冻结）；含 `slot_registry`、`groups` |
| **`{app_data}`** | Tauri 应用数据目录；含 **`app.db`** |
| **`reply`** | AI 回复契约字段名；**不是** `response` |

---

## 与 NAMING 交叉引用

完整权威名、禁止别名、crate 边界：[creator-docs/NAMING_CONVENTIONS.md](../creator-docs/NAMING_CONVENTIONS.md)

- §1.3 六槽键名
- §4.2 Canonical import
- §5 `slot_registry` vs `plugin_backends`
- §6 禁止别名表

---

## 验收

- [ ] 能解释 `srid` 与 `mrid` 何时相同、何时不同
- [ ] 能区分 `slot_registry`（v2 磁盘）与 `PluginBackends`（运行时折叠）

---

## 深度链接

- [ROLE_PACK_BOUNDARY](../handoff/ROLE_PACK_BOUNDARY.md)
- [SETTINGS_REFERENCE](../creator-docs/cli/SETTINGS_REFERENCE.md)
