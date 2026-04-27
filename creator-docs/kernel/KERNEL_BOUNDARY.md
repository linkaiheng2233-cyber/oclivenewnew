# 内核/发行版边界（Kernel / Distribution Boundary）

> 版本：v0.2（可执行基线）  
> 生效范围：oclivenewnew 仓库（`src-tauri/`）  
> 维护者：按 `handoff/WEEKLY_DEV_GUIDE.md` 节奏更新

---

## 0. Kernel Baseline v1（建议冻结对象）

本仓库以 “Linux 设计哲学” 演进：**Kernel 只提供最小可信闭环 + 标准化可替换点**，发行版负责 UI/分发/体验。

- **Kernel Baseline v1 文档（建议作为对外/验收基线）**：
  - **[KERNEL_BASELINE_V1.md](./KERNEL_BASELINE_V1.md)**

---

## 1. 术语定义

| 术语 | 含义 |
|------|------|
| **内核（core）** | 平台无关的领域逻辑与调度。不依赖 Tauri、OS 窗口、快捷键、渲染。可独立编译为库（`oclive_core`）。 |
| **发行版（distribution）** | 依赖特定平台的适配层：Tauri 桌面端、VSCode 扩展、CLI、HTTP API。 |
| **适配器（adapter）** | 连接“内核能力”与“发行版传输/UI”的薄层（如 `invoke` handler → domain call）。 |
| **OOCP** | OClive Open Control Protocol — 内核对外暴露的统一能力面（方法 + 事件 stream）。 |

---

## 2. 内核包含（永久保留于 `oclive_core`）

以下领域逻辑属于内核，**不依赖** Tauri / 操作系统 / 窗口：

### 2.1 对话调度与编排
- **主入口**：`process_message`（`src-tauri/src/domain/chat_engine/mod.rs`）
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

### 2.5 OOCP 协议面
- 方法路由（method → domain call map）
- capabilities 声明
- 事件流定义
- 请求/响应/事件 schema（OOCP types）

### 2.6 MCP Client 协议面
- MCP server 发现（`mcp-servers/*.json`）
- tool 列表 / 调用
- 与 Agent 的集成

### 2.7 持久化接口（Repository trait）
- `MemoryRepository`：长期/短期记忆读写，检索排序
- `FavorabilityRepository`：好感度、关系阶段、身份维度
- `EventRepository`：事件记录
- 角色运行时（`role_runtime`）：场景、情绪、人格向量/档案、交互模式
- 数据库表结构以 `src-tauri/migrations/001_init.sql` 为准

### 2.8 业务引擎（engines/analyzers）
- 情绪分析：`user_emotion_analyzer`、`emotion_analyzer`、`complex_emotion`
- 性格引擎：`personality_engine`、`profile_personality`、`mutable_profile_llm`
- 关系引擎：`relation_engine`（好感阶段判定）
- 事件引擎：`event_detector`、`event_estimator`、`event_impact_ai`
- 记忆引擎：`memory_engine`、`memory_retrieval`
- Prompt：`prompt_builder`、`prompt_assembler`
- 策略：`policy`、`affect_policy`
- Agent：`agent`

---

## 3. 发行版包含（不进入 `oclive_core`）

以下属于发行版适配层，依赖具体平台：

### 3.1 Tauri 桌面端
- UI（Vue 3 前端）
- 窗口管理、快捷键（`hotkeys`）
- `tauri::generate_handler!` 注册的 invoke 命令（`src-tauri/src/api/*.rs`）
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
以下接口在 v0.x 期间可能调整，但**必须同步更新此文档与 OOCP spec**：
- 所有 domain trait 签名
- DTO 字段（以 `src-tauri/src/models/dto.rs` 为准）
- Repository trait 方法签名
- PluginBackends 枚举与路由逻辑

### 4.2 v1.0 冻结（计划冻结）
以下对象在 v1.0 发布后进入 **Deprecation + 迁移周期**：
- OOCP `capabilities` 版本号与语义
- OOCP 方法名（`session.create` / `chat.send_message` 等）
- 事件类型与 payload schema
- 数据库 schema（`migrations/`，仅允许 ALTER TABLE ADD COLUMN）
- DTO `reply` 字段名（**永不改名**）

---

## 5. 代码分层最小骨架

当前 `src-tauri/src/domain/` 的文件将逐步归入以下两级：

```
src-tauri/src/domain/
├── core/                    ← 内核：平台无关
│   ├── mod.rs
│   ├── chat_engine/         → 从 domain/ 迁入
│   ├── personality_engine.rs
│   ├── emotion_analyzer.rs
│   ├── ...
│   ├── repository.rs        → trait 定义保持在 core
│   └── plugin_host.rs
│
├── adapters/                ← 发行版适配：每个适配一个文件/子目录
│   ├── mod.rs
│   ├── tauri_invoke.rs      → Tauri invoke → core 映射
│   └── oocp_transport.rs    → OOCP 传输无关 handler
│
└── (其余文件暂时保持原位，随迁移逐步归入上述目录)
```

当前阶段 **仅建立目录与模块声明**，不强制一次性搬迁所有文件。
每迁移一个文件，必须补编译验证（`cargo build`）+ 相关测试。

---

## 6. 内核入口清单（当前对外能力）

此节列出当前通过 Tauri invoke 对外暴露的所有命令名、输入/输出 DTO 及事件/stream。
所有 OOCP 方法请参见 `creator-docs/oocp/OOCP_SPEC_v0_1.md`。

详见随附文档：**[KERNEL_ENTRY_CHECKLIST.md](./KERNEL_ENTRY_CHECKLIST.md)**

---

## 7. 禁止事项（硬约束）

- ❌ 内核代码不得 `use tauri::*` 或依赖 `tauri` crate
- ❌ 内核代码不得访问 `AppHandle`、`Window`、`Manager`
- ❌ DTO 字段 `reply` 不得改名
- ❌ 数据库表不得虚构（以 `migrations/001_init.sql` 为准）
- ❌ 不得在 API 层（`src-tauri/src/api/*.rs`）编写业务逻辑
