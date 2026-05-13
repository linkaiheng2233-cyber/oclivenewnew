# Kernel V2：极薄内核与默认实现分层

> 状态：**阶段 5 主体已落地**；**6-1** models、**6-2** 事件与 **6-3** Prompt 门面已最小稳定（**§6.0 / §6.4–6.5**）。**阶段 7-1**：Prompt 正文已迁入 **`oclive_prompt_builtin`**（见 §4 / §6.2 / §6.6）。**阶段 7-2 / 7 收尾（已完成算法下沉）**：**`EventEstimator` trait** 在 **`oclive_kernel_core`**；**`event_impact_ai` / `EventDetector`** 与 JSON/人格轴辅助已迁入 **`oclive_event_builtin`**；**`oclive_kernel_runtime`** 仅保留 **`KernelEventImpactEngine`** 桥与 `domain` **re-export**（路径兼容）。**§6.7** 保留历史依赖评估结论（结论已随下沉更新，见表与 §6.8）。  
> Baseline v1 契约与测试须保持向后兼容；物理拆分以独立提交推进。

## 1. 目标

- **极薄核心（`oclive_kernel_core`）**：协议与「宪法」级抽象——共享错误、核心 DTO/模型、数据访问 **trait**、后续逐步迁入的调度与 OOCP 路由等（按依赖顺序）。
- **默认实现（`oclive_kernel_runtime`）**：builtin 引擎、SQLite、`Ollama`/云端 LLM、远程/目录插件、市场同步、角色包 ZIP 等；通过 **`PluginBackends`** 的 `builtin` / `remote` / `directory` 与 **Cargo feature** 与核心衔接。
- **兼容**：`oclive_kernel_runtime` 的 **`full`** 继续代表官方「全功能」体验，等价于当前 V1 发行版能力组合。

## 2. 特性与「极薄」模式（规划）

| 组合 | 含义 |
|------|------|
| `oclive_kernel_runtime` **`full`**（默认） | 与现网 V1 一致：HTTP/OOCP、ZIP、市场、Agent 等。 |
| `default-features = false` + 按需子特性 | 裁剪默认实现；长期目标为**仅核心调度 + 必选协议**的极简运行时（阶段 3–4 用独立 `default-*-providers` feature 细化）。 |

## 3. 阶段与仓库映射（摘要）

| 阶段 | 内容 |
|------|------|
| 1 | 本文档 + 边界说明（ ongoing ） |
| 2 | 新建 `oclive_kernel_core`，迁入 `AppError`、`Memory` / `MemoryContext`、`repository` traits；`runtime` 再导出 |
| 3 | 以 **LLM** 为试点：`default-llm-providers`（默认开），关则仅保留 trait/远程或目录后端 |
| 4 | Memory / Emotion / Event / Prompt 等 **default-\*-providers** |
| 5 | `default-*-providers` 细化：**官方默认模块**（memory / emotion / complex_emotion / **agent ReAct**；工程名 **设施 crate** / `oclive_*_builtin`）+ directory 示例；`kernel-agent` 与 **`default-agent-providers`** 分离（MCP 基础 vs 进程内 Builtin ReAct）；极薄 `cargo check -p oclive_kernel_runtime --no-default-features` |
| 7（收尾） | **事件**：**`oclive_event_builtin`** 含 **`EventDetector` + `event_impact_ai`** 与 **`BuiltinEventEstimator*`**；runtime **`KernelEventImpactEngine`** 委托 **`oclive_event_builtin::estimate_event_impact`**（见 §6.8） |

## 4. 官方默认模块与 default-*-providers（阶段 5 快照）

**产品术语**：随官方发行版提供、经 Cargo feature 可选链接的 **进程内 Builtin 实现**，统称为 **「官方默认〈领域〉模块」**（例：**官方默认记忆模块** ↔ `oclive_memory_builtin`）。README、PR 与 Cargo 讨论中仍可简称 **设施 crate** 或 **`*_builtin` crate**。**第九模块（专家模型设施）** 是内核托管的 ExpertGraph / 侧车装配等，**不是**本节的「官方默认××模块」，见 [MODULE_9_EXPERT_MODELS_FACILITY.md](./MODULE_9_EXPERT_MODELS_FACILITY.md)。

