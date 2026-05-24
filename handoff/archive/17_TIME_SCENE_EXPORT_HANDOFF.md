# 虚拟时间 / 场景 / 导出 — 交接说明（DeepSeek 审查对齐）

本文档与审查意见对齐，记录**交接确认项**、实现要点与**待办**。

## 交接确认（Cursor 已核对）

### 1. `virtual_time_ms` 按角色隔离

- 存储位置：`role_runtime` 表，主键为 **`role_id`**。
- 读写均为 `WHERE role_id = ?`，见 `DbManager::get_virtual_time_ms` / `set_virtual_time_ms`。
- `get_time_state` / `jump_time` 初始化与更新均传入**当前角色的** `role_id`，**不会**跨角色覆盖。

### 2. 场景 ID 与前端 fallback

- 后端：`switch_scene` 拒绝不在角色包场景列表中的 `scene_id`；`get_role_info` 返回 `scenes` 与 `current_scene`。
- 前端：`App.vue` 中 `resolveSceneId()` —— 若 `current_scene` 在 `scenes` 中则采用；否则若本地持久化的 `sceneId` 仍在新列表中则保留；否则取列表首项或 `"default"`。  
  避免 manifest 变更或脏数据导致下拉框指向非法值。

### 3. 导出功能

- **后端**：`export_chat_logs` 返回 `content` + `suggested_filename`（JSON/TXT），数据来自 `short_term_memory`。
- **前端**：`ChatExportBar.vue` 调用 `exportChatLogs`，`src/utils/download.ts` 使用 **Blob + `<a download>`** 触发浏览器下载（Tauri WebView 下通常进入用户下载目录，**无需**额外 Rust `dialog`/`fs` 权限）。可选「导出全部角色」。

## 实现索引

| 内容 | 位置 |
|------|------|
| 命令注册 | `src-tauri/src/lib.rs` → `switch_scene`、`get_time_state`、`jump_time`、`generate_monologue`、`export_chat_logs` |
| 分钟取整 | `src-tauri/src/api/time.rs` → `round_to_minute_ms`（含单元测试） |
| 迁移 | `src-tauri/migrations/005_add_virtual_time.sql` |
| 场景列表 | `src-tauri/src/infrastructure/storage.rs` → `list_scene_ids`；`roles/mumu/manifest.json` 顶层 `scenes` |
| 短期对话 FIFO | `src-tauri/src/infrastructure/db.rs` → `SHORT_TERM_FIFO_LIMIT`，事务内写入 `short_term_memory` |
| 前端场景解析 | `src/App.vue` → `resolveSceneId` |
| 虚拟时间 UI | `src/components/VirtualTimeBar.vue`（`datetime-local` 分钟步长 + `jump_time`；「独白」插入聊天） |
| 导出 UI | `src/components/ChatExportBar.vue`、`src/utils/download.ts` |

## 质量门禁建议

- 日常：`cargo fmt`、`cargo clippy --all-targets -- -D warnings`、`cargo test --lib --tests`、`npm run build`。
- `cargo test` 含 **doctest** 时可能因历史文档示例失败，与本轮功能无关；可择机清理示例代码。

## 发布前建议（与审查一致）

- 联调：角色切换、场景切换、时间跳转（`jump_time`）、导出下载。
- 打包：`cargo tauri build`，干净环境安装验证。
- P2：圆形时间拨盘皮肤、从 `scene.json` 读场景中文名。

## 结论

后端与前端核心体验（虚拟时间条、导出下载、独白插入）已落地；剩余多为视觉与文案增强。
