# Kernel Baseline v1（AI Linux 内核基线）

> 版本：v1（基线草案，建议冻结）  
> 目标：让创作者/发行版 **不用从零开始**，但又不把发行版体验写死在 Kernel。  
> 原则：Kernel 提供 “**最小可用闭环 + batteries included 默认实现 + 标准化替换接口**”；Distribution 负责 UI/分发/体验/打包策略。

---

## 1. Kernel 必须提供的能力（batteries included）

这些能力对应 Linux 的 “syscall + 默认驱动 + 可运行的最小用户态”，保证 **5 分钟跑通**（加载角色 → 建会话 → 发消息 → 得到回复 → 状态落库）。

### 1.1 协议与对外能力面（ABI）

- **OOCP**（WebSocket 优先）：方法名、capabilities、事件流。
- **版本字段**：`capabilities.version` / `schema_version` / `limits` 必须稳定。
- **鉴权**：支持 token（如 `OOCP_API_TOKEN`）及 `auth_required` 语义对齐。

### 1.2 会话与状态机（Kernel 闭环）

- session：create / destroy / get_state / switch_scene / switch_interaction_mode
- chat：send_message（主入口）+ generate_monologue（可选但建议保留）
- time：get_state + jump（用于 life schedule / remote life 等逻辑）

### 1.3 持久化与迁移（可升级）

- **SQLite + migrations**：schema 以 `src-tauri/migrations/001_init.sql` 为唯一真相源。
- 事务一致性：回合级原子更新（例如 `apply_chat_turn_atomic`）保证 “一次发消息” 的状态一致。
- Repository trait + SQLite 实现：Kernel runtime 必须能独立运行并落库。

### 1.4 内置默认实现（可用即默认）

Kernel 必须自带 builtin 路径（否则创作者必然要先搭一堆外部服务）：

- **Emotion**：`emotion_analyzer` / `user_emotion_analyzer` builtin / builtin_v2
- **Event**：`event_detector` + `event_estimator` builtin / builtin_v2（含 AI 估计与规则回退）
- **Memory**：`memory_engine` + `memory_retrieval` builtin / builtin_v2（含 query overlap boost）
- **Prompt**：`prompt_builder` + `prompt_assembler` builtin / builtin_v2
- **LLM**：Ollama（本地）+ remote（HTTP JSON-RPC）两条路径
- **Agent**：Builtin ReAct Agent（可调用 MCP tools；可查看 traces）

> 提醒：前端 UI 不属于 Kernel（见 §3），但可以作为 **Module 8：Frontend Shell** 与 1–7 模块并列，围绕“可变内核”形成不同发行版体验：
> - **[MODULE_8_FRONTEND_SHELL.md](./MODULE_8_FRONTEND_SHELL.md)**

---

## 2. Kernel 的标准化替换点（可插拔接口）

这些替换点是生态边界：第三方发行版或创作者想替换能力时，只替换一层，不 fork kernel。

### 2.1 PluginBackends（编排选择器）

- `plugin_backends.*` 为单一事实来源，允许选择 builtin / builtin_v2 / remote / directory / local（按模块不同而异）。
- Kernel 负责解析 + fallback：remote/directory 不可用时回退 builtin（并打日志）。

### 2.2 PluginHost（后端解析/选择）

- Kernel runtime 必须提供 `PluginHost`：把 `Role.plugin_backends` + session override 解析成一组 `Arc<dyn Trait>` 句柄。
- `ResolvedRolePlugins` 在单次 `send_message` 内只解析一次并复用（性能与行为一致性）。

### 2.3 Remote Plugin Sidecar（HTTP JSON-RPC）

- Kernel runtime 必须提供 `remote_plugin`：
  - `memory.rank` / `emotion.analyze` / `event.estimate` / `prompt.build_prompt` / `prompt.top_topic_hint`
  - 以及 remote LLM：`llm.generate` / `llm.generate_tag`
- 失败回退 builtin：保持创作者最小闭环可用。

### 2.4 Directory Plugins（进程插件槽位）

- `directory_plugins.*` 槽位选择，wire 与 remote plugin 一致（HTTP JSON-RPC）。
- Kernel 负责懒启动、RPC url 解析、失败回退 builtin。

### 2.5 Local Plugin Bridge（注册与门禁）

- Kernel 提供 provider descriptor、schema_version 门禁、capability registry。
- 当前阶段可先支持 “发现/注册/选择”，真正执行可逐步扩展（但替换点先固定）。

---

## 3. 不属于 Kernel 的能力（必须留在发行版）

这些属于 “发行版体验与生态运营层”，放进 Kernel 会导致不可复用、强绑定平台。

- UI（Vue / Webview / 主题 / 动画 / 交互）
- 快捷键、窗口管理、系统托盘、深链、文件系统 watcher
- 角色包编辑器、市场、安装器、更新器、资产服务器
- 平台特定权限申请与系统集成

Kernel 只提供协议与能力面，发行版用 OOCP/Tauri invoke/HTTP 去编排体验。

---

## 4. v1 冻结范围（建议）

建议在 v1.0 后进入 “deprecated + 迁移窗口” 的对象：

- **OOCP 方法名 + payload schema**
- **capabilities 语义（auth_required/limits/schema_version/version）**
- **DTO 对外字段名**（尤其 `reply`）
- **PluginBackends 枚举与 fallback 语义**
- **DB schema**：仅允许 add column（禁止破坏性改动）

---

## 5. 与当前代码的映射（现状对齐）

- Kernel runtime：`crates/oclive_kernel_runtime`
- Kernel server：`crates/oclive_kernel_server`
- Distribution（桌面）：`src-tauri` + 前端 UI
- 入口清单：`creator-docs/kernel/KERNEL_ENTRY_CHECKLIST.md`
- 插件契约：`creator-docs/plugin-and-architecture/PLUGIN_V1.md`

