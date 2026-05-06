# 内核/发行版边界（Kernel / Distribution Boundary）

> 版本：v0.2（可执行基线）  
> 生效范围：oclivenewnew 仓库（`src-tauri/`）  
> 维护者：按 `handoff/WEEKLY_DEV_GUIDE.md` 节奏更新

---

## 0. Kernel Baseline v1（建议冻结对象）

本仓库以 “Linux 设计哲学” 演进：**Kernel 只提供最小可信闭包 + 标准化可替换接口**，发行版负责 UI/分发/体验。

- **Kernel Baseline v1 文档（建议作为对外验收基线）**：
  - **[KERNEL_BASELINE_V1.md](./KERNEL_BASELINE_V1.md)**
- **Module 8：Frontend Shell（发行版 UI 模块）**：
  - **[MODULE_8_FRONTEND_SHELL.md](./MODULE_8_FRONTEND_SHELL.md)**
- **Module 9：专家模型设施（内核托管）**
  - **[MODULE_9_EXPERT_MODELS_FACILITY.md](./MODULE_9_EXPERT_MODELS_FACILITY.md)**

---

## 1. 术语定义

| 术语 | 含义 |
|------|------|
| **内核（core）** | 平台无关的领域逻辑与调度。不依赖 Tauri、OS 窗口、快捷键、渲染。可独立编译为库（`oclive_kernel_core` / `oclive_kernel_runtime` 等 crate 组合）。 |
| **发行版（distribution）** | 依赖特定平台的适配层：Tauri 桌面端、VSCode 扩展、CLI、HTTP API。 |
| **适配器（adapter）** | 连接“内核能力”与“发行版传输/UI”的薄层（如 `invoke` handler → domain call）。 |
| **OOCP** | OClive Open Control Protocol：内核对外暴露的统一能力面（方法 + 事件 stream）。 |

---

## 1.1 官方默认模块（Kernel V2 产品术语）

以下名称指 **随官方发行版提供、经 Cargo feature 可选链接的进程内 Builtin 实现**；工程上对应 **`oclive_*_builtin`** crate，历史文档亦常称 **设施 crate**。

| 产品术语 | Crate | 典型 feature |
|----------|--------|----------------|
| 官方默认记忆模块 | `oclive_memory_builtin` | `default-memory-providers` |
| 官方默认情绪模块 | `oclive_emotion_builtin` | `default-emotion-providers` |
| 官方默认复杂情感模块 | `oclive_complex_emotion_builtin` | `default-complex-emotion-providers` |
| 官方默认事件模块 | `oclive_event_builtin` | `default-event-providers` |
| 官方默认 Prompt 模块 | `oclive_prompt_builtin` | `default-prompt-providers` |
| 官方默认 Agent 模块 | `oclive_agent_builtin` | `default-agent-providers` |

**与编号模块的区分**：**第九模块（专家模型设施）** 为内核托管的 ExpertGraph / Prompt 风格 / 侧车编译等，**不是** 上表中的「官方默认××模块」，也 **不是** `plugin_backends` 中与 `memory` 同形的枚举槽位。详见 [MODULE_9_EXPERT_MODELS_FACILITY.md](./MODULE_9_EXPERT_MODELS_FACILITY.md) §2、§5。

角色包 **`plugin_backends.* = builtin`** 时，由 **`PluginHost`** 装配的进程内路径才可落到上表实现（对应 feature 关闭时见桩/降级，[LIGHTWEIGHT_PROFILE.md](./LIGHTWEIGHT_PROFILE.md)）。

---

## 2. 内核包含（平台无关域逻辑；主力实现见 `crates/oclive_kernel_runtime`）

以下领域逻辑属于内核，**不依赖** Tauri / 操作系统 / 窗口。

### 2.1 对话调度与编排

- **主入口**：`process_message`（`crates/oclive_kernel_runtime/src/domain/chat_engine/process_message.rs`）
- 共景（co_present）、异地占位（remote_stub）、异地心声（remote_life）模式调度
- 回合管线：用户情绪分析 → 事件检测 → 性格演化 → 记忆检索 → Prompt 构建 → LLM 调用 → 回复后处理 → 持久化

### 2.2 降级链（Fallback Chain）

- LLM 失败时的备用短回复（`chat_llm_fallback`）
- 幻觉 token 剥离（`strip_hallucination_tokens`）
- 软追加防护（`soft_append_guard`）

### 2.3 角色包解析

- manifest.json 加载与校验（`role_manifest_validate`、`oclive_validation` crate）
- settings.json、scenes/、knowledge_index 解析
- `Role` 模型构建

### 2.4 插件协议面（Plugin Protocol）

- 插件主机（`PluginHost`）：后端解析、能力匹配
- 本地插件桥（`LocalPluginBridge`）：invoke 白名单、事件订阅
- 插件后端路由：memory / emotion / event / prompt / llm / agent 六大模块

### 2.5 OOCP 协议层

- 方法路由（method → domain call map）
- capabilities 声明
- 事件流定义
- 请求/响应/事件 schema（OOCP types）

### 2.6 MCP Client 协议层