| 官方默认模块 | Crate | `default-*-providers` | runtime 中保留的薄层 / 基础能力 |
|--------------|-------|----------------------|----------------------------------|
| 官方默认记忆模块 | `oclive_memory_builtin` | `default-memory-providers` | `classic` 再导出；directory 示例 `memory.rank` |
| 官方默认情绪模块 | `oclive_emotion_builtin` | `default-emotion-providers` | `EmotionAnalyzer` / `EmotionResultExt`；`emotion.analyze` 示例 |
| 官方默认复杂情感模块 | `oclive_complex_emotion_builtin` | `default-complex-emotion-providers` | `affect_metrics_from_seven_dim`；`complex_emotion.resolve_turn` 示例 |
| 官方默认 Agent 模块 | `oclive_agent_builtin` | `default-agent-providers` | **`McpShellAgent`** 仍在 runtime；ReAct 在本 crate；`agent.process` 示例（契约演示） |
| 官方默认 Prompt 模块 | `oclive_prompt_builtin` | `default-prompt-providers` | `PromptBuilder`（`classic`）；`BuiltinPromptAssembler*`（`providers`）；runtime 保留槽位 / Remote 占位 / HTTP；directory 示例 `prompt.build_prompt` |
| 官方默认事件模块 | `oclive_event_builtin` | `default-event-providers` | **`EventEstimator` trait** 在 **`oclive_kernel_core`**；**算法在设施 crate**；runtime 仅 **`KernelEventImpactEngine` + re-export**（槽位接线） |

**Agent**：`kernel-agent` 控制 MCP 栈与轻量 **`McpShellAgent`**；**`default-agent-providers`** 单独控制是否链接 **`BuiltinReActAgent`**（`oclive_agent_builtin/providers`）。

**Prompt**：**`default-prompt-providers`** 链入 **`oclive_prompt_builtin/providers`**（隐含 **`classic`**）；关闭时 **`DisabledPromptAssembler`**。目录形态见 **`examples/oclive-prompt-builtin-directory/`** 与 **`LIGHTWEIGHT_PROFILE.md`**。

## 5. 已实现（阶段 2 首包）

- Crate：`crates/oclive_kernel_core`
- 已迁入：`AppError` / `Result`、`models::memory`、`repository`（`MemoryRepository`、`FavorabilityRepository`、`ExpertModelsRepository`）
- `oclive_kernel_runtime` 对上述路径 **再导出**；移除 `tauri_invoke`（`AppError` 外置后不再在 runtime 内提供 `From<AppError> for InvokeError`，桌面显式 `to_frontend_error()`）。

## 6. 阶段 6 待定 — 事件与 Prompt 剥离评估（阶段 5-6 / 5-7 结论）

以下为 **2026-05** 起依赖扫描结论：**官方默认 Prompt 模块**（**`oclive_prompt_builtin`**）已在 **阶段 7-1** 落地（见 §4、§6.2）。**官方默认事件模块**（**`oclive_event_builtin`**）在 **阶段 7-2 收尾** 已迁入 **`EventDetector` / `event_impact_ai`（`estimate_event_impact` 等）**；**`oclive_kernel_runtime`** 保留 **`KernelEventImpactEngine`**、`domain::event_detector` **re-export** 与 **`#[cfg(feature = "default-event-providers")]`** 下的薄 **`event_impact_*` 模块**（见 §6.8、**`examples/oclive-event-builtin-directory/`**）。§6.7 表格保留当时 **runtime 内聚**评估的**归档**表述，**现行路径以源码与 §6.8 为准**。

### 6.0 阶段 6-1 固化：`oclive_kernel_models` 迁入清单

以下类型已迁至 **`crates/oclive_kernel_models`**（纯数据：无 `*Engine` / `*Repository` / `PluginHost` / I/O）。`oclive_kernel_runtime` 通过 `pub use oclive_kernel_models::...` 薄再导出，既有 `crate::models::*` 路径可保持不变。

| 类型 | 原 `kernel_runtime` 位置 | 现定义位置 |
|------|--------------------------|------------|
| `EventType`、`Event` | `src/models/event.rs`（曾本地定义） | `kernel_models::event` |
| `PersonalityVector`（含 `clamp` / `effective_from_core_delta` / `to_json_vec` 等） | `src/models/personality.rs` | `kernel_models::personality` |
| `PersonalityDefaults`、`EvolutionBounds`、`EvolutionConfig`、`MemoryConfig`、`UserRelation` | `src/models/role.rs`（与 `Role` 混排） | `kernel_models::role_config` |
| `KnowledgeEventAugment` | `src/models/knowledge.rs`（曾与 `KnowledgeIndex` / merge 逻辑混写） | `kernel_models::knowledge_augment` |
| `EventImpactEstimate` | `domain/event_impact_ai.rs`（曾与 LLM/规则算法混写） | `kernel_models::event_impact`（**阶段 6-2**） |
| `PromptRolePromptSlice` | （原内嵌于 `Role` + `PromptBuilder` 读字段） | `kernel_models::prompt_role`（**阶段 6-3**） |

