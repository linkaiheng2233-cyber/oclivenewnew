# Agent / AI 协作说明（A.I.Live · oclivenewnew）

本仓库为 **A.I.Live** 桌面角色对话应用（**Tauri + Vue 3 + Rust**；工程代号 **oclive**）。自动化助手或外部 Agent 在修改代码前，请先阅读：

- **命名与 canonical import SSOT**：[creator-docs/NAMING_CONVENTIONS.md](creator-docs/NAMING_CONVENTIONS.md)（DTO → `oclive_kernel_types`；trait → `oclive_kernel_contracts`；编排 → `oclive_kernel_host`）

### 发版版本（`main`，2026-06-07）

| 产物 | 版本 | 位置 |
|------|------|------|
| **桌面宿主** | **0.3.0** | 根 `package.json`、`src-tauri/Cargo.toml`、`src-tauri/tauri.conf.json` |
| **VS Code 扩展** | **0.3.0** | 姊妹仓 `oclive-vscode/package.json` |
| **`oclive-cli`** | **0.1.0** | `crates/oclive-cli/Cargo.toml` |
| **`oclive_kernel_runtime`** | **0.2.0** | `crates/oclive_kernel_runtime/Cargo.toml` |
| **`oclive_validation`** | **0.1.0** | `crates/oclive_validation/Cargo.toml` |

独立 SemVer 策略见 [`creator-docs/development/RELEASE_VERSIONING.md`](creator-docs/development/RELEASE_VERSIONING.md)；用户可见变更见 [`CHANGELOG.md`](CHANGELOG.md) **`[0.3.0]`**。

- **跨平台**：[`docs/DEV_CROSS_PLATFORM.md`](docs/DEV_CROSS_PLATFORM.md)。
- **Rust Release / workspace 依赖**：[`handoff/RUST_RELEASE_AND_DEPENDENCIES.md`](handoff/RUST_RELEASE_AND_DEPENDENCIES.md)。
- **性能与包体**：阶段总表 [`handoff/PERF_PHASES.md`](handoff/PERF_PHASES.md)（v0.2 P1–P3 已收尾）；[`handoff/PERFORMANCE_BASELINE_ACCEPTANCE.md`](handoff/PERFORMANCE_BASELINE_ACCEPTANCE.md)、[`handoff/FRONTEND_CHUNK_OPTIMIZATION.md`](handoff/FRONTEND_CHUNK_OPTIMIZATION.md)、[`handoff/BUNDLE_RESOURCES_SIZING.md`](handoff/BUNDLE_RESOURCES_SIZING.md)。
- **项目约束**：根目录 [`.cursor/rules/oclivenewnew.mdc`](.cursor/rules/oclivenewnew.mdc)（编排、持久化、Tauri 命令注册、DTO、Prompt 约定）。
- **创作者与架构文档**：[`creator-docs/README.md`](creator-docs/README.md) → [`creator-docs/getting-started/DOCUMENTATION_INDEX.md`](creator-docs/getting-started/DOCUMENTATION_INDEX.md)。
- **愿景与路线**：[`creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md`](creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md)、[`creator-docs/roadmap/VISION_OPEN_LAB.md`](creator-docs/roadmap/VISION_OPEN_LAB.md)（开放实验场摘要）。

### 工程纪律（C2）

- **Breaking 变更流程**（识别、审阅、兼容层、`oclive_validation` 与契约文档同步、PR/迁移模板）：[`handoff/BREAKING_CHANGE_PROCESS.md`](handoff/BREAKING_CHANGE_PROCESS.md)。贡献者摘要见 [`CONTRIBUTING.md`](CONTRIBUTING.md)「破坏性变更」。
- **关键路径交接（Bus factor）**（编排入口、`PluginHost`、错误码、迁移、OOCP/CI 定位）：[`handoff/BUS_FACTOR_NOTES.md`](handoff/BUS_FACTOR_NOTES.md)。索引入口见 [`creator-docs/getting-started/DOCUMENTATION_INDEX.md`](creator-docs/getting-started/DOCUMENTATION_INDEX.md)「工程纪律」。

### 脚手架（`oclive-cli`）

