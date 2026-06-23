# 聊天存储后端收敛 · hybrid + mirror（破坏性摘要）

**日期**：2026-06-05 · 对应优化计划阶段三 `p3-storage-collapse`

## 变更

| 旧配置 | 新行为 |
|--------|--------|
| `backend: hybrid` | SQLite 真源 + `mirror: true`（默认） |
| `backend: sqlite` | 同 hybrid，`mirror: false`（**deprecated** 枚举，仍可反序列化） |
| `backend: file` | 同 hybrid，`mirror: true`；**独立 `FileConversationStore` 已移除** |
| `OCLIVE_CHAT_STORAGE_BACKEND=file` | 映射 hybrid + mirror on，启动时 `warn` |

## 新增配置键

`config.json` → `chat_storage.mirror`：`true` | `false` | 省略（按上表 legacy `backend` 推导）

## 迁移

- 纯 **file** 后端用户：需将既有 JSON 会话 **import** 至 SQLite（设置 → 存储管理 / `migrate_indexeddb_to_backend` 同类路径），或一次性脚本读 `{app_data}/chats/`。
- **sqlite** 用户：无数据迁移；`get_chat_storage_capabilities().backend_kind` 仍报告 `"sqlite"`（mirror off）。

## 契约

- `kernel/crates/oclive_kernel_types` → `RolePackChatStorageConfig.mirror`
- `get_chat_storage_capabilities` → 新增 `default_max_messages_per_session`
- 前端 `RoleChatStorageConfig.mirror` 可选

详见 [`handoff/BREAKING_CHANGE_PROCESS.md`](BREAKING_CHANGE_PROCESS.md) 审阅清单（validation / 设置 UI 已同步）。
