# Agent / AI 协作说明（oclivenewnew）

本仓库为 **Tauri + Vue 3 + Rust** 桌面角色对话应用。自动化助手或外部 Agent 在修改代码前，请先阅读：

- **跨平台**：[`docs/DEV_CROSS_PLATFORM.md`](docs/DEV_CROSS_PLATFORM.md)。
- **Rust Release / workspace 依赖**：[`handoff/RUST_RELEASE_AND_DEPENDENCIES.md`](handoff/RUST_RELEASE_AND_DEPENDENCIES.md)。
- **性能与包体**：阶段总表 [`handoff/PERF_PHASES.md`](handoff/PERF_PHASES.md)（v0.2 P1–P3 已收尾）；[`handoff/PERFORMANCE_BASELINE_ACCEPTANCE.md`](handoff/PERFORMANCE_BASELINE_ACCEPTANCE.md)、[`handoff/FRONTEND_CHUNK_OPTIMIZATION.md`](handoff/FRONTEND_CHUNK_OPTIMIZATION.md)、[`handoff/BUNDLE_RESOURCES_SIZING.md`](handoff/BUNDLE_RESOURCES_SIZING.md)。
- **项目约束**：根目录 [`.cursor/rules/oclivenewnew.mdc`](.cursor/rules/oclivenewnew.mdc)（编排、持久化、Tauri 命令注册、DTO、Prompt 约定）。
- **创作者与架构文档**：[`creator-docs/README.md`](creator-docs/README.md) → [`creator-docs/getting-started/DOCUMENTATION_INDEX.md`](creator-docs/getting-started/DOCUMENTATION_INDEX.md)。
- **愿景与路线**：[`creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md`](creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md)、[`creator-docs/roadmap/VISION_OPEN_LAB.md`](creator-docs/roadmap/VISION_OPEN_LAB.md)（开放实验场摘要）。

### 脚手架（`oclive-cli`）

- **crate**：[`crates/oclive-cli/`](crates/oclive-cli/)（workspace 成员）；`cargo run -p oclive-cli -- init` 交互或 `--non-interactive --preset` 生成**可独立 `cargo build`** 的最小内核/库骨架（当前占位依赖 `serde`/`serde_json`，便于硬件与无头场景先统一目录与 `settings.json` 形状）。
- **文档**：[OCLIVE_CLI_GUIDE.md](creator-docs/cli/OCLIVE_CLI_GUIDE.md) · [SETTINGS_REFERENCE.md](creator-docs/cli/SETTINGS_REFERENCE.md)（`plugin_backends` 与预设矩阵）；接入真实 `oclive_kernel_runtime` / `oclive_kernel_server` 时在生成 `Cargo.toml` 中改为 path 依赖并替换入口代码。

### 架构 RFC

- **高耦合编译模式（Monolith）**：[RFC_OCLIVE_MONOLITH_MODE.md](creator-docs/rfc/RFC_OCLIVE_MONOLITH_MODE.md)（路线图见 RFC §9，已与 `oclive-cli` 实现对齐）。**`oclive-cli`**：`init --monolith` 或交互「开发者编译选项」生成 **`monolith.toml`**、`vendor/oclive_monolith_builtin/`（**七槽焊接桩唯一模板源**）、**`process_message_monolith.rs`**、双 **`[[bin]]`**（`main.rs` / `main_monolith.rs`）；**`cargo run -p oclive-cli -- build|bench`** 再生成与对比；**`bench --save` / `--compare`** 用于本地性能历史与对比（见 [OCLIVE_CLI_GUIDE.md](creator-docs/cli/OCLIVE_CLI_GUIDE.md)）。**`oclive dev`**：监听脚手架或内核项目下 **`roles/`** 中 `manifest.json` / `settings.json` 变更，便于热重载脚本对接。

### 内核架构（主应用 `src-tauri`）

