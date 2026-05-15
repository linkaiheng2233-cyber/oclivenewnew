# Changelog

> **English mirror**: [CHANGELOG.en.md](CHANGELOG.en.md) — 用户可见变更请与中英两份同步维护。

## [Unreleased]

**Kernel / CLI / quality（当前 `main`，应用版本号仍为 package.json / `src-tauri` 的 0.2.0）：**

- **内核编排**：**`process_message`** 为 Tauri 与 HTTP API 的**唯一主编排入口**；**入口蓝图（`pipeline.ocblueprint`）主路径已移除**，子流程在 `chat_engine` 模块内顺序展开。
- **Monolith**：**`oclive-cli`** 四阶段（**`init --monolith`** → **`build`** → 双二进制 **`bench`**）与 **`vendor/oclive_monolith_builtin/`** 焊接桩落地；详见 [RFC_OCLIVE_MONOLITH_MODE.md](creator-docs/rfc/RFC_OCLIVE_MONOLITH_MODE.md) 与 [OCLIVE_CLI_GUIDE.md](creator-docs/cli/OCLIVE_CLI_GUIDE.md)。
- **CLI**：**`oclive dev`**（`roles/` 热文件监听）；**`oclive bench --save` / `--compare`**（`bench_history.json`）；**`oclive pack validate|create|publish`**（角色包校验与 `.oclivepack` 发布）。
- **启动健康检查**：首轮对话前 **`startup_health`**（槽位、包文件、**`DbManager::health_ping`**、可选 LLM 探测）；可用 **`OCLIVE_SKIP_STARTUP_HEALTH`** 等环境变量跳过。
- **错误与可观测性**：**`thiserror`** 统一 **`AppError`**、前端可映射文案；**`tracing`** + **`RUST_LOG`**（CLI / 库 **`init_tracing`** 默认 **`info`**）。
- **静态分析**：工作区 **`[workspace.lints]`** 与 **`cargo clippy ... -D warnings`** 在 CI 与本地 **`check:rust:clippy`** 对齐，告警即失败。
- **前端 i18n**：新增 **`app.documentTitle`**，**`App.vue` / `DirectoryShellApp.vue`** 随语言同步 **`document.title`** 与 **`document.documentElement.lang`**；**`index.html`** 内联脚本按 **`oclive.appLocale`** 与浏览器语言减少首屏页签语言闪动；**整壳 Vue 引导**（`directoryShellBootstrap`）补充 **`app.use(i18n)`**。新增 Vitest **`i18n_locale_parity`** 校验 **zh-CN / en-US** 词条键树一致。

### Added

- 插件清单支持声明订阅的宿主事件（`shell.bridge.events` 或 `ui_slots[].bridge.events`），避免不必要的事件广播。
- 设置页「常规」区域增加「强制 iframe 模式」开关，开启后所有插件界面统一使用 iframe 渲染，获得最高级别沙箱隔离。
- 开发者模式下加载 Vue 插槽组件时，对源码进行静态安全扫描（基于 acorn），检测到危险 API 时弹出警告对话框，由用户决定是否继续。
- **产品线 A1.1（HTTP 子集）**：新增 **`scripts/e2e-core-api-restart.mjs`**（两轮「起 `--api` → `/health` → `POST /chat` → 终止进程」）；根 **`package.json`** 脚本 **`test:e2e:core-api-restart`**；CI **`oocp-test-suite`** 在 OOCP **`run.mjs`** 之后执行该脚本。
- **产品线 A1.2（`invoke` 宿主热路径）**：[`handoff/INVOKE_HOTPATH_MATRIX.md`](handoff/INVOKE_HOTPATH_MATRIX.md) 与集成测 [`src-tauri/tests/invoke_hotpath_matrix.rs`](src-tauri/tests/invoke_hotpath_matrix.rs)（**9** 条 `*_impl`：角色/对话/记忆 + **catalog**、**plugin_state**、**hotkeys**）；[`handoff/PRODUCT_RELEASE_CHECKLIST.md`](handoff/PRODUCT_RELEASE_CHECKLIST.md) §A1 中 **A1.2** 已可勾选；**golden / 全 handler** 仍后续增强。
- **产品线 A1.1b（Web 预览壳）**：**`@playwright/test`** + [`e2e/preview-shell.spec.ts`](e2e/preview-shell.spec.ts)（`vite preview` 首屏）；**`npm run test:e2e:preview`**；CI **`frontend`** job；发版表 **A1.1c** 单列原生安装包 / Tauri 窗延伸项。