**刻意未迁入 models**：完整 **`Role`**（磁盘 / 运行时宿主模型）、**`PluginBackends`**、知识索引等仍驻 **`kernel_runtime`**。**`PromptInput`** 的 **契约 DTO** 在 **`oclive_kernel_core::prompt`**（见 **§6.5**）；`role_any` 承载 **`&Role` 的 `dyn Any`** 仅供侧车序列化向下转型。

#### `kernel_core` 与 `oclive_kernel_models`

| Crate | 职责 | 依赖关系 |
|-------|------|----------|
| `oclive_kernel_models` | 共享 **纯数据**（事件 / 性格向量 / 进化与用户关系片段 / 知识增强 / 事件估计 DTO） | **不**依赖 `kernel_core` |
| `oclive_kernel_core` | `AppError`、`Emotion`、`Memory`、`LlmClient`、各类 **trait**、**`PromptInput` / `PromptAssembler`**、仓库端口 | **阶段 6-1**：不依赖 `kernel_models`。**阶段 6-2 起**：依赖 `kernel_models`（`EventEstimator`、`PromptAssembler` 签名引用的纯数据）；**仍不**依赖完整 runtime |
| `oclive_kernel_runtime` | 内置引擎、存储、HTTP、插件宿主 | 依赖 **core + models** |

### 6.1 事件（`EventEstimator` / `event_impact_ai`）

| 类别 | 依赖项 |
|------|--------|
| **门面** | **`EventEstimator` trait**（**`oclive_kernel_core::event_estimator`**，**阶段 6-2**）；**`EventImpactEstimate`**（**`oclive_kernel_models::event_impact`**）；runtime **`domain/event_estimator.rs`** 仅保留内置类型、`default_event_slot_*`、`RemoteEventEstimatorPlaceholder` 并再导出 trait |
| **算法主体** | **`oclive_event_builtin::estimate_event_impact`** 等 → LLM `generate_tag` + JSON 解析（runtime **`KernelEventImpactEngine`** 委托） |
| **规则回退与检测** | **`EventDetector`**（**`oclive_event_builtin`**；runtime **`domain/event_detector.rs`** 仅 **re-export**） |
| **人格公式** | 影响链内公式已随 **`event_impact_ai`** 迁入 **`oclive_event_builtin`**（如 **`affect_axis`**）；runtime **`affect_policy`** 等仍服务其它编排路径 |
| **工具** | **`oclive_event_builtin::json_loose`**（`extract_json_object`） |
| **外部抽象** | `LlmClient`（已在 core） |
| **模型 / DTO** | `Emotion`（core）；`Event` / `EventType`、`PersonalityVector`、`KnowledgeEventAugment`、`EventImpactEstimate`（**models**，runtime 再导出）；`PersonalitySource`（**`oclive_validation`**） |

**结论（阶段 7-2 收尾后）**：**`BuiltinEventEstimator*`** → **`EventImpactEngine`** → **`KernelEventImpactEngine`** → **`oclive_event_builtin::estimate_event_impact`**。**`KnowledgeEventAugment` 等 DTO** 在 **models**；与知识索引管线的耦合留在 **编排 / 调用方**，与 Agent/MCP **无硬耦合**（仅需 `LlmClient`）。**`default-event-providers`** 控制 **Builtin 槽与 bridge 模块**（关则 **`DisabledEventEstimator`**，runtime **`event_impact_ai` / `event_impact_bridge` 不编译**；**`EventDetector` re-export** 仍可用，见 **`domain/mod.rs`**）。

### 6.2 Prompt（`PromptAssembler` / `PromptBuilder`）— **阶段 7-1 已完整剥离设施 crate**

