# Agent / AI 协作说明（oclivenewnew）

本仓库为 **Tauri + Vue 3 + Rust** 桌面角色对话应用。自动化助手或外部 Agent 在修改代码前，请先阅读：

- **项目约束**：根目录 [`.cursor/rules/oclivenewnew.mdc`](.cursor/rules/oclivenewnew.mdc)（编排、持久化、Tauri 命令注册、DTO、Prompt 约定）。
- **创作者与架构文档**：[`creator-docs/README.md`](creator-docs/README.md) → [`creator-docs/getting-started/DOCUMENTATION_INDEX.md`](creator-docs/getting-started/DOCUMENTATION_INDEX.md)。
- **愿景与路线**：[`creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md`](creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md)、[`creator-docs/roadmap/VISION_OPEN_LAB.md`](creator-docs/roadmap/VISION_OPEN_LAB.md)（开放实验场摘要）。
- **轻量化与审计基线**：[`creator-docs/development/LIGHTWEIGHT_PROFILE.md`](creator-docs/development/LIGHTWEIGHT_PROFILE.md)。

**契约优先**：角色包 `manifest.json` / `settings.json` 键与行为以 `roles/README_MANIFEST.md`、`RoleStorage::load_role` 及校验 crate 为准；新增顶层键需同步 `crates/oclive_validation` 与文档。

**姊妹仓库**（同级目录常见）：`oclive-pack-editor`（角色包编写器）、`oclive-launcher`（启动器）、`oclive-plugin-market`（市场站）。各仓可有各自的 `AGENTS.md`，指向本仓文档索引即可。

**开发机磁盘**：本仓库根目录 [`.cargo/config.toml`](.cargo/config.toml) 将 **Cargo `target-dir`** 指到仓库外的 `../oclive-dev-artifacts/oclivenewnew-cargo-target/`，与源码分离；发版安装包体积与此无关。姊妹仓 **oclive-pack-editor**、**oclive-launcher** 使用同级目录下的 `oclive-pack-editor-cargo-target/`、`oclive-launcher-cargo-target/`（各仓自有 `.cargo/config.toml`）。旧版留在仓库内的 `target/`、`src-tauri/target/` 可整夹删除。

### 编排（内核）

- **主编排入口**：[`src-tauri/src/domain/chat_engine/mod.rs`](src-tauri/src/domain/chat_engine/mod.rs) 的 **`process_message`**（HTTP `--api` 与 Tauri `send_message` 均经此路径）。子路径含共景 **`co_present::process_co_present`**、异地/远程等分支。
- **无独立「入口蓝图」管线**：编排逻辑集中在 `chat_engine` 与各 `*_engine` / analyzer；**不再**使用单独的入口蓝图 DSL 作为主路径。
- **可替换子系统**：[`PluginHost`](src-tauri/src/domain/plugin_host.rs) 按 `plugin_backends` 解析 **memory / emotion / event / prompt / llm / agent**；详见 [PLUGIN_V1.md](creator-docs/plugin-and-architecture/PLUGIN_V1.md)。

### 测试体系（三层）

与 **当前 `main` 代码** 对齐的划分如下（**不**宣称已存在未落地的 OOCP CI）：

1. **协议 / 引擎层（Rust）**：`src-tauri` 内 **`cargo test`**（含 `tests/` 集成测试、领域单测）。CI 在 Ubuntu / Windows 上执行。契约以 **`models/dto.rs`** 为准。
2. **插件 / 侧车层**：`examples/remote_plugin_minimal` 等 + CI job **`remote-plugin-demo`**（`memory.rank` JSON-RPC 烟测）。目录式插件见 [DIRECTORY_PLUGINS.md](creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md)。
3. **前端 / 组件层**：CI 执行 **`npm ci` + `npm run build`** 作为静态构建守门；**未** 在 `package.json` 中启用 Vitest 单测脚本。若引入 Vue 单测，应更新 [`creator-docs/testing/TEST_OUTPUT_SCHEMA.md`](creator-docs/testing/TEST_OUTPUT_SCHEMA.md)。