- **主编排入口**：Tauri IPC 与 **`--api` HTTP** 均在 **`src-tauri/src/domain/chat_engine/mod.rs`** 的 **`process_message`**（及 `co_present` / `scene` 等子模块）内顺序编排；**入口蓝图（`pipeline.ocblueprint`）已从主路径移除**，不再作为首轮调度 DSL。若 `creator-docs/kernel/` 仍保留 Pipeline Schema 等文档，仅供契约或史料对照，**运行时行为以本仓库 `process_message` 为准**。
- **错误与日志**：统一错误类型见 **`src-tauri/src/error.rs`**（`thiserror`、可映射前端文案）；**机器 `code` 与 JSON 体**以 **`oclive_kernel_runtime::KernelErrorBody`** 与 **`creator-docs/getting-started/KERNEL_ERROR_CODE_CONVENTION.md`** 为准（与 `AppError::code()`、`http_chat_codes` 对齐）。结构化日志为 **`tracing`**（`RUST_LOG`，`init_tracing` 默认 `info`）。
- **启动健康检查**：首轮对话前 **`startup_health::ensure_once`**（槽位、`plugin_backends`、角色包文件、**`DbManager::health_ping`**、可选 LLM 探测）；环境变量 **`OCLIVE_SKIP_STARTUP_HEALTH`** / **`OCLIVE_SKIP_LLM_STARTUP_PROBE`** 可跳过。实现：**`src-tauri/src/domain/startup_health.rs`**。

### 测试体系（三层归属）

- **协议层 → 本仓**：**OOCP HTTP 黑盒（S0–S11）** 已入库且 **CI 已集成**——场景与 CI 说明见 [`creator-docs/testing/OOCP_TEST_SUITE.md`](creator-docs/testing/OOCP_TEST_SUITE.md)；可执行脚本在 [`examples/oocp-test-suite/`](examples/oocp-test-suite/)（`node run.mjs`）。CI **`.github/workflows/ci.yml`** 的 **`oocp-test-suite`** job（Ubuntu）会 `cargo build -p oclivenewnew-tauri`、拉起 **`oclivenewnew-tauri --api`**（默认 **`OCLIVE_HTTP_API_MOCK_LLM=1`**）、轮询 **`GET /health`** 后执行 **`node run.mjs`**，再执行根目录 **`scripts/e2e-core-api-restart.mjs`**（**进程重启后再对话** 烟测，A1.1a）。**Ubuntu `frontend`** job 在 **`npm run build`** 后另跑 **Playwright + `vite preview` 首屏**（A1.1b；Windows `frontend` 不跑 Playwright）。另含 **`src-tauri`** 下 **`cargo test`**、`tests/` 集成测与 HTTP 路由单测等。
- **`invoke` 热路径集成（A1.2）**：矩阵 [`handoff/INVOKE_HOTPATH_MATRIX.md`](handoff/INVOKE_HOTPATH_MATRIX.md)，集成测 [`src-tauri/tests/invoke_hotpath_matrix.rs`](src-tauri/tests/invoke_hotpath_matrix.rs)（**9** 条 `*_impl`；`cargo test -p oclivenewnew-tauri --test invoke_hotpath_matrix`）。
- **组件层 → oclive-pack-editor**：编写器 UI、Vitest、Playwright E2E 等（不在本仓重复维护用例树）。
- **插件层 → oclive-pack-editor**：目录插件范式、**`official-vue-test-runner`** 等；主仓不复制该树。
- **主仓前端最小烟测**：根目录 **`npm run test:unit`**（Vitest，`src/smoke.test.ts`）；**Playwright + `vite preview`**（**`npm run test:e2e:preview`**，**CI 仅 Ubuntu `frontend`**，见 CONTRIBUTING）。
- **总览**：[creator-docs/testing/OVERVIEW.md](creator-docs/testing/OVERVIEW.md)。

### 供应链与安全审计

- **当前状态**：**已知漏洞跟踪中**；**不宣称零漏洞**。摘要执行日期与命中条数见 [creator-docs/development/LIGHTWEIGHT_PROFILE.md](creator-docs/development/LIGHTWEIGHT_PROFILE.md) §6.4；**漏洞级清单与升级路线**见 [creator-docs/security/KNOWN_VULNERABILITIES.md](creator-docs/security/KNOWN_VULNERABILITIES.md)；**审查边界**见 [creator-docs/security/SECURITY_AUDIT_SCOPE.md](creator-docs/security/SECURITY_AUDIT_SCOPE.md)。
- **CI**：**`cargo-audit`** job（**cargo-audit 0.22.1**）为 **`continue-on-error: true`**，用于可见性；待依赖升级后可改为失败即红。