- MCP server 发现（`mcp-servers/*.json`）
- tool 列表 / 调用
- 与 Agent 的集成

### 2.7 持久化接口（Repository trait）

- `MemoryRepository`：长期/短期记忆读写，检索排序
- `FavorabilityRepository`：好感度、关系阶段、身份维度
- `EventRepository`：事件记录
- 角色运行时（`role_runtime`）：场景、情绪、人格向量、档案、交互模式
- 数据库表结构以 `crates/oclive_kernel_runtime/migrations/001_init.sql` 为准

### 2.8 业务引擎（engines/analyzers）

- 情绪分析：`user_emotion_analyzer`、`emotion_analyzer`、`complex_emotion`
- 性格引擎：`personality_engine`、`profile_personality`、`mutable_profile_llm`
- 关系引擎：`relation_engine`（好感阶段判定）
- 事件引擎：`event_detector`、`event_estimator`、`event_impact_ai`
- 记忆引擎：`memory_engine`、`memory_retrieval`
- Prompt：`prompt_builder`、`prompt_assembler`
- 策略：`policy`、`affect_policy`
- Agent：`agent`（`AgentProvider`；进程内 **ReAct** 默认实现在 **`oclive_agent_builtin`**；**`McpShellAgent`** 仍驻 `kernel_runtime`，见 `LIGHTWEIGHT_PROFILE.md`）

### 2.9 专家模型设施（Module 9）

- **简称**：专家模型设施。**全称**：专家模型设施模块（与 UI 文案 *Expert Models / Module 9* 互参）。
- **定位**：内核托管的 **配置 / 资产型设施**（`role_runtime` JSON、`ExpertModelsRepository`、图编译、Prompt 风格覆盖等），**不是** `PluginBackends` 中与 memory 平行的路由槽。
- **详述**：[MODULE_9_EXPERT_MODELS_FACILITY.md](./MODULE_9_EXPERT_MODELS_FACILITY.md)

---

## 3. 发行版包含（不进入内核 crate 树中的平台层；如 `src-tauri/`）

以下属于发行版适配层，依赖具体平台。

### 3.1 Tauri 桌面端

- UI（Vue 3 前端）
- 窗口管理、快捷键（`hotkeys`）
- `tauri::generate_handler!` 注册与 invoke 命令（`src-tauri/src/api/*.rs`）
- 插件协议桥（`plugin_bridge_invoke`）WebView ↔ Rust
- 插件 HTML 注入（`inject_plugin_bridge_script`）
- 插件 asset server（`serve_ocliveplugin_asset`）
- 深度链接（`oclive://...`）
- 文件系统监听（`plugin_fs_watcher`）

### 3.2 HTTP API（`run_api_server`）

- 用于 pack-editor 试聊的本地 HTTP 接口

### 3.3 VSCode 扩展（P1）

- VSCode webview / extension host
- OOCP WS client → 连接到内核

### 3.4 CLI（未来）

- 命令行工具、批量处理

### 3.5 UI/渲染/主题/交互

- 前端所有 Vue 组件、视图
- 样式系统、主题
- 前端 stores、composables
- `ui.json` 渲染

---

## 4. 冻结对象与版本策略

### 4.1 v0.x（当前，可变更期）

以下接口在 v0.x 期间可能调整，但**必须同步更新此文档与 OOCP spec**。

- 所有 domain trait 签名
- DTO 字段（以 `crates/oclive_kernel_runtime/src/models/dto.rs` 为准）
- Repository trait 方法签名
- PluginBackends 枚举与路由逻辑

### 4.2 v1.0 冻结（计划冻结）

以下对象在 v1.0 发布后进入 **Deprecation + 迁移周期**。

- OOCP `capabilities` 版本号与语义
- OOCP 方法名（`session.create` / `chat.send_message` 等）
- 事件类型与 payload schema
- 数据库 schema（`migrations/`，仅允许 ALTER TABLE ADD COLUMN）
- DTO `reply` 字段名（**永不改名**）

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

### 5.1 轻量编译配置（可选特性 / OOCP / invoke）

嵌入式宿主与 SKU 裁剪时的 **`Cargo` 特性组合**、OOCP 行为说明、Tauri `invoke` 分组及 **`http_api` / 依赖去重** 拟定说明见 **[LIGHTWEIGHT_PROFILE.md](./LIGHTWEIGHT_PROFILE.md)**。

---

## 6. 内核入口清单（当前对外能力）

此节列出当前通过 Tauri invoke 对外暴露的所有命令名、输入/输出 DTO 及事件 stream。  
所有 OOCP 方法请参见 `creator-docs/oocp/OOCP_SPEC_v0_1.md`。

详见随附文档：**[KERNEL_ENTRY_CHECKLIST.md](./KERNEL_ENTRY_CHECKLIST.md)**

---

## 7. 禁止事项（硬约束）

- **禁止**：内核代码不得 `use tauri::*` 或依赖 `tauri` crate
- **禁止**：内核代码不得访问 `AppHandle`、`Window`、`Manager`
- **禁止**：DTO 字段 `reply` 不得改名
- 数据库表不得虚构（以 `crates/oclive_kernel_runtime/migrations/001_init.sql` 为准）
- **禁止**：不得在 API 层（`src-tauri/src/api/*.rs`）编写业务逻辑