| 类别 | 依赖项 |
|------|--------|
| **门面** | **`PromptAssembler` trait**（**`oclive_kernel_core::prompt`**，**阶段 6-3**）；runtime **`domain/prompt_assembler.rs`** 保留 **`default_prompt_slot_*`**、**`RemotePromptAssemblerPlaceholder`**；进程内 **`BuiltinPromptAssembler*`** 由 **`oclive_prompt_builtin`**（`providers`）提供 |
| **输入 DTO** | **`PromptInput<'a>`**（**core**：`role_any` + **`PromptRolePromptSlice`** + 记忆 / 场景 / 事件等字段）；**`DEFAULT_REPLY_QUALITY_ANCHOR`**、**`effective_reply_quality_anchor`**（core） |
| **实现** | **`PromptBuilder`**（**`oclive_prompt_builtin`**，`classic` / `classic/stub`）；**`#[cfg(feature = "default-prompt-providers")]`** 链入 **`oclive_prompt_builtin/providers`** |
| **模型** | `Memory`（core）；`PersonalityVector`、`EventType`、`TopicHintContext`、`PromptRolePromptSlice`（models）；完整 **`Role`**（runtime，`prompt_slice()` / `topic_hint_context()`）；侧车 JSON 仍序列化完整 **`Role`** |

**结论（阶段 7-1）**：**`top_topic_hint`** 仅依赖 **`TopicHintContext`**（models）；**`build_prompt`** 主路径仅依赖 **`PromptInput` + models/core 类型**，已无对完整 **`Role`** 类型的直接引用。关闭 **`default-prompt-providers`** 时 **`DisabledPromptAssembler`**；可通过 **directory** + **`examples/oclive-prompt-builtin-directory/`** + **`oclive_prompt_from_json`** 恢复与内置一致的侧车正文。

### 6.3 验收（当前阶段）

- **`cargo check --workspace`** / **`cargo test --workspace`** 通过；极薄检出：**`cargo check -p oclive_kernel_runtime --no-default-features`**。
- **`default-prompt-providers`**：**`full`** 下为 **`["oclive_prompt_builtin/providers"]`**；**开启**时链入设施 crate **`providers`**（含 **`PromptBuilder` 全文**）；**关闭**时 runtime **不**链入 **`providers`**，builtin 槽与 Remote 占位回退 **`DisabledPromptAssembler`**。
- **`default-event-providers`**：行为同 **§6.4**（关闭时 **不编译** runtime **`domain::event_impact_ai` / `event_impact_bridge`**，**`DisabledEventEstimator`**；**`oclive_event_builtin`** 仍为直接依赖，供 **`EventDetector` re-export** 等路径）。

### 6.4 阶段 6-2：事件门面最小切线（已实现）

- **`EventEstimator`**：`oclive_kernel_core::event_estimator`（`async_trait` + `LlmClient` + models DTO + `PersonalitySource`）。
- **`EventImpactEstimate`**：`oclive_kernel_models::event_impact`（serde，Remote JSON-RPC 与内置共用）。
- **`BuiltinEventEstimator` / `BuiltinEventEstimatorV2`**：仍在 **`oclive_kernel_runtime::domain::event_estimator`**，整段置于 **`#[cfg(feature = "default-event-providers")]`**；算法委托 **`oclive_event_builtin::estimate_event_impact`**（经 **`KernelEventImpactEngine`**）。
- **`domain::event_impact_ai` / `event_impact_bridge`**：仅在该 feature **开启**时编译（`domain/mod.rs` 条件模块）；与 **`estimate_event_impact`** 实现本体（**`oclive_event_builtin`**）分离。
- **桩**：**`DisabledEventEstimator`**（`disabled_default_providers.rs`），feature 关闭时由 **`default_event_slot_v1/v2`** 选用。

### 6.5 阶段 6-3：Prompt 门面最小切线（已实现）→ **阶段 7-1：正文迁入 `oclive_prompt_builtin`**

- **`PromptAssembler`**：`oclive_kernel_core::prompt`（**`build_prompt` + `top_topic_hint(&TopicHintContext, …)`**；编排层从 `Role` 提取 **`TopicHintContext`**，见 **`oclive_kernel_models::TopicHintContext`**）。
- **`PromptInput`**： **`role_any`**（侧车 **`downcast_ref::<Role>`** 序列化）+ **`role_prompt: PromptRolePromptSlice`**。
- **`PromptRolePromptSlice`**：`kernel_models::prompt_role`；**`Role::prompt_slice()`**（runtime）填充。
- **`PromptBuilder` / `BuiltinPromptAssembler*`**：在 **`oclive_prompt_builtin`**（**`default-prompt-providers`** → **`providers`**）；runtime **`domain/prompt_builder.rs`** 仅再导出 **`PromptInput`** 等与 **`PromptBuilder`**（feature gate）；单测仍在 runtime，受 **`default-prompt-providers`** 约束。
- **桩**：**`DisabledPromptAssembler`**（**`MODULE_NONE_SEMANTICS.md`** 与 **`NonePromptAssembler`** 仍由 runtime 提供）。