### Changed

- 调整切换角色后的事件广播时机，确保插件订阅信息已同步再发送 `role:switched`。
- 插件引导信息（`get_directory_plugin_bootstrap`）返回值增加 `subscribedHostEvents` 字段。

### Fixed

- 修复多个插槽组件并发拉取引导信息时事件订阅集合可能不一致的问题。
- 修复插件自定义事件也被 `bridge.events` 过滤误拦截的问题：仅宿主内置事件走订阅过滤，自定义事件保持可广播。

### Performance

- 对 `get_directory_plugin_catalog` 的 IPC 合并并发 in-flight 请求（全局单次调用）。
- 对 `get_directory_plugin_bootstrap` 的 IPC 按 `role_id` 合并并发请求，减少多插槽同时挂载时的重复调用。
- 对 `get_plugin_state` 的 IPC 按 `role_id` 合并并发请求，并在 `save/reset` 前清理对应 in-flight 键，降低并发下读取陈旧状态的概率。
- 开发者模式下 Vue 插槽：安全扫描读入的源码复用于 `vue3-sfc-loader`，避免对同一 `.vue` 二次 `read_plugin_asset_text`。
- Rust：`directory_plugin_bootstrap_dto` 在构建 `ui_slots` 的同一趟扫描中合并 `subscribed_host_events`，每个已启用插件目录只解析一次 `manifest.json`（整壳 URL 仍单独解析一次）。
- `pluginStore.refresh()` 仅在目录插件 `catalog` / `pluginState` 实际变化时才替换状态对象，减少无效重渲染。
- `setHostEventSubscribedEvents` 增加签名短路，相同订阅集合不重复重建 `Set`。
- `pluginStore.pluginsOrderedForSlot` 增加 slot 级 memo，并将顺序过滤从 `includes` 改为 `Set` 查找，减少频繁读取时的重复排序与线性扫描开销。
- `pluginStore` 在 `catalog` 变化时一次性预计算 `catalogCandidatesBySlot`（已排序），`pluginsOrderedForSlot` 直接复用，避免每次筛选/排序目录插件清单。
- `pluginStore.refresh()` 合并并发调用为共享 in-flight Promise，避免重复拉取 catalog / state / bootstrap；`applyDirectoryBootstrap` 将 `ui_slots` 写入 `bootstrapUiSlots`，各插槽组件从 store 读取，减少重复的 `get_directory_plugin_bootstrap` 前端请求。
- 嵌入插槽（聊天工具栏 / 设置页 / 角色详情）共用 `useDirectoryPluginSlotEmbed` composable；`pluginStore` 在目录 `catalog` 变更时 `slotOrderMemo.clear()`，避免旧 memo 与新区块长期并存。

### Engineering

