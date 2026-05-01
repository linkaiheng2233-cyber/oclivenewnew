# 内核/发行版边界（Kernel / Distribution Boundary�?

> 版本：v0.2（可执行基线�? 
> 生效范围：oclivenewnew 仓库（`src-tauri/`�? 
> 维护者：�?`handoff/WEEKLY_DEV_GUIDE.md` 节奏更新

---

## 0. Kernel Baseline v1（建议冻结对象）

本仓库以 “Linux 设计哲学�?演进�?*Kernel 只提供最小可信闭�?+ 标准化可替换�?*，发行版负责 UI/分发/体验�?

- **Kernel Baseline v1 文档（建议作为对�?验收基线�?*�?
  - **[KERNEL_BASELINE_V1.md](./KERNEL_BASELINE_V1.md)**
- **Module 8：Frontend Shell（发行版 UI 模块�?*�?
  - **[MODULE_8_FRONTEND_SHELL.md](./MODULE_8_FRONTEND_SHELL.md)**
- **Module 9：专家模型设施（内核托管）**
  - **[MODULE_9_EXPERT_MODELS_FACILITY.md](./MODULE_9_EXPERT_MODELS_FACILITY.md)**

---

## 1. 术语定义

| 术语 | 含义 |
|------|------|
| **内核（core�?* | 平台无关的领域逻辑与调度。不依赖 Tauri、OS 窗口、快捷键、渲染。可独立编译为库（`oclive_core`）�?|
| **发行版（distribution�?* | 依赖特定平台的适配层：Tauri 桌面端、VSCode 扩展、CLI、HTTP API�?|
| **适配器（adapter�?* | 连接“内核能力”与“发行版传输/UI”的薄层（如 `invoke` handler �?domain call）�?|
| **OOCP** | OClive Open Control Protocol �?内核对外暴露的统一能力面（方法 + 事件 stream）�?|

---

## 2. 内核包含（永久保留于 `oclive_core`�?

以下领域逻辑属于内核�?*不依�?* Tauri / 操作系统 / 窗口�?

### 2.1 对话调度与编�?
- **主入口**：`process_message`（`crates/oclive_kernel_runtime/src/domain/chat_engine/process_message.rs`）
- 共景（co_present）、异地占位（remote_stub）、异地心声（remote_life）模式调�?
- 回合管线：用户情绪分�?�?事件检�?�?性格演化 �?记忆检�?�?Prompt 构建 �?LLM 调用 �?回复后处�?�?持久�?

### 2.2 降级链（Fallback Chain�?
- LLM 失败时的备用短回复（`chat_llm_fallback`�?
- 幻觉 token 剥离（`strip_hallucination_tokens`�?
- 软追加防护（`soft_append_guard`�?

### 2.3 角色包解�?
- manifest.json 加载与校验（`role_manifest_validate`、`oclive_validation` crate�?
- settings.json、scenes/、knowledge_index 解析
- `Role` 模型构建

### 2.4 插件协议面（Plugin Protocol�?
- 插件主机（`PluginHost`）：后端解析、能力匹�?
- 本地插件桥（`LocalPluginBridge`）：invoke 白名单、事件订�?
- 插件后端路由：memory / emotion / event / prompt / llm / agent 六大模块

### 2.5 OOCP 协议�?
- 方法路由（method �?domain call map�?
- capabilities 声明
- 事件流定�?
- 请求/响应/事件 schema（OOCP types�?

### 2.6 MCP Client 协议�?
- MCP server 发现（`mcp-servers/*.json`�?
- tool 列表 / 调用
- �?Agent 的集�?

### 2.7 持久化接口（Repository trait�?
- `MemoryRepository`：长�?短期记忆读写，检索排�?
- `FavorabilityRepository`：好感度、关系阶段、身份维�?
- `EventRepository`：事件记�?
- 角色运行时（`role_runtime`）：场景、情绪、人格向�?档案、交互模�?
- 数据库表结构�?`crates/oclive_kernel_runtime/migrations/001_init.sql` 为准

### 2.8 业务引擎（engines/analyzers�?
- 情绪分析：`user_emotion_analyzer`、`emotion_analyzer`、`complex_emotion`
- 性格引擎：`personality_engine`、`profile_personality`、`mutable_profile_llm`
- 关系引擎：`relation_engine`（好感阶段判定）
- 事件引擎：`event_detector`、`event_estimator`、`event_impact_ai`
- 记忆引擎：`memory_engine`、`memory_retrieval`
- Prompt：`prompt_builder`、`prompt_assembler`
- 策略：`policy`、`affect_policy`
- Agent：`agent`

### 2.9 专家模型设施（Module 9）

- **简称**：专家模型设施。**全称**：专家模型设施模块（与 UI 文案 *Expert Models / Module 9* 互参）。
- **定位**：内核托管的 **配置 / 资产型设施**（`role_runtime` JSON、`ExpertModelsRepository`、图编译、Prompt 风格覆盖等），**不是** `PluginBackends` 中与 memory 平行的路由槽。
- **详述**：[MODULE_9_EXPERT_MODELS_FACILITY.md](./MODULE_9_EXPERT_MODELS_FACILITY.md)