- **crate**：[`crates/oclive-cli/`](crates/oclive-cli/)（workspace 成员）；`cargo run -p oclive-cli -- init` 交互或 `--non-interactive --preset` 生成**可独立 `cargo build`** 的最小内核/库骨架（当前占位依赖 `serde`/`serde_json`，便于硬件与无头场景先统一目录与 `settings.json` 形状）。
- **文档**：[OCLIVE_CLI_GUIDE.md](creator-docs/cli/OCLIVE_CLI_GUIDE.md) · [SETTINGS_REFERENCE.md](creator-docs/cli/SETTINGS_REFERENCE.md)（`plugin_backends` 与预设矩阵）；接入真实 `oclive_kernel_runtime` / `oclive_kernel_server` 时在生成 `Cargo.toml` 中改为 path 依赖并替换入口代码。
- **GitHub 插件索引缓存**：SSOT 为 [`data/plugins.json`](data/plugins.json)。桌面写入 `{app_data}/plugin_index_cache.json`；CLI 为 `~/.oclive/plugin_index_cache.json`（Windows：`%USERPROFILE%\.oclive\`）。离线或索引未 push 时可 `Copy-Item data/plugins.json` 到上述路径；GitHub 不可达时安装可设 **`OCLIVE_LOCAL_MONOREPO`** 指向本仓根目录。见 [handoff/GITHUB_PLUGIN_INDEX_LINE.md](handoff/GITHUB_PLUGIN_INDEX_LINE.md)。

### 架构 RFC

- **运行时双核双态（Stable / Experimental，Opt-in Beta，默认关）**：[RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md](creator-docs/rfc/RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md) · **Cursor 对齐进度** [handoff/DUAL_CORE_CURSOR_HANDOFF.md](handoff/DUAL_CORE_CURSOR_HANDOFF.md) · 速查 [handoff/DUAL_CORE_ALIGNMENT.md](handoff/DUAL_CORE_ALIGNMENT.md)（与 Monolith **构建态** 正交；**不阻塞**当前 v2 交付）。
- **高耦合编译模式（Monolith）**：[RFC_OCLIVE_MONOLITH_MODE.md](creator-docs/rfc/RFC_OCLIVE_MONOLITH_MODE.md)（路线图见 RFC §9，已与 `oclive-cli` 实现对齐）。**`oclive-cli`**：`init --monolith` 或交互「开发者编译选项」生成 **`monolith.toml`**、`vendor/oclive_monolith_builtin/`（**七焊接键焊接桩唯一模板源**）、**`process_message_monolith.rs`**、双 **`[[bin]]`**（`main.rs` / `main_monolith.rs`）；**`cargo run -p oclive-cli -- build|bench`** 再生成与对比；**`bench --save` / `--compare`** 用于本地性能历史与对比（见 [OCLIVE_CLI_GUIDE.md](creator-docs/cli/OCLIVE_CLI_GUIDE.md)）。**`oclive dev`**：监听脚手架或内核项目下 **`roles/`** 中 `manifest.json` / `settings.json` 变更，便于热重载脚本对接。

### 内核架构（主应用 `src-tauri`）

- **Crate 速查**：[`crates/README.md`](crates/README.md)（依赖图、改哪个 crate、canonical import）
- **架构总述（对外）**：契约型薄核 + **单核双态**；**第 1–6 模块** / **第 N 设施子模块** / **后端模块插件模块** — [`creator-docs/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md`](creator-docs/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md)。
- **主编排入口**：Tauri IPC 与 **`--api` HTTP** 均在 **`crates/oclive_kernel_host/src/domain/chat_engine/process_message.rs`** 的 **`process_message`**（经 `chat_engine/mod.rs` re-export；及 `co_present` / `scene` 等子模块）内顺序编排（`oclivenewnew-tauri` 经 `lib.rs` re-export）。角色包 **v2** 以 **蓝图文件 `pipeline.ocblueprint`** 为磁盘 SSOT；**不以**蓝图 `steps[]` 作首轮调度 DSL。运行时行为以本仓库 `process_message` 为准。
- **角色包与蓝图边界**：**角色包** = 身份、人格、关系、**`prompts/`**、**`reply_quality_anchor`**（初级创作者）。**蓝图** = **`slot_registry`**、**`groups`**、后端/模型/交互模式/记忆策略、**`runtime_config.dual_core`**（管理员；默认关）。逻辑分责见 **[handoff/ROLE_PACK_BOUNDARY.md](handoff/ROLE_PACK_BOUNDARY.md)** · [ROLE_PACK_SPEC.md](creator-docs/role-pack/ROLE_PACK_SPEC.md) §0 · [SETTINGS_REFERENCE.md](creator-docs/cli/SETTINGS_REFERENCE.md) §零。勿让 Agent 在「角色」任务中改 `slot_registry`。
- **错误与日志**：统一错误类型见 **`src-tauri/src/lib.rs`** 内联 `error` 模块（re-export `oclive_kernel_types::error`）；Tauri 命令层见 **`src-tauri/src/api/error.rs`**（`ApiError` / `CommandError`）；**机器 `code` 与 JSON 体**以 **`oclive_kernel_types::KernelErrorBody`** 与 **`creator-docs/getting-started/KERNEL_ERROR_CODE_CONVENTION.md`** 为准（与 `AppError::code()`、`http_chat_codes`、目录插件 **`ApiError` JSON** 对齐；**Sentry / 用户可见错误扫尾**见 **`handoff/A3_CLOSURE_SUMMARY.md`** / **`handoff/A3_CLOSURE_SUMMARY.en.md`**）。结构化日志为 **`tracing`**：`init_tracing()` / `init_tracing_with_log_dir()`（`lib.rs`）默认 `info`，受 **`RUST_LOG`** 控制；设置 **`OCLIVE_LOG_DIR`** 或 **`--api`** 模式（`main.rs` → `temp/oclive_api_app_data/logs/`）可同时写入 rolling 文件；**`RUST_LOG` 含 `json`** 时 stdout/文件使用 JSON 行格式。
- **启动健康检查**：首轮对话前 **`startup_health::ensure_once`**（槽位、`plugin_backends`、角色包文件、**`DbManager::health_ping`**、可选 LLM 探测）；环境变量 **`OCLIVE_SKIP_STARTUP_HEALTH`** / **`OCLIVE_SKIP_LLM_STARTUP_PROBE`** 可跳过。实现：**`crates/oclive_kernel_host/src/domain/startup_health.rs`**。
- **实验性双核运行时（feature）**：`oclivenewnew-tauri` 的 Cargo feature **`dual_core`**（**默认关闭**）。未启用时 `dual_pipeline*` 不参与编译，`role.dual_core_gated()` 走常规 `CoPresent` 路径。本地实验：`cargo build -p oclivenewnew-tauri --features dual_core`。见 [`handoff/DUAL_CORE_CURSOR_HANDOFF.md`](handoff/DUAL_CORE_CURSOR_HANDOFF.md)。
- **多发行版单写者（Phase 2）**：桌面与 VS Code **平等**——`GET :8420/health` attach 优先，否则 spawn `oclive-kernel-server`；数据目录 **`OCLIVE_APP_DATA`** → `%LOCALAPPDATA%/OCLive/data`。桌面 **`kernel_lifecycle/`** + **`kernel_attach`** 为 HTTP 薄客户端，**不**内嵌 `api_router` 写库。无头 HTTP 入口 crate：**[`crates/oclive_kernel_host/`](crates/oclive_kernel_host/)**（`init_tracing` / `run_api_server` / `http_api` re-export）；**`oclive-kernel-server`** 依赖该 crate 而非直接依赖 `oclivenewnew-tauri`。规范：[`creator-docs/kernel/DISTRO_KERNEL_LIFECYCLE.md`](creator-docs/kernel/DISTRO_KERNEL_LIFECYCLE.md) · [`OCLIVE_APP_DATA.md`](creator-docs/kernel/OCLIVE_APP_DATA.md) · [`CROSS_HOST_MEMORY.md`](creator-docs/role-pack/CROSS_HOST_MEMORY.md)。
- **内核自举与发行版适配（P1–P4）**：各发行版可在安装根提供 **`distro.oclive.toml`**（契约 [`DISTRO_CAPABILITY_PROFILE.md`](creator-docs/kernel/DISTRO_CAPABILITY_PROFILE.md) · 示例 `examples/distro-profiles/`）。**P2a** `KernelBinaryManifest` + sidecar + `GET /health` 的 `kernel_manifest` + `oclive-kernel-server --version-json`。**P3a** `promote_with_backup` / `rollback_shared_kernel`（`crates/oclive_kernel_runtime/src/kernel_runtime_ops.rs`）+ `cargo run -p oclive-cli -- kernel status|promote|rollback`；桌面 `ensure`/`reconnect` 经 `apply_promote_to_candidate`。**P4** `HostProfile`（`crates/oclive_kernel_host/src/domain/host_profile.rs`）：`OCLIVE_DISTRO_ID` / `OCLIVE_DISTRO_PROFILE` 加载 profile，合并 `plugin_backends` 上限，按 `host_flags` 跳过 Agent / 复杂情感，简洁 Prompt overlay；spawn 子进程时传递 distro 环境变量。延后：**P2b** 多发行版差异化 manifest 字段；**P3b** 内核进程内自升级（当前由宿主协调 promote）。

### 测试体系（三层归属）

- **协议层 → 本仓**：**OOCP HTTP 黑盒（S0–S12，共 13 场景；可选 S13/S14）** 已入库且 **CI 已集成**——场景与 CI 说明见 [`creator-docs/testing/OOCP_TEST_SUITE.md`](creator-docs/testing/OOCP_TEST_SUITE.md)；可执行脚本在 [`examples/oocp-test-suite/`](examples/oocp-test-suite/)（`node run.mjs`）。CI **`.github/workflows/ci.yml`** 的 **`oocp-test-suite`** job（Ubuntu）会 `cargo build -p oclivenewnew-tauri --features dual_core`、拉起 **`oclivenewnew-tauri --api`**（默认 **`OCLIVE_HTTP_API_MOCK_LLM=1`**）、轮询 **`GET /health`** 后执行 **`node run.mjs --include-dual-core`**（S13/S14），再执行根目录 **`scripts/e2e-core-api-restart.mjs`**（**进程重启后再对话** 烟测，A1.1a）。**Ubuntu `frontend`** job 在 **`npm run build`** 后另跑 **Playwright + `vite preview` 首屏**（A1.1b；Windows `frontend` 不跑 Playwright）。另含 **`src-tauri`** 下 **`cargo test`**、`tests/` 集成测与 HTTP 路由单测等。
- **`invoke` 热路径集成（A1.2）**：矩阵 [`handoff/INVOKE_HOTPATH_MATRIX.md`](handoff/INVOKE_HOTPATH_MATRIX.md)，集成测 [`src-tauri/tests/invoke_hotpath_matrix.rs`](src-tauri/tests/invoke_hotpath_matrix.rs)（**9** 条 `*_impl`；`cargo test -p oclivenewnew-tauri --test invoke_hotpath_matrix`）。
- **组件层 → oclive-pack-editor**：编写器 UI、Vitest、Playwright E2E 等（不在本仓重复维护用例树）。
- **插件层 → oclive-pack-editor**：目录插件范式、**`official-vue-test-runner`** 等；主仓不复制该树。
- **主仓前端最小烟测**：根目录 **`npm run test:unit`**（Vitest，`src/smoke.test.ts`）；**Playwright + `vite preview`**（**`npm run test:e2e:preview`**，**CI 仅 Ubuntu `frontend`**，见 CONTRIBUTING）。
- **总览**：[creator-docs/testing/OVERVIEW.md](creator-docs/testing/OVERVIEW.md)。

### 供应链与安全审计

- **当前状态**：**已知漏洞跟踪中**；**不宣称零漏洞**。摘要执行日期与命中条数见 [creator-docs/development/LIGHTWEIGHT_PROFILE.md](creator-docs/development/LIGHTWEIGHT_PROFILE.md) §6.4；**漏洞级清单与升级路线**见 [creator-docs/security/KNOWN_VULNERABILITIES.md](creator-docs/security/KNOWN_VULNERABILITIES.md)；**审查边界**见 [creator-docs/security/SECURITY_AUDIT_SCOPE.md](creator-docs/security/SECURITY_AUDIT_SCOPE.md)。
- **CI**：**`cargo-audit`** job（**cargo-audit 0.22.1**）为 **`continue-on-error: true`**，用于可见性；待依赖升级后可改为失败即红。

### 第 1 设施子模块 — 复杂情感设施子模块（`narrative_hint` · 共景 → 下一轮 Prompt）

### 用户身份 & 回复后处理（v0.3 · 非六槽）

- **User Identity Prompt Template**：角色包 `user_identities/`（`index.json` + `*.md`）；编排 **`resolve_active_user_identity`** → **`PromptBuilder.push_user_identity_section`**（`turn_pipeline/pre`，LLM 之前）。Tauri / HTTP：`get_user_identity_state`、`set_user_identity`、`POST /user_identity/set` 等。
- **Reply Post-Processor**：角色包 **`config.json` → `reply_post_processor`**（**默认 `enabled: false`**）；编排 **`resolve_reply_post_processor`** → **`process_reply`**（内置 `post_llm` 之后）。backend：`builtin` / `remote` / `directory`；发行版 `[post_process].chain` 合并见 `host_profile.rs`。
- **禁止**：将二者写入 **`slot_registry`**、蓝图 `runtime_config` 六槽键，或 Experimental 核 step（正交能力，见 [RFC](../creator-docs/rfc/RFC_USER_IDENTITY_AND_REPLY_POST_PROCESSOR.md)）。
- **文档**：[ROLE_PACK_SPEC §1.1 / §9.7](creator-docs/role-pack/ROLE_PACK_SPEC.md) · [OCLIVE_ARCHITECTURE_OVERVIEW](creator-docs/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md)「正交能力单元」· handoff [USER_IDENTITY_REPLY_POST_PROCESSOR_PHASE2.md](handoff/USER_IDENTITY_REPLY_POST_PROCESSOR_PHASE2.md)。

### 第 2 设施子模块 — 专家模型设施子模块（专家路由 · `expert_routing.json` · `slot.expert.invoke`）

编号与分层见 [`creator-docs/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md`](creator-docs/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md)（**不是**第 1–6 后端模块；**不是**后端模块插件模块）。

- **类型与内置规则**：[`crates/oclive_kernel_runtime/src/domain/complex_emotion.rs`](crates/oclive_kernel_runtime/src/domain/complex_emotion.rs)（`ComplexEmotionInput` / `ComplexEmotionOutput`、`BuiltinKeywordComplexEmotionProvider::resolve_turn_inner`）；可选 Remote 见 [`src-tauri/src/infrastructure/remote_plugin/complex_emotion_http.rs`](src-tauri/src/infrastructure/remote_plugin/complex_emotion_http.rs)。
- **主路径 wiring**：[`crates/oclive_kernel_host/src/domain/chat_engine/turn_pipeline/`](crates/oclive_kernel_host/src/domain/chat_engine/turn_pipeline/) 在 `load_recent_context` 之后、**`build_prompt` 之前**解析本回合复杂情感；上一轮 `narrative_hint` 经 **`SessionCache`** / DB（`complex_emotion_hint` 表）按 `srid` 读取；通过 **`PromptInput::previous_complex_emotion_narrative_hint`** 传入 [`PromptBuilder::build_prompt`](crates/oclive_kernel_runtime/src/domain/prompt_builder.rs)（定义在 `oclive_kernel_runtime`，经 `oclive_kernel_host::domain` re-export；段落标题为「复杂情感叙事提示」）。
- **集成测试**：[`src-tauri/tests/narrative_hint_prompt_roundtrip.rs`](src-tauri/tests/narrative_hint_prompt_roundtrip.rs)。

### 聊天记录混合存储（SQLite 真源 + JSON 镜像 · phase 1–3 架构完整）

- **架构**：[`handoff/CHAT_STORAGE_ARCHITECTURE.md`](handoff/CHAT_STORAGE_ARCHITECTURE.md) · 创作者选型 [`creator-docs/storage/STORAGE_BACKEND_GUIDE.md`](creator-docs/storage/STORAGE_BACKEND_GUIDE.md) — `chat_sessions` / `chat_messages` 与 `short_term_memory` / `long_term_memory` **完全解耦**；删聊天记录**不**清记忆表。
- **聊天存储（phase 3）**：运行时始终构造 **`HybridConversationStore`**（SQLite 真源 + 可选 JSON 镜像）；`OCLIVE_CHAT_STORAGE_BACKEND` / `config.json` → `chat_storage.backend` 的 **`file`** / **`sqlite`** 仅影响 **`resolve_mirror_enabled`**（镜像开关），不切换独立 `file_store` / `sqlite_store` 实现。脚手架 `oclive-cli init` 交互可选 **`location`**（`global` / `role_pack`）。
- **能力探测（PATCH-1）**：`get_chat_storage_capabilities` 返回 `backend_kind` 与 `supports_search` / `supports_replay` / `supports_cleanup`；前端存储管理按后端 **隐藏不可用操作**（file 无自动清理）。
- **记忆回放（phase 3）**：`replay_memory_extraction` / `get_replay_progress` — 从聊天记录**合并**重提取 AI 记忆（**不覆盖**已有 `long_term_memory`；阈值可配 `chat_storage.replay_similarity_threshold`，默认 0.6）。设置 → 存储管理 UI 可触发。
- **实现**：[`src-tauri/src/infrastructure/chat_storage/`](src-tauri/src/infrastructure/chat_storage/) · `AppState::conversation_store` · CoPresent `post_llm` 写入并回填 `SendMessageResponse` 消息 id/时间戳 · 角色包 `config.json` → `chat_storage`（`backend`、`location`、`max_messages_per_session`、`auto_cleanup_*`、`replay_similarity_threshold`）。
- **前端**：[`src/stores/chatStore.ts`](src/stores/chatStore.ts) 从 `fetch_chat_messages` 加载；IndexedDB 仅遗留迁移；设置 → **存储管理**（[`ChatStorageSettingsPanel.vue`](src/components/settings/ChatStorageSettingsPanel.vue)）显示当前后端名称，支持搜索、导出、自动清理（按能力）、单条删改、记忆回放。
- **Tauri**：`list_chat_sessions` / `fetch_chat_messages` / … / `run_chat_auto_cleanup` / **`replay_memory_extraction`** / **`get_replay_progress`** / **`get_chat_storage_capabilities`**（完整表见架构文档）。
- **助手勿**：让 `MemoryEngine` / 归档 LLM 读取 `{app_data}/chats/` 或 `chat_messages` 充当记忆真源；编排上下文仍走 `short_term_memory` / `long_term_memory`。

**契约优先**：角色包 `manifest.json` / `settings.json` 键与行为以 `roles/README_MANIFEST.md`、`RoleStorage::load_role` 及校验 crate 为准；新增顶层键需同步 `crates/oclive_validation` 与文档。

**姊妹仓库**（同级目录常见）：`oclive-pack-editor`（角色包编写器）、`oclive-launcher`（启动器）、`oclive-plugin-market`（市场站）。各仓可有各自的 `AGENTS.md`，指向本仓文档索引即可。

**演示视频（Remotion）**：独立仓库 **`oclive-remotion-demo`**（与主应用同级目录常见）。所有 `npm run preview` / `render:*` / `capture:validate` **须在该仓库根目录执行**，勿在主仓 `oclivenewnew` 根目录运行（会报 `Missing script`）。使用说明见该仓库根目录 **`README.md`**（本地常与主仓并列，例如 `D:\oclive-remotion-demo`）。

**开发机磁盘**：本仓库根目录 [`.cargo/config.toml`](.cargo/config.toml) 将 **Cargo `target-dir`** 指到仓库外的 `../oclive-dev-artifacts/oclivenewnew-cargo-target/`，与源码分离；发版安装包体积与此无关。姊妹仓 **oclive-pack-editor**、**oclive-launcher** 使用同级目录下的 `oclive-pack-editor-cargo-target/`、`oclive-launcher-cargo-target/`（各仓自有 `.cargo/config.toml`）。旧版留在仓库内的 `target/`、`src-tauri/target/` 可整夹删除。

### 前端：插件管理入口与 Tauri `invoke`

- **插件与模型入口**：**Ctrl+Shift+F** 打开极简已安装列表（[`SimplePluginManagerPanel`](src/views/SimplePluginManagerPanel.vue)）；**Ctrl+Shift+M** 打开模型管理（[`ModelManagerPanel`](src/views/ModelManagerPanel.vue) → [`ModelManagerBody`](src/components/model/ModelManagerBody.vue)，本会话 LLM 后端与 Ollama 探测）；顶栏「更多」另有插件市场。逻辑见 [`usePluginManagerWindow.ts`](src/composables/usePluginManagerWindow.ts)、[`useModelManagerWindow.ts`](src/composables/useModelManagerWindow.ts)。文案见 i18n `app.more.*`、`modelManager.*`。
- **架构图专业面板（代码保留、默认不挂载）**：[`PluginManagerPanel`](src/components/plugin-manager/PluginManagerPanel.vue) 仍可供开发/CLI 场景复用，主应用 `App.vue` 不再默认挂载。
- **`invoke` 参数名**：Tauri 将 Rust 命令的 `snake_case` 形参映射为前端的 **camelCase** 键（如 `plugin_id` → `pluginId`）。[`src/api/`](src/api/) 封装（如 `get_plugin_logs`、`spawn_plugin_for_test`）须与之一致；若命令仍手写 `snake_case` 载荷，会出现「missing required key `pluginId`」类错误。

### Agent / Skill（最小闭环）

- **agent 后端模块**（产品亦称扩展槽）：`plugin_backends.agent`（`builtin` / `remote` / `directory`）；会话覆盖与来源快照包含 `agent`。（`none` 语义见 `creator-docs/kernel/MODULE_NONE_SEMANTICS.md` §7（若存在）。）
- **后端骨架**：
  - [`crates/oclive_kernel_host/src/domain/agent.rs`](crates/oclive_kernel_host/src/domain/agent.rs)：`AgentProvider` trait 与 `BuiltinReActAgent`。
  - [`src-tauri/src/infrastructure/mcp_client.rs`](src-tauri/src/infrastructure/mcp_client.rs)：扫描 `{app_data}/mcp-servers/*.json`、列出 server、调用工具（http/stdio）。
  - [`src-tauri/src/api/agent.rs`](src-tauri/src/api/agent.rs)：`list_mcp_servers` / `call_mcp_tool` / `get_agent_debug_traces` / `clear_agent_debug_traces`。
- **调试 UI**：Agent 调试可经目录插件 [`examples/directory-plugin-minimal/`](examples/directory-plugin-minimal/) 或后续专用入口接入（主应用已移除独立 `AgentDebugPanel` 面板）。
- **示例 Skill / MCP**：MCP server 接入形状见 [`src-tauri/src/infrastructure/mcp_client.rs`](src-tauri/src/infrastructure/mcp_client.rs) 与运行期 `{app_data}/mcp-servers/*.json`；在库可参考的最小 RPC server 示例为 [`examples/directory-plugin-minimal/`](examples/directory-plugin-minimal/) 与 [`examples/common/jsonrpc_http.py`](examples/common/jsonrpc_http.py)。（`examples/weather_skill/`（`get_weather(city)` 最小 MCP server）**尚未入库，为计划中的示例**。）

### Agent / Skill 通用接入标准（v1）

- **MCP 配置目录**：`{app_data}/mcp-servers/*.json`，支持 `transport=http|stdio`、`timeout_ms`、`tools` 预声明；运行时可 `list_mcp_servers`、`list_mcp_tools`、`call_mcp_tool`。
- **Function Calling**：后端统一走 [`src-tauri/src/infrastructure/function_call_parser.rs`](src-tauri/src/infrastructure/function_call_parser.rs)：
  - `parse_from_llm_response` 解析 `tool_calls[]` 与 `function_call` 两种主流输出；
  - `to_function_calling_schema` 将 MCP tool 列表转为函数 schema。
- **Agent 路由**：`plugin_backends.agent` 为六宿主槽之一，与 memory/emotion 等保持同样的包默认 / 会话覆盖 / 来源快照语义。

## 内核约束 - 权限弹窗

- **Directory 插件**：首次启用高风险能力（如 `process:spawn`、`network:*` 出站）前，必须经过用户确认授予；未授予则必须降级且有可见提示/审计。
- **MCP servers**：任何 `transport=stdio` 的 server 必须显式授权（等同 `process:spawn`）；`transport=http` 必须显式授权（`network:*`）。未授权不得调用。
- **Remote env providers**：检测到 env 配置不等于启用；必须先授予 `network:*`，否则 provider 只能降级为 placeholder 并提示。

### 创作者工具链（v1）

- **脚手架**：`create_plugin_scaffold`（Tauri 命令，见 [`src-tauri/src/api/plugin_scaffold.rs`](src-tauri/src/api/plugin_scaffold.rs)）生成 `manifest.json` + 语言模板 + README，并打开目标目录。
- **一键打包**：`pack_plugin` 校验 manifest 后输出 `.oclive-plugin` 与 `*.signature.json`（SHA-256）。
- **调试体验**：
  - `EnvVarManager.vue` 管理 `OCLIVE_*` 会话草稿并复制 PowerShell 设置命令；
  - 目录插件 `shell.bridge.invoke` 经 [`plugin_bridge_invoke`](src-tauri/src/api/plugin_bridge.rs) 调用宿主命令（attach 模式下 `send_message` 等与主 UI 一致走 kernel HTTP）。