- **Clippy / rustfmt**：全量 `cargo clippy -- -D warnings` 清零（`is_none_or`、`lines` 读错误处理、`clamp`、`contains`、URI 协议 needless borrow 等）；工作区 **`cargo fmt --all`** 与 CI `rustfmt --check` 对齐。
- **共享 crate `crates/oclive_validation`**：`validate_disk_manifest`、`parse_hhmm`、`KnowledgePackConfigDisk` 等与磁盘 manifest 相关的校验与 DTO 单一来源；运行时依赖该 crate，编写器可用 **wasm**（`--features wasm`，目标 `wasm32-unknown-unknown`）调用 `validate_manifest_wasm`。
- **本地 HTTP API**：可执行文件支持 `--api` / `--port`（或 `OCLIVE_API_PORT`），默认 `http://127.0.0.1:8420`，提供 `GET /health`、`POST /chat`（`role_path` + `message`，可选 `session_id`），供编写器试聊等工具调用。`session_id` 会映射为内部 SQLite 会话键 `{manifest_role_id}__sess__{sanitized}`，与无 `session_id` 的默认对话隔离；JSON 响应含 `reply`、回显的 `session_id`，以及在扁平 `SendMessageResponse` 字段之外同层的 **`personality_source`**（`vector`|`profile`，与包内 `evolution` 一致）。`POST /chat` 对 **空 `message`** 返回 400；会话键总长度限制为 **256** 字符以防异常输入。**`POST /chat`** 在 Tokio 上使用 **`spawn_blocking`** 执行目录探测与 `load_role_from_dir`，避免阻塞异步运行时线程（与 `import_role_pack` 一致）。
- **Tauri**：`peek_role_pack` 预览 manifest 改为 **`spawn_blocking`**，避免在异步命令线程上长时间读压缩包/磁盘。
- **Clippy**：`chat_engine` 编排入口 `process_co_present` / `process_remote_life` 与 `detect_movement_intent` 显式 **`allow(too_many_arguments)`** 并注释原因，使 `cargo clippy -- -D warnings` 与 CI 一致通过。
- **HTTP API 测试**：新增 `tests/http_api_chat.rs`，`tower::oneshot` 覆盖 `GET /health`、`POST /chat`（空消息 400、成功含 `personality_source` + `reply`）；导出 **`api_router`** 与 `serve_api` 共用路由。
- **角色加载**：若 `plugin_backends` 声明 `remote` 但未设置 `OCLIVE_REMOTE_PLUGIN_URL` / `OCLIVE_REMOTE_LLM_URL`，在 `load_role_from_dir` 成功路径上记 `oclive_plugin` 警告（行为仍为回退内置，与 PLUGIN_V1 一致）。
- **角色包契约收紧**：`manifest.json` 可选 **`min_runtime_version`**（semver）；宿主版本低于要求时 **`load_role` 拒绝加载**。根对象 **顶层键白名单**（`oclive_validation::json_keys`）；`manifest` / `settings` 中不允许的顶层键报错，**`_` 前缀说明键**仍允许。共享 crate 增加 **`validate_min_runtime_version`**；wasm 侧 **`validate_manifest_wasm`** 第三参为宿主版本字符串。
- **CI**：`oclivenewnew` 在 Ubuntu 与 Windows 上运行 Rust（fmt / clippy / `cargo test`）与 `npm run build`；**oclive-pack-editor**、**oclive-launcher** 各自增加/对齐双平台 workflow。
- **npm**：新增 `npm run check:release`（发版门槛：全量 `cargo test`）；README 补充 Sentry / 离线安装包说明。
- **界面**：顶栏「身份」HelpHint 区分关系身份与核心性格档案；注释「人设回复」改为「角色回复」。
- **API / UI**：`RoleInfo` 与 `RoleData` 增加 **`personality_source`**（`vector` | `profile`），与 `evolution` 一致；前端 `roleStore` 与调试面板「性格向量」在 **profile** 下显示视图说明 HelpHint。
- **Remote 插件**：`EventEstimator::estimate` 与 `event.estimate` 的 `params` 增加 **`personality_source`**；`prompt.build_prompt` 的 `params` 在完整 `role` 之外另含顶层 **`personality_source`**（与 `role.evolution_config` 一致）。
- **主界面**：`RoleRuntimePanel` 展示当前 **人格来源**（vector / 档案）与 HelpHint，与调试面板文案对齐。

### Documentation

