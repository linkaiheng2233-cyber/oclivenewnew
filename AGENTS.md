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

- **高耦合编译模式（Monolith）**：[RFC_OCLIVE_MONOLITH_MODE.md](creator-docs/rfc/RFC_OCLIVE_MONOLITH_MODE.md) 描述通过 **`monolith.toml`**（`oclive init` 生成、`oclive build` 读取）与 Cargo **`monolith`** feature 在 **编译期** 焊接选定七槽子系统、消除 trait 虚调用的路线；与运行时 **`pipeline.ocblueprint`** 解耦。**当前状态：第一阶段草案，尚未在 `oclive-cli` 或主仓编排中实现。**

### 测试体系（协议层 + UI 层）

- **OOCP / 无头内核（语言无关）**：标准化场景见 [`creator-docs/oocp/OOCP_TEST_SUITE.md`](creator-docs/oocp/OOCP_TEST_SUITE.md)；索引导航 [`creator-docs/oocp/OOCP_SPEC_COMPLETE_REFERENCE.md`](creator-docs/oocp/OOCP_SPEC_COMPLETE_REFERENCE.md)。官方 Node 可执行对照实现见 [`examples/oocp-test-suite/`](examples/oocp-test-suite/)（对 `oclive_kernel_server` 跑 `GET /health` + WebSocket 方法链）。Linux CI 工作流 **`.github/workflows/ci.yml`** 中的 **`oocp-test-suite`** job 会构建 `tools/oocp-client`、拉起 `oclive_kernel_server` 并执行 `npm test`。
- **前端 / Vitest（框架专用）**：官方目录插件 [`plugins/official-vue-test-runner/README.md`](plugins/official-vue-test-runner/README.md) 通过 JSON-RPC 侧车调用本机 `npx vitest`，在插件壳 [`ui/index.html`](plugins/official-vue-test-runner/ui/index.html) 展示结构化结果与运行历史；编写器侧可在「前端测试」视图调用同一插件（工作区指向 oclivenewnew 仓库根）。编写组件级 OOCP 载荷时可用同目录 [`test_utils/oocp_mock.ts`](plugins/official-vue-test-runner/test_utils/oocp_mock.ts)。

**契约优先**：角色包 `manifest.json` / `settings.json` 键与行为以 `roles/README_MANIFEST.md`、`RoleStorage::load_role` 及校验 crate 为准；新增顶层键需同步 `crates/oclive_validation` 与文档。

**姊妹仓库**（同级目录常见）：`oclive-pack-editor`（角色包编写器）、`oclive-launcher`（启动器）、`oclive-plugin-market`（市场站）。各仓可有各自的 `AGENTS.md`，指向本仓文档索引即可。

**演示视频（Remotion）**：独立仓库 **`oclive-remotion-demo`**（与主应用同级目录常见）。所有 `npm run preview` / `render:*` / `capture:validate` **须在该仓库根目录执行**，勿在主仓 `oclivenewnew` 根目录运行（会报 `Missing script`）。使用说明见该仓库根目录 **`README.md`**（本地常与主仓并列，例如 `D:\oclive-remotion-demo`）。

**开发机磁盘**：本仓库根目录 [`.cargo/config.toml`](.cargo/config.toml) 将 **Cargo `target-dir`** 指到仓库外的 `../oclive-dev-artifacts/oclivenewnew-cargo-target/`，与源码分离；发版安装包体积与此无关。姊妹仓 **oclive-pack-editor**、**oclive-launcher** 使用同级目录下的 `oclive-pack-editor-cargo-target/`、`oclive-launcher-cargo-target/`（各仓自有 `.cargo/config.toml`）。旧版留在仓库内的 `target/`、`src-tauri/target/` 可整夹删除。

### 可编程调度引擎（`pipeline.ocblueprint`）

- **Schema 与错误前缀**：[`creator-docs/kernel/PIPELINE_SCHEMA.md`](creator-docs/kernel/PIPELINE_SCHEMA.md)；加载错误码登记于 [`handoff/10_ERROR_CODE_DICTIONARY.md`](handoff/10_ERROR_CODE_DICTIONARY.md)（Pipeline 蓝图段）。
- **Rust 模块**：`crates/oclive_kernel_runtime/src/domain/chat_engine/` 下的 `pipeline_loader.rs`（解析校验）、`pipeline_interpreter.rs`（顺序 / `branch` / `parallel`）、`pipeline_actions.rs`（原子与 `ACTION_IO_TYPES`）、`pipeline_predicates.rs`、`turn_context.rs`；入口编排见同目录 `process_message.rs`（蓝图在 `validate_scene` 之后加载）。
- **测试**：crate 内 `pipeline_loader` 单测；集成测试 `tests/pipeline_*_smoke.rs`、`tests/pipeline_validator_edges.rs`。
- **Criterion**：`cargo bench -p oclive_kernel_runtime --bench kernel_pipeline_blueprint`；基线记录 [`creator-docs/kernel/KERNEL_PERFORMANCE_BASELINE.md`](creator-docs/kernel/KERNEL_PERFORMANCE_BASELINE.md)。

### 前端：插件管理入口与 Tauri `invoke`

- **V1 / V2 路由**：`uiStore.experimentalPluginManagerV2`（Pinia 持久化）为唯一开关；顶栏「更多」与 **Ctrl+Shift+F** 的打开逻辑集中在 [`src/composables/usePluginManagerWindow.ts`](src/composables/usePluginManagerWindow.ts)。设置页与快捷键说明中的**用户可见文案**集中在 [`src/lib/pluginManagerEntryCopy.ts`](src/lib/pluginManagerEntryCopy.ts)，避免多处硬编码漂移（设置里需 `v-html` 的段落仅输出静态 HTML，勿拼接用户输入）。
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