### 6.6 阶段 6 总览（定型）

| 模块 | Trait / 核心 DTO 基座 | `default-*-providers` 关闭时的行为 | 算法 / builtin 主体位置 |
|------|----------------------|-----------------------------------|-------------------------|
| Memory | `MemoryRetrieval`（core） | `DisabledMemoryRetrieval` | **官方默认记忆模块** `oclive_memory_builtin` |
| User 情绪 | `UserEmotionAnalyzer`（core） | `DisabledUserEmotionAnalyzer` | **官方默认情绪模块** `oclive_emotion_builtin` |
| 复杂情感 | `ComplexEmotionProvider`（core） | `DisabledComplexEmotionProvider` | **官方默认复杂情感模块** `oclive_complex_emotion_builtin` |
| 事件 | **`EventEstimator`**（core）+ **`EventImpactEstimate`**（models） | **`DisabledEventEstimator`**；**不编译** runtime **`event_impact_ai` / `event_impact_bridge`** | **`EventDetector` / `event_impact_ai`**（**`oclive_event_builtin`**）；runtime **桥 + re-export** |
| Prompt | **`PromptAssembler`**（core）+ **`PromptInput`** / **`PromptRolePromptSlice`** | **`DisabledPromptAssembler`**；**不链入 `oclive_prompt_builtin/providers`** | **官方默认 Prompt 模块** **`oclive_prompt_builtin`**（`PromptBuilder` + `BuiltinPromptAssembler*`） |
| Agent | **`AgentProvider`**（core） | **`NoopAgent`** / **`McpShellAgent`**（视 **`kernel-agent`**） | **官方默认 Agent 模块** `oclive_agent_builtin`（`BuiltinReActAgent`） |

**`full`**（默认）仍为官方一体化能力组合；嵌入式 / SKU 使用 **`default-features = false`** 并按 **[LIGHTWEIGHT_PROFILE.md](./LIGHTWEIGHT_PROFILE.md)** 开启子特性。

**后续（非承诺）**：继续下沉 **`Role` / `PromptInput` 字段** 等——须在 **`oclive_validation`** 与 OOCP 契约侧同步版本策略。

### 6.7 阶段 6 后续 / 阶段 7：事件 **完整剥离**可行性评估 + Prompt **现状**（2026-05，**事件算法已下沉；本节为归档**）

> **现行实现**：**`EventDetector` + `event_impact_ai`** 已在 **`oclive_event_builtin`**（见 §6.8）。  
> **历史目的**：在 trait / DTO 已稳定（§6.4–6.5）的前提下，评估 **`BuiltinEventEstimator*` + `event_impact_ai` 全链路** 与 **`BuiltinPromptAssembler*` + `PromptBuilder` 全链路** 是否迁入独立设施 crate。**Prompt** 已在 **阶段 7-1** 迁入 **`oclive_prompt_builtin`**（见 §6.2）；以下 **6.7.1 / 6.7.4** 表格以 **当时「仍在 runtime」的文件划分** 为主，**便于对照迁移工作量**；**路径与模块边界以当前源码为准**。

#### 6.7.1 归档：迁移前 **Runtime 内**事件相关实现清单（**多数已迁至 `oclive_event_builtin`**）