- **creator-docs-en**：新增 **`FAQ.md`**、**`COMPATIBILITY.md`**；**`plugin-and-architecture/`** 下补齐 **`REMOTE_PLUGIN_PROTOCOL`**、**`DIRECTORY_PLUGINS`**、**`BRIDGE_API_REFERENCE`**、**`EXTENSION_POINTS`**、**`CREATOR_PLUGIN_ARCHITECTURE`** 英文全文（与 `creator-docs/` 中文对拍）。**`DOCUMENTATION_INDEX`（英）**、**`PROJECT_STATUS_AND_ALIGNMENT`（英）**、**`creator-docs-en/README.md`**、**`creator-docs/README.md`** 与 **`getting-started/DOCUMENTATION_INDEX.md`（中）** 已互链更新。
- **creator-docs-en（续）**：新增 **`LICENSE_POLICY.md`**、**`guides/CONFIGURATION_FILES.md`**、**`guides/REGRESSION_COMPLEX_EMOTION_QA.md`**、**`guides/MUMU_UI_ACCEPTANCE_CHECKLIST.md`**、**`plugin-and-architecture/LOCAL_PLUGIN_BRIDGE_SPEC.md`**、**`plugin-and-architecture/HOW_TO_REPLACE_MODULES.md`**；英 **`DOCUMENTATION_INDEX`** 快速入口与「建议阅读顺序」中插件类链接统一为 **`creator-docs-en/plugin-and-architecture/...`**，**mumu** 与 **Extension points** 改 **`../guides/...`** / **`../plugin-and-architecture/...`**；**`PROJECT_STATUS_AND_ALIGNMENT`（英）** 用户手册表指向 **`../guides/CONFIGURATION_FILES.md`**、**`../LICENSE_POLICY.md`**、**mumu** 英稿，插件表补充 **`HOW_TO_REPLACE_MODULES`**；**`EXTENSION_POINTS` / `CREATOR_PLUGIN_ARCHITECTURE`（英）** 内 **`HOW_TO` / `LOCAL`** 指同目录英文稿。
- **创作者文档双语收尾**：**`creator-docs-en/README.md`** 增加 **Documentation bilingual closure baseline**（权威语料、已镜像范围、中文-only 长尾、随开发更新的纪律）；**`creator-docs/README.md`**、**`getting-started/DOCUMENTATION_INDEX.md`（中）**、**`PROJECT_STATUS_AND_ALIGNMENT`（中/英）**、**`PROJECT_CURRENT_STATUS`（中/英）**、**`handoff/I18N_FOUR_REPO_BASELINE.md`**、**`PRODUCT_AND_KERNEL_GAP_CHECKLIST.md` §A6** 与之互参（§A6 中 **creator-docs-en** 主干项标为已满足；**界面无残留中文**仍为未勾选）。
- **产品线落实（细碎 + 中小块）**：新增 **[handoff/PRODUCT_RELEASE_CHECKLIST.md](handoff/PRODUCT_RELEASE_CHECKLIST.md)**（P0 发版勾选）、**[handoff/PLUGIN_HIGH_RISK_ACCEPTANCE.md](handoff/PLUGIN_HIGH_RISK_ACCEPTANCE.md)**（高风险能力演示表）；根 **README / README.en** 增加获取帮助、早期采用者与「三问」；**SECURITY** 增加分发与隐私速览；**bug_report** 模板链 FAQ/索引/ERROR_CODES；**CONTRIBUTING（中/英）** 增加 Breaking 流程与 **`check:release` 不含 `test:unit`** 说明；**VISION / BACKLOG**、**DOCUMENTATION_INDEX**、**COMPATIBILITY**、**ERROR_CODES（中/英）**、**PRODUCT_LINE_TASK_BUCKETS**、**PRODUCT_AND_KERNEL_GAP_CHECKLIST §C1** 互链或内容更新。
- **产品线阶段切换（文档）**：**[PRODUCT_LINE_TASK_BUCKETS.md](handoff/PRODUCT_LINE_TASK_BUCKETS.md)**、**[PRODUCT_RELEASE_CHECKLIST.md](handoff/PRODUCT_RELEASE_CHECKLIST.md)**、**[PRODUCT_AND_KERNEL_GAP_CHECKLIST.md](handoff/PRODUCT_AND_KERNEL_GAP_CHECKLIST.md) §D**、**[PROJECT_STATUS_AND_ALIGNMENT.md](creator-docs/getting-started/PROJECT_STATUS_AND_ALIGNMENT.md)**（中/英）、**[DOCUMENTATION_INDEX.md](creator-docs/getting-started/DOCUMENTATION_INDEX.md)** 标明 **批次一已完成**，**下一默认焦点为 §四硬骨头**（逐项 issue）。
- **PLUGIN_V1 / Remote 协议**：[PLUGIN_V1.md](creator-docs/plugin-and-architecture/PLUGIN_V1.md) 补充 `RoleInfo` / `RoleData`、HTTP `/chat` 与 `prompt.build_prompt` 的 **`personality_source`**；[REMOTE_PLUGIN_PROTOCOL.md](creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md) 新增 §3.4 与 `event.estimate` 参数表行。
- **性格档案设计轴心**：重写 **[docs/personality-archive-notes.md](docs/personality-archive-notes.md)**（核心/可变档案、`personality_source`、七维视图、三应用分工）；新增 **[docs/design-axis-evolution.md](docs/design-axis-evolution.md)** 记录思路变化（旧 handoff 不删）；根 README、`creator-docs` 索引与入门文档、`roles/README_MANIFEST.md` §二 §5.3、`PACK_VERSIONING.md`、`CREATOR_ROLE_PACK_CUSTOMIZATION.md`、`CREATOR_SCENE_GUIDE.md`、`CREATOR_PLUGIN_ARCHITECTURE.md` 等与之一致并互链；**`roles/settings.template.json`** 的 `evolution` 显式包含 **`personality_source`**。
- 新增 **[creator-docs/getting-started/SIDECAR_LLM_USER_GUIDE.md](creator-docs/getting-started/SIDECAR_LLM_USER_GUIDE.md)**：本机侧车 + 用户自带 Key（BYOK）接闭源云端模型的用户向步骤；与 `REMOTE_PLUGIN_PROTOCOL`、`examples/remote_plugin_minimal`、启动器注入环境变量互链；文档索引与根 README 已链入。
- 新增 **[examples/remote_plugin_openai_compat/](examples/remote_plugin_openai_compat/)**：OpenAI 兼容 `chat/completions` 侧车示例（`requests` + `.env.example`）；`SIDECAR_LLM_USER_GUIDE` 与文档索引已链入。
- **示例重构**：抽出 **[examples/common/](examples/common/)**（`jsonrpc_http.py`、`oclive_stub_handlers.py`），最小侧车与 OpenAI 范例共用 JSON-RPC 与非 LLM 占位，降低重复职责。
- 新增 **[creator-docs/roadmap/PLUGIN_WEB_SECTION.md](creator-docs/roadmap/PLUGIN_WEB_SECTION.md)**：社区站 **插件区** 路由建议、`plugins.json` 清单字段、与 Remote 协议文档的衔接、与角色包板块边界。  
- 新增 **[creator-docs/roadmap/COMMUNITY_WEB_VISION.md](creator-docs/roadmap/COMMUNITY_WEB_VISION.md)**：网页社区站愿景——**论坛（贴吧式）/ 角色包（C 站式）/ 插件区** 三板块、聊天记录与评论边界、Discord 取舍建议；与 MARKET 文档互链；补充 **是否新仓** 与 **技术栈分层（前端 / 论坛产品 / BaaS 或自研 API / 对象存储）**。  
- 新增 **[creator-docs/roadmap/MARKET_LAUNCHER_INTEGRATION.md](creator-docs/roadmap/MARKET_LAUNCHER_INTEGRATION.md)**：角色包 / 插件市场与启动器联动（发版同发、静态索引、阶段 A/B/C、CI 绑定思路）；文档索引、根 README、`BACKLOG_EXPERIENCE_AND_ECOSYSTEM` 第三节与 **oclive-launcher** README 已链入；补充 **社区发现型站点（类比 Civitai 体验）** 与 oclive 角色包字段的对应表及 UGC 隐含成本说明。
- 新增 **[creator-docs/roadmap/SOMEDAY_TOOLCHAIN_CI.md](creator-docs/roadmap/SOMEDAY_TOOLCHAIN_CI.md)**：后日待办（工具链 / CI 性价比备忘，非阻塞）；文档索引与根 README 已链入。
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
