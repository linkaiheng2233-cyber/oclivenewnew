# Changelog

## [Unreleased]

### Engineering

- **共享 crate `crates/oclive_validation`**：`validate_disk_manifest`、`parse_hhmm`、`KnowledgePackConfigDisk` 等与磁盘 manifest 相关的校验与 DTO 单一来源；运行时依赖该 crate，编写器可用 **wasm**（`--features wasm`，目标 `wasm32-unknown-unknown`）调用 `validate_manifest_wasm`。
- **本地 HTTP API**：可执行文件支持 `--api` / `--port`（或 `OCLIVE_API_PORT`），默认 `http://127.0.0.1:8420`，提供 `GET /health`、`POST /chat`（`role_path` + `message`，可选 `session_id`），供编写器试聊等工具调用。`session_id` 会映射为内部 SQLite 会话键 `{manifest_role_id}__sess__{sanitized}`，与无 `session_id` 的默认对话隔离；JSON 响应含 `reply` 与回显的 `session_id`。`POST /chat` 对 **空 `message`** 返回 400；会话键总长度限制为 **256** 字符以防异常输入。
- **角色加载**：若 `plugin_backends` 声明 `remote` 但未设置 `OCLIVE_REMOTE_PLUGIN_URL` / `OCLIVE_REMOTE_LLM_URL`，在 `load_role_from_dir` 成功路径上记 `oclive_plugin` 警告（行为仍为回退内置，与 PLUGIN_V1 一致）。
- **CI**：`oclivenewnew` 在 Ubuntu 与 Windows 上运行 Rust（fmt / clippy / `cargo test`）与 `npm run build`；**oclive-pack-editor**、**oclive-launcher** 各自增加/对齐双平台 workflow。
- **npm**：新增 `npm run check:release`（发版门槛：全量 `cargo test`）；README 补充 Sentry / 离线安装包说明。

### Documentation

- **第 1 月（契约边界）**：`VISION_ROADMAP_MONTHLY.md` 第 1 月与 `plugin_backends` / `PluginHost` 对齐；`PLUGIN_V1.md` 增加 `send_message` 编排顺序；`PACK_VERSIONING.md` 补充里程碑与远程后端加载时警告说明。
- 新增 **[creator-docs/roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md](creator-docs/roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md)**：体验差异化方向（编写器试聊、启动器依赖、市场/UGC）与愿景对照；`VISION_ROADMAP_MONTHLY.md`、文档索引与根 README 已链入。
- 新增 **[creator-docs/getting-started/PROJECT_OVERVIEW.md](creator-docs/getting-started/PROJECT_OVERVIEW.md)**：三件套分工、已落实能力、文档地图、命令、人机分工、愿景对照与发版极简清单。
- **oclive-launcher**（另仓）：环境与排障页（Node/npm/Ollama/项目路径检测、重置配置、打开配置目录）；**[BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md](creator-docs/roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md)** 补充启动器已落实项与长期路线引用。
- **GitHub**：[GITHUB_REPO_CHECKLIST.md](creator-docs/getting-started/GITHUB_REPO_CHECKLIST.md)；Dependabot、PR 模板、CI 手动触发（`workflow_dispatch`）、README CI 徽章；**oclive-pack-editor** / **oclive-launcher** 同步 Dependabot 与 PR 模板。

---

## [0.2.0] — 2026-04-02

### Added

- 大角色包导入进度：后端 `import_progress` 事件 + 前端导入进度条模态框。
- 角色包导入前预览（`manifest.json` peek）与冲突处理：当角色 ID 已存在时弹出“覆盖/取消”确认。
- 角色包导入支持 **`.zip`**（与 `.ocpak` 相同容器）以及 **已解压目录**（与 `roles/{角色id}/` 布局一致）；见 `roles/README_MANIFEST.md`。
- 场景切换欢迎语：`switch_scene` 成功后读取 `scene.json` 的 `welcome_message`（或稳定随机 monologue）并自动插入聊天区人设消息。
- 关系阶段升级提示：`send_message` 响应增加 `relation_state`，前端在“升级”时插入系统消息。

### Changed

- 虚拟滚动策略：`ChatMessageList` 在有消息时始终启用虚拟滚动（减少 DOM 压力）。
- 角色包导出命名：导出文件默认改为 `{role_name}_{version}.ocpak`（安全化文件名）。

### API

- `send_message` 响应新增 `relation_state`；`emotion` 仍表示用户输入侧七维分析。

### Frontend

- 顶栏与聊天展示继续使用 `bot_emotion` 驱动立绘与情绪图片/emoji。

### Documentation

- 创作者文档迁至根目录 **`creator-docs/`**（分 `getting-started`、`plugin-and-architecture`、`role-pack`、`roadmap`）；旧 `docs/*.md` 迁移说明见 `docs/README.md`。开发日志归档见 **`ARCHIVE_PROJECT_HISTORY.md`**。
- **`roles/README_MANIFEST.md`**：新增「在 oclive 中导入角色包」；**`CREATOR_WORKFLOW.md`**、**`DOCUMENTATION_INDEX.md`**、根 **`README.md`** 同步应用内导入说明。
- **`roles/TESTING_ROLE_PACK_IMPORT.md`**：角色包导入手工测试清单；压缩包预览优先根目录 `manifest.json` 等行为见 **`role_pack.rs`**。
- 详见 `handoff/20_SESSION_OPTIMIZATION_REPORT.md`。

---

## [0.1.0]

- 初始公开基线（以仓库内首次标记版本为准）。