---

## 8. Kernel V2：trait / 共享类型 / feature（阶段 6 定型）

本节与 **[KERNEL_V2_DESIGN.md](./KERNEL_V2_DESIGN.md) §6** 对齐，便于宿主与 **官方默认模块（`oclive_*_builtin`）** 选型依赖；**源码为最终权威**。（历史用语「设施 crate」与此同指；产品命名见 **§1.1**。）

### 8.1 `oclive_kernel_core`（协议与门面 trait）

| 项 | 源码路径 |
|----|----------|
| `AppError` / `Result` | `crates/oclive_kernel_core/src/error.rs` |
| `Memory` / `MemoryContext` | `crates/oclive_kernel_core/src/models/memory.rs` |
| `Emotion` | `crates/oclive_kernel_core/src/models/emotion.rs` |
| `LlmClient` | `crates/oclive_kernel_core/src/llm.rs` |
| `MemoryRetrieval` | `crates/oclive_kernel_core/src/memory_retrieval.rs` |
| `UserEmotionAnalyzer` | `crates/oclive_kernel_core/src/user_emotion_analyzer.rs` |
| `ComplexEmotionProvider` | `crates/oclive_kernel_core/src/complex_emotion.rs` |
| `AgentProvider` | `crates/oclive_kernel_core/src/agent.rs` |
| `EventEstimator` | `crates/oclive_kernel_core/src/event_estimator.rs` |
| `PromptAssembler` / `PromptInput` / `PromptRolePromptSlice`（再导出） | `crates/oclive_kernel_core/src/prompt.rs` |
| `DEFAULT_REPLY_QUALITY_ANCHOR` / `effective_reply_quality_anchor` | `crates/oclive_kernel_core/src/prompt.rs` |
| Repository traits | `crates/oclive_kernel_core/src/repository.rs` |

### 8.2 `oclive_kernel_models`（纯数据，无 I/O）

| 项 | 模块路径 |
|----|----------|
| `EventType` / `Event` | `crates/oclive_kernel_models/src/event.rs` |
| `KnowledgeEventAugment` | `crates/oclive_kernel_models/src/knowledge_augment.rs` |
| `PersonalityVector` | `crates/oclive_kernel_models/src/personality.rs` |
| `EvolutionConfig` / `MemoryConfig` / `UserRelation` / … | `crates/oclive_kernel_models/src/role_config.rs` |
| `EventImpactEstimate` | `crates/oclive_kernel_models/src/event_impact.rs` |
| `PromptRolePromptSlice` | `crates/oclive_kernel_models/src/prompt_role.rs` |

**依赖方向**：`kernel_models` 不依赖 `kernel_core`；`kernel_core` 依赖 `kernel_models`（trait 签名与 Prompt/Event DTO）。

### 8.2.1 设施 crate（`oclive_*_builtin`）与目录示例索引

| Crate | 路径 | 示例插件（`examples/`） |
|-------|------|-------------------------|
| `oclive_memory_builtin` | `crates/oclive_memory_builtin` | `oclive-memory-builtin-directory` |
| `oclive_emotion_builtin` | `crates/oclive_emotion_builtin` | `oclive-emotion-builtin-directory` |
| `oclive_complex_emotion_builtin` | `crates/oclive_complex_emotion_builtin` | `oclive-complex-emotion-builtin-directory` |
| `oclive_prompt_builtin` | `crates/oclive_prompt_builtin` | `oclive-prompt-builtin-directory`（需构建 **`oclive_prompt_from_json`**，见示例 README） |
| `oclive_agent_builtin` | `crates/oclive_agent_builtin` | `oclive-agent-builtin-directory` |

### 8.3 `oclive_kernel_runtime` 默认能力与 Cargo feature

| Feature | 含义 |
|---------|------|
| `full` | 官方默认组合（含各 `default-*-providers` 等与 HTTP/ZIP/市场等，见 crate `Cargo.toml`） |
| `default-memory-providers` | 进程内记忆 builtin（`oclive_memory_builtin`） |
| `default-emotion-providers` | 进程内用户句情绪 builtin |
| `default-complex-emotion-providers` | 进程内复杂情感 builtin |
| `default-event-providers` | 编译 `event_impact_ai` + `BuiltinEventEstimator*` |
| `default-prompt-providers` | 链入 **`oclive_prompt_builtin/providers`**（`PromptBuilder` + `BuiltinPromptAssembler*`；正文在设施 crate） |
| `default-agent-providers` | 进程内 `BuiltinReActAgent`（`oclive_agent_builtin`） |
| `kernel-agent` | MCP 栈、`McpShellAgent`、Remote Agent HTTP 等 |

各模块关闭默认实现时的桩：**`crates/oclive_kernel_runtime/src/domain/disabled_default_providers.rs`**。

详细裁剪说明：**[LIGHTWEIGHT_PROFILE.md](./LIGHTWEIGHT_PROFILE.md)**。