| 领域 | 原文件 / 现况（`oclive_kernel_runtime/src/domain/` 为主） | 职责摘要 | 关键符号 |
|------|---------------------------------------------------|----------|----------|
| **事件 · 门面与 Builtin 槽** | `event_estimator.rs`（**仍在 runtime**） | 再导出 `EventEstimator`；**`BuiltinEventEstimator` / `BuiltinEventEstimatorV2`**；`default_event_slot_*`；**`RemoteEventEstimatorPlaceholder`** | `BuiltinEventEstimator::estimate` → **`oclive_event_builtin::estimate_event_impact`** |
| **事件 · LLM + 规则主体** | **`oclive_event_builtin/src/event_impact_ai.rs`**（runtime **`event_impact_ai.rs`** 为 **re-export / 条件模块**） | `estimate_event_impact`、JSON 解析、LLM 失败回退 | `estimate_event_impact`、`parse_event_impact_ai_output`、`event_impact_ai_enabled` |
| **事件 · 规则检测** | **`oclive_event_builtin/src/event_detector.rs`**（runtime **`event_detector.rs`** **re-export**） | 关键词 / 情绪组合分类；**`KnowledgeEventAugment`** 合并 | `EventDetector::detect_with_augment` 等 |
| **事件 · 人格轴辅助** | **`oclive_event_builtin`**（如 **`affect_axis`**）；runtime **`affect_policy`** 仍服务其它路径 | impact 链内公式已随设施 crate 下沉 | （归档）曾列 `affect_policy` / `personality_engine` |
| **事件 · 工具** | **`oclive_event_builtin/src/json_loose.rs`** | `extract_json_object` | 被 **`event_impact_ai`** 使用 |
| **Prompt · 集成薄层（已迁出正文）** | `prompt_assembler.rs`、`prompt_builder.rs` | **`BuiltinPromptAssembler*`** / **`PromptBuilder`** 由 **`oclive_prompt_builtin`** 提供；runtime 保留 **`default_prompt_slot_*`**、**`RemotePromptAssemblerPlaceholder`**、**`PromptInput` 再导出** | 侧车 **`prompt_http.rs`** 仍序列化完整 **`Role`**；**`oclive_prompt_from_json`** 仅需 JSON 字段子集即可拼装（见示例目录 README） |

**其它引用（非算法主体，但构成集成面）**：

- **`PluginHost`**（`domain/plugin_host.rs`）：仅持有 **`Arc<dyn EventEstimator>`**、**`Arc<dyn PromptAssembler>`**，按 `PluginBackends` 分发；**不**直接引用 `EventImpactEstimate` 类型名，返回值由 trait（定义于 **`oclive_kernel_core`**）约束。
- **Remote HTTP**：`infrastructure/remote_plugin/event_http.rs`、`prompt_http.rs` 实现侧车 JSON-RPC，失败时回退 **`default_*_slot_v1`**。
- **编排**：`chat_engine::process_message` 等通过 **`ResolvedRolePlugins`** 调用 `pl.event` / `pl.prompt`，与剥离目标解耦。

#### 6.7.2 `EventImpactEstimate` 与 PluginHost — 依赖结论

| 检查项 | 结论 |
|--------|------|
| **DTO 位置** | **`EventImpactEstimate`** 已在 **`oclive_kernel_models::event_impact`**（§6.0）；Remote / Builtin 共用。 |
| **`EventEstimator` trait** | 在 **`oclive_kernel_core::event_estimator`**，签名已使用 **`kernel_models`** 中的 `Event`、`KnowledgeEventAugment`、`PersonalityVector` 等；**不**依赖 runtime。 |
| **PluginHost** | 仅按枚举绑定 **`Arc<dyn EventEstimator>`**；**无**对 `EventImpactEstimate` 的额外硬编码路径。 |
| **可行性含义** | **共享类型与宿主注册方式已解耦**。（归档）当时阻碍在 **算法模块之间的 runtime 内聚**；**事件链已迁入 `oclive_event_builtin`**（见 §6.8）。 |

#### 6.7.3 `PromptInput` / `Role` — 依赖结论（**阶段 7-1 已落实设施 crate**）

| 检查项 | 结论 |
|--------|------|
| **`PromptInput` / `PromptRolePromptSlice`** | 契约在 **`oclive_kernel_core::prompt`** + **`kernel_models::prompt_role`**（§6.5）；**`build_prompt` 主路径仅依赖 DTO + `Memory`（core）+ models 枚举**。 |
| **`Role` 耦合点** | 话题提示仅 **`TopicHintContext`**（编排层由 **`Role::topic_hint_context()`** 填充）。**`role_any`** 仍为侧车 **`downcast_ref::<Role>`** 序列化保留；**非 `build_prompt` 算法主路径依赖**。 |
| **阶段 7-1 结果** | **`oclive_prompt_builtin`** 实现 **`PromptBuilder` + `BuiltinPromptAssembler*`**；关闭 **`default-prompt-providers`** 时 **`DisabledPromptAssembler`**；directory 示例 + **`oclive_prompt_from_json`** 可恢复正文。 |