---

## 3. 发行版包含（不进�?`oclive_core`�?

以下属于发行版适配层，依赖具体平台�?

### 3.1 Tauri 桌面�?
- UI（Vue 3 前端�?
- 窗口管理、快捷键（`hotkeys`�?
- `tauri::generate_handler!` 注册�?invoke 命令（`src-tauri/src/api/*.rs`�?
- 插件协议桥（`plugin_bridge_invoke`）WebView �?Rust
- 插件 HTML 注入（`inject_plugin_bridge_script`�?
- 插件 asset server（`serve_ocliveplugin_asset`�?
- 深度链接（`oclive://...`�?
- 文件系统监听（`plugin_fs_watcher`�?

### 3.2 HTTP API（`run_api_server`�?
- 用于 pack-editor 试聊的本�?HTTP 接口

### 3.3 VSCode 扩展（P1�?
- VSCode webview / extension host
- OOCP WS client �?连接到内�?

### 3.4 CLI（未来）
- 命令行工具、批量处�?

### 3.5 UI/渲染/主题/交互
- 前端所�?Vue 组件、视�?
- 样式系统、主�?
- 前端 stores、composables
- `ui.json` 渲染

---

## 4. 冻结对象与版本策�?

### 4.1 v0.x（当前，可变更期�?
以下接口�?v0.x 期间可能调整，但**必须同步更新此文档与 OOCP spec**�?
- 所�?domain trait 签名
- DTO 字段（以 `crates/oclive_kernel_runtime/src/models/dto.rs` 为准�?
- Repository trait 方法签名
- PluginBackends 枚举与路由逻辑

### 4.2 v1.0 冻结（计划冻结）
以下对象�?v1.0 发布后进�?**Deprecation + 迁移周期**�?
- OOCP `capabilities` 版本号与语义
- OOCP 方法名（`session.create` / `chat.send_message` 等）
- 事件类型�?payload schema
- 数据�?schema（`migrations/`，仅允许 ALTER TABLE ADD COLUMN�?
- DTO `reply` 字段名（**永不改名**�?

---

## 5. 代码分层（当前落地）

领域编排、引擎、Repository trait、DB 与 SQLx 迁移的**单一真相源**在 **`crates/oclive_kernel_runtime/`**（crate `oclive_kernel_runtime`）。Tauri **`src-tauri/`** 保留 **`api/*.rs`**、**`domain/adapters/`**（OOCP 等）、**`lib.rs` 注册**。

**已与内核对齐、Tauri 侧仅为 re-export / 别名（避免 `DbManager` / `PolicyContext` 等类型双轨）：**

- **`state`**：`pub type AppState = KernelAppState`；`resolve_roles_dir`、`PolicySet` 与内核一致。
- **`infrastructure/db.rs`**、**`domain/policy.rs`**、**`domain/repository.rs`**、**`infrastructure/repositories.rs`**：对内核模块 `pub use`。

**`domain`**：`src-tauri/src/domain/mod.rs` 对 **`oclive_kernel_runtime::domain`** 做子模块级 **`pub use`**（含 **`permission_tokens`**）；本地仅保留 **`adapters/`**（Tauri OOCP 等）。编排入口仍为 **`chat_engine::process_message`**（内核实现）。

```
crates/oclive_kernel_runtime/
├── migrations/
├── src/domain/             # chat_engine、plugin_host、repository、policy …
├── src/infrastructure/     # db、repositories_runtime、llm、remote_plugin …
└── src/state/              # KernelAppState、resolve_roles_dir

src-tauri/src/
├── api/
├── domain/adapters/        # OOCP / Tauri 专用
├── domain/mod.rs           # 对内核 domain 子模块 pub use；仅 adapters 本地
└── lib.rs
```

后续新增业务逻辑应落在 **`oclive_kernel_runtime`**，避免在 **`src-tauri/src/api`** 堆叠公式。

---

## 6. 内核入口清单（当前对外能力）

此节列出当前通过 Tauri invoke 对外暴露的所有命令名、输�?输出 DTO 及事�?stream�?
所�?OOCP 方法请参�?`creator-docs/oocp/OOCP_SPEC_v0_1.md`�?

详见随附文档�?*[KERNEL_ENTRY_CHECKLIST.md](./KERNEL_ENTRY_CHECKLIST.md)**

---

## 7. 禁止事项（硬约束�?

- �?内核代码不得 `use tauri::*` 或依�?`tauri` crate
- �?内核代码不得访问 `AppHandle`、`Window`、`Manager`
- �?DTO 字段 `reply` 不得改名
- 数据库表不得虚构（以 `crates/oclive_kernel_runtime/migrations/001_init.sql` 为准）
- �?不得�?API 层（`src-tauri/src/api/*.rs`）编写业务逻辑