**OOCP**：若独立协议套件落地，见 [`creator-docs/testing/OOCP_TEST_SUITE.md`](creator-docs/testing/OOCP_TEST_SUITE.md)（当前为占位说明）。

**插件后端烟测**：[`src-tauri/tests/plugin_backends_v2_resolve.rs`](src-tauri/tests/plugin_backends_v2_resolve.rs)（`cargo test --test plugin_backends_v2_resolve`）。

### 已知漏洞跟踪

- **清单与升级路线**：[creator-docs/security/KNOWN_VULNERABILITIES.md](creator-docs/security/KNOWN_VULNERABILITIES.md)（`cargo-audit` **0.22.1**，漏洞级命中 **5** 条，**2026-05-13** 扫描）。
- **供应链基线**：[creator-docs/development/LIGHTWEIGHT_PROFILE.md](creator-docs/development/LIGHTWEIGHT_PROFILE.md) §6.4–6.7。
- **声明**：**已知漏洞跟踪中**；不对外宣称「audit 全绿 / 零漏洞」。CI **`cargo-audit`** job 当前 **`continue-on-error: true`**。

### 安全审查范围（当前）

- **已完成 / 未覆盖 / 后续计划**：[creator-docs/security/SECURITY_AUDIT_SCOPE.md](creator-docs/security/SECURITY_AUDIT_SCOPE.md)。
- 与 **KNOWN_VULNERABILITIES** 互补：后者管 **RUSTSEC 编号与升级**；本文件管 **流程与范围边界**。

### 前端：插件管理入口与 Tauri `invoke`

- **V1 / V2 路由**：`uiStore.experimentalPluginManagerV2`（Pinia 持久化）为唯一开关；顶栏「更多」与 **Ctrl+Shift+F** 的打开逻辑集中在 [`src/composables/usePluginManagerWindow.ts`](src/composables/usePluginManagerWindow.ts)。设置页与快捷键说明中的**用户可见文案**集中在 [`src/lib/pluginManagerEntryCopy.ts`](src/lib/pluginManagerEntryCopy.ts)，避免多处硬编码漂移（设置里需 `v-html` 的段落仅输出静态 HTML，勿拼接用户输入）。
- **V1 已安装区 UI**：侧栏 + 右侧「单插件配置 + 调试台」抽为 [`src/components/InstalledPluginWorkspaceDetail.vue`](src/components/InstalledPluginWorkspaceDetail.vue)，由 [`src/views/PluginManagerPanel.vue`](src/views/PluginManagerPanel.vue) 引用。
- **`invoke` 参数名**：Tauri 将 Rust 命令的 `snake_case` 形参映射为前端的 **camelCase** 键（如 `plugin_id` → `pluginId`）。[`src/utils/tauri-api.ts`](src/utils/tauri-api.ts) 中 `get_plugin_logs`、`spawn_plugin_for_test` 等须与之一致；若命令仍手写 `snake_case` 载荷，会出现「missing required key `pluginId`」类错误。

### Agent / Skill（最小闭环）

- **第七模块**：`plugin_backends` 新增 `agent`（`builtin` / `remote` / `directory`）与 `directory_plugins.agent` 槽位；会话覆盖与来源快照同样包含 `agent`。
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

### 创作者工具链（v1）

- **脚手架**：`create_plugin_scaffold`（后端）+ `PluginScaffoldWizard.vue`（前端）生成 `manifest.json` + 语言模板 + README，并打开目标目录。
- **一键打包**：`pack_plugin` 校验 manifest 后输出 `.oclive-plugin` 与 `*.signature.json`（SHA-256）。
- **调试体验**：
  - `AgentDebugPanel.vue` 支持模板库（含 localStorage 自定义模板）、请求历史与 Diff 对比；
  - `EnvVarManager.vue` 管理 `OCLIVE_*` 会话草稿并复制 PowerShell 设置命令；
  - `PluginScaffoldWizard.vue` 内置 manifest 实时校验（必填字段与权限枚举约束）。