#### 6.7.4 综合评估表（是否进入阶段 7 实作）

| 剥离目标 | 阶段 7 结论 | 可行性分析摘要 |
|----------|-------------|----------------|
| **事件：`BuiltinEventEstimator*` 壳层** | **阶段 7 可执行（低价值）** | 壳层仅委托 `estimate_event_impact`；单独迁壳**无独立意义**，通常与算法同迁。 |
| **事件：`estimate_event_impact` + `EventDetector` + 人格轴辅助 + json 工具** | **阶段 7 已完成** | 已整体迁入 **`oclive_event_builtin`**；runtime **`KernelEventImpactEngine`** 单向委托，**无** `runtime ↔ event_builtin` 循环依赖。 |
| **Prompt：`BuiltinPromptAssembler*` + `PromptBuilder` 全文** | **阶段 7-1 已完成** | 已迁入 **`oclive_prompt_builtin`**（`providers` + `classic`）；runtime 薄层 + **`DisabledPromptAssembler`** 门控不变。 |
| **PluginHost / trait / DTO** | **已就绪（非阻碍）** | 注册与类型边界已满足独立 Builtin crate 接入形态。 |

**小结**：**`oclive_prompt_builtin` 已落地**。**事件**：**`oclive_event_builtin` 已承载算法主体**；runtime 保留 **槽位、`KernelEventImpactEngine`、条件编译的薄模块与 `EventDetector` re-export**（见 §6.8）。

#### 6.7.5 事件：`EventEstimator` / `EventImpactEstimate` 边界（阶段 7-2 固化）

- **Trait / DTO**：**`EventEstimator`** 在 **`oclive_kernel_core::event_estimator`**；**`EventImpactEstimate`** 在 **`oclive_kernel_models::event_impact`**；签名与 serde 已稳定，trait **不**依赖完整 runtime。
- **`DisabledEventEstimator`**（**`default-event-providers` 关闭**）：返回 **`EventType::Ignore`**、`impact_factor = 0`、`confidence = 0`，与 **`MODULE_NONE_SEMANTICS.md` §3**（「无事件、无影响」）及好感/演化链路的「按无事件继续」预期一致；见 **`disabled_default_providers`** 单测 **`disabled_event_estimator_returns_ignore_zero_impact`**。
- **算法主体**：**`event_impact_ai` / `EventDetector`** 与相关工具在 **`oclive_event_builtin`**；runtime **§6.7.1** 表格为**归档**对照。

### 6.8 阶段 7 剥离状态快照（维护）

| 领域 | 设施 crate | runtime 残留 | 目录示例 |
|------|------------|--------------|----------|
| Memory | `oclive_memory_builtin` | 槽位 / HTTP / 引擎接线 | `examples/oclive-memory-builtin-directory` |
| User 情绪 | `oclive_emotion_builtin` | 同上 | `examples/oclive-emotion-builtin-directory` |
| 复杂情感 | `oclive_complex_emotion_builtin` | 同上 | `examples/oclive-complex-emotion-builtin-directory` |
| **Prompt** | **`oclive_prompt_builtin`** | **`prompt_assembler` 槽位**、`RemotePromptAssemblerPlaceholder`、`prompt_http`；**`prompt_builder` 再导出** | **`examples/oclive-prompt-builtin-directory`**（依赖 **`oclive_prompt_from_json`**） |
| Agent ReAct | `oclive_agent_builtin` | `McpShellAgent`、MCP 栈 | `examples/oclive-agent-builtin-directory` |
| **事件** | **`oclive_event_builtin`（`EventDetector` / `event_impact_ai` / `BuiltinEventEstimator*`）** | **`KernelEventImpactEngine`**、**`event_estimator` 槽位**、**`event_impact_*` 条件模块**、**`EventDetector` re-export**、Remote 占位 | **`examples/oclive-event-builtin-directory/`**（directory **`event.estimate`**） |

**验证**：**`cargo check --workspace`**、**`cargo test --workspace`**、**`cargo check -p oclive_kernel_runtime --no-default-features`**；可选组合见 **`LIGHTWEIGHT_PROFILE.md`** §6。

## 7. 参考

- [KERNEL_BOUNDARY.md](./KERNEL_BOUNDARY.md)
- [KERNEL_BASELINE_V1.md](./KERNEL_BASELINE_V1.md)
- [LIGHTWEIGHT_PROFILE.md](./LIGHTWEIGHT_PROFILE.md)
