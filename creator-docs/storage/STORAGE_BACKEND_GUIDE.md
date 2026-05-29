# 聊天记录存储后端选择指南

> 技术真源：[handoff/CHAT_STORAGE_ARCHITECTURE.md](../../handoff/CHAT_STORAGE_ARCHITECTURE.md) · 配置字段见 [SETTINGS_REFERENCE.md §六](../cli/SETTINGS_REFERENCE.md) · 角色包 `config.json` 亦见 [ROLE_PACK_SPEC.md §9](../role-pack/ROLE_PACK_SPEC.md)。

## 三种后端对比

| | **hybrid**（默认） | **file** | **sqlite** |
|---|-------------------|----------|------------|
| 聊天真源 | SQLite + JSON 镜像 | JSON 文件 | SQLite |
| 数据路径 | DB + `{app_data}/chats/{role}/{scene}/*.json` | `{app_data}/chats/{role}/{scene}/*.json` | DB only |
| 搜索 | SQLite LIKE | 遍历 JSON（需 `role_id`） | SQLite LIKE |
| 自动清理 | ✅ | ❌ | ✅ |
| 记忆回放 | ✅ | ✅（读 JSON 聊天；**写** `long_term_memory` 仍走宿主 SQLite） | ✅ |
| 适用场景 | 桌面主应用；需要透明文件 + DB 性能 | 轻量/嵌入式；用户直接管理 JSON | 性能优先；不需要镜像文件 |

## 如何切换

**优先级**（高 → 低）：

1. 环境变量 **`OCLIVE_CHAT_STORAGE_BACKEND`** = `hybrid` \| `file` \| `sqlite`（进程级，覆盖角色包）
2. 角色包 **`config.json`** → `chat_storage.backend`
3. 默认 **`hybrid`**

**脚手架**：`oclive-cli init` 交互步骤「Chat history storage backend」会写入生成包的 `config.json`。

**注意**：切换后端需重启宿主；不会自动迁移已有聊天数据到另一种布局。

## File 后端限制

- **不支持自动清理**（`supports_cleanup: false`）；用户可自行删除 `{app_data}/chats/` 下 JSON
- 搜索必须带 **角色 id**（存储管理面板在角色上下文中搜索）
- 记忆回放读取 JSON 聊天，但 **写入** `long_term_memory` 仍走宿主 SQLite

## 记忆回放阈值

`config.json` → `chat_storage.replay_similarity_threshold`（默认 **0.6**，范围 **0.1–1.0**）：

- **更高**（如 0.9）：更严格去重，更少「相似记忆」合并
- **更低**（如 0.3）：更宽松，更多内容视为同一条记忆并累加 `mention_count`

设置 → 存储管理 →「重新提取记忆」会读取当前角色配置并传入回放任务。

### 存储位置

`config.json` → `chat_storage.location` 可设为：

- `"role_pack"`（推荐）：聊天记录跟随角色包存储在 `roles/{id}/chats/`，分享/迁移角色包时对话历史一同移动
- `"global"`（默认）：存储在 `{app_data}/chats/`，与角色包位置独立

当角色包位于只读文件系统或不可写时，自动退回 `global` 并记录 warn 日志。

## 推荐

| 场景 | 推荐 |
|------|------|
| 日常桌面对话、需要导出/镜像文件 | **hybrid** |
| 开发调试、嵌入式、用户要直接看/拷 JSON | **file** |
| 大量会话、不关心磁盘镜像、要最低 I/O | **sqlite** |

---

[English](../creator-docs-en/storage/STORAGE_BACKEND_GUIDE.md)