### 复杂情感 `narrative_hint`（共景 → 下一轮 Prompt）

- **类型与内置规则**：[`src-tauri/src/domain/complex_emotion.rs`](src-tauri/src/domain/complex_emotion.rs)（`ComplexEmotionInput` / `ComplexEmotionOutput`、`BuiltinKeywordComplexEmotionProvider::resolve_turn_inner`）；可选 Remote 见 [`src-tauri/src/infrastructure/remote_plugin/complex_emotion_http.rs`](src-tauri/src/infrastructure/remote_plugin/complex_emotion_http.rs)。
- **主路径 wiring**：[`src-tauri/src/domain/chat_engine/co_present.rs`](src-tauri/src/domain/chat_engine/co_present.rs) 在 `load_recent_context` 之后、**`build_prompt` 之前**解析本回合复杂情感；上一轮 `narrative_hint` 缓存在 **`AppState::last_complex_emotion_narrative_hint`**（按会话命名空间 `srid`）；通过 **`PromptInput::previous_complex_emotion_narrative_hint`** 传入 [`PromptBuilder::build_prompt`](src-tauri/src/domain/prompt_builder.rs)（段落标题为「复杂情感叙事提示」）。
- **集成测试**：[`src-tauri/tests/narrative_hint_prompt_roundtrip.rs`](src-tauri/tests/narrative_hint_prompt_roundtrip.rs)。

**契约优先**：角色包 `manifest.json` / `settings.json` 键与行为以 `roles/README_MANIFEST.md`、`RoleStorage::load_role` 及校验 crate 为准；新增顶层键需同步 `crates/oclive_validation` 与文档。

**姊妹仓库**（同级目录常见）：`oclive-pack-editor`（角色包编写器）、`oclive-launcher`（启动器）、`oclive-plugin-market`（市场站）。各仓可有各自的 `AGENTS.md`，指向本仓文档索引即可。

**演示视频（Remotion）**：独立仓库 **`oclive-remotion-demo`**（与主应用同级目录常见）。所有 `npm run preview` / `render:*` / `capture:validate` **须在该仓库根目录执行**，勿在主仓 `oclivenewnew` 根目录运行（会报 `Missing script`）。使用说明见该仓库根目录 **`README.md`**（本地常与主仓并列，例如 `D:\oclive-remotion-demo`）。

**开发机磁盘**：本仓库根目录 [`.cargo/config.toml`](.cargo/config.toml) 将 **Cargo `target-dir`** 指到仓库外的 `../oclive-dev-artifacts/oclivenewnew-cargo-target/`，与源码分离；发版安装包体积与此无关。姊妹仓 **oclive-pack-editor**、**oclive-launcher** 使用同级目录下的 `oclive-pack-editor-cargo-target/`、`oclive-launcher-cargo-target/`（各仓自有 `.cargo/config.toml`）。旧版留在仓库内的 `target/`、`src-tauri/target/` 可整夹删除。

### 前端：插件管理入口与 Tauri `invoke`

- **V1 / V2 路由**：`uiStore.experimentalPluginManagerV2`（Pinia 持久化）为唯一开关；顶栏「更多」与 **Ctrl+Shift+F** 的打开逻辑集中在 [`src/composables/usePluginManagerWindow.ts`](src/composables/usePluginManagerWindow.ts)。设置页、顶栏「更多」与快捷键说明中的**用户可见文案**以 [`src/i18n/locales/zh-CN.ts`](src/i18n/locales/zh-CN.ts) / [`en-US.ts`](src/i18n/locales/en-US.ts) 为准（`settings.*`、`app.more.*`、`common.shortcutHelp.*` 等；设置里需 `v-html` 的段落仅输出静态翻译 HTML，勿拼接用户输入）。
- **V1 已安装区 UI**：侧栏 + 右侧「单插件配置 + 调试台」抽为 [`src/components/InstalledPluginWorkspaceDetail.vue`](src/components/InstalledPluginWorkspaceDetail.vue)，由 [`src/views/PluginManagerPanel.vue`](src/views/PluginManagerPanel.vue) 引用。
- **`invoke` 参数名**：Tauri 将 Rust 命令的 `snake_case` 形参映射为前端的 **camelCase** 键（如 `plugin_id` → `pluginId`）。[`src/utils/tauri-api.ts`](src/utils/tauri-api.ts) 中 `get_plugin_logs`、`spawn_plugin_for_test` 等须与之一致；若命令仍手写 `snake_case` 载荷，会出现「missing required key `pluginId`」类错误。

### Agent / Skill（最小闭环）

- **第七模块**：`plugin_backends` 新增 `agent`（`builtin` / `remote` / `directory` / `none`）与 `directory_plugins.agent` 槽位；会话覆盖与来源快照同样包含 `agent`。（`none` 语义见 `creator-docs/kernel/MODULE_NONE_SEMANTICS.md` §7。）
- **后端骨架**：
  - [`src-tauri/src/domain/agent.rs`](src-tauri/src/domain/agent.rs)：`AgentProvider` trait 与 `BuiltinReActAgent`。
  - [`src-tauri/src/infrastructure/mcp_client.rs`](src-tauri/src/infrastructure/mcp_client.rs)：扫描 `{app_data}/mcp-servers/*.json`、列出 server、调用工具（http/stdio）。
  - [`src-tauri/src/api/agent.rs`](src-tauri/src/api/agent.rs)：`list_mcp_servers` / `call_mcp_tool` / `get_agent_debug_traces` / `clear_agent_debug_traces`。
- **调试 UI**：[`src/components/AgentDebugPanel.vue`](src/components/AgentDebugPanel.vue) 挂在「插件与后端管理 → 后端模块」页，用于查看 Agent 任务拆解与工具调用链路。
- **示例 Skill**：[`examples/weather_skill/`](examples/weather_skill/) 提供最小 Node MCP server（`get_weather(city)`）与示例 server manifest。

### Agent / Skill 通用接入标准（v1）

- **MCP 配置目录**：`{app_data}/mcp-servers/*.json`，支持 `transport=http|stdio`、`timeout_ms`、`tools` 预声明；运行时可 `list_mcp_servers`、`list_mcp_tools`、`call_mcp_tool`。
- **Function Calling**：后端统一走 [`src-tauri/src/infrastructure/function_call_parser.rs`](src-tauri/src/infrastructure/function_call_parser.rs)：
  - `parse_from_llm_response` 解析 `tool_calls[]` 与 `function_call` 两种主流输出；
  - `to_function_calling_schema` 将 MCP tool 列表转为函数 schema。
- **Agent 路由**：`plugin_backends.agent` 为第七模块，与其他模块保持同样的包默认 / 会话覆盖 / 来源快照语义。

## 内核约束 - 权限弹窗

- **Directory 插件**：首次启用高风险能力（如 `process:spawn`、`network:*` 出站）前，必须经过用户确认授予；未授予则必须降级且有可见提示/审计。
- **MCP servers**：任何 `transport=stdio` 的 server 必须显式授权（等同 `process:spawn`）；`transport=http` 必须显式授权（`network:*`）。未授权不得调用。
- **Remote env providers**：检测到 env 配置不等于启用；必须先授予 `network:*`，否则 provider 只能降级为 placeholder 并提示。

### 创作者工具链（v1）

- **脚手架**：`create_plugin_scaffold`（后端）+ `PluginScaffoldWizard.vue`（前端）生成 `manifest.json` + 语言模板 + README，并打开目标目录。
- **一键打包**：`pack_plugin` 校验 manifest 后输出 `.oclive-plugin` 与 `*.signature.json`（SHA-256）。
- **调试体验**：
  - `AgentDebugPanel.vue` 支持模板库（含 localStorage 自定义模板）、请求历史与 Diff 对比；
  - `EnvVarManager.vue` 管理 `OCLIVE_*` 会话草稿并复制 PowerShell 设置命令；
  - `PluginScaffoldWizard.vue` 内置 manifest 实时校验（必填字段与权限枚举约束）。
