# Kernel V2：极薄内核与默认实现分层

> 状态：**阶段 5 主体已落地**；**6-1** models、**6-2** 事件与 **6-3** Prompt 门面已最小稳定（**§6.0 / §6.4–6.5**）。**§6.7** 为阶段 6 后续——事件 / Prompt **完整剥离**可行性结论（是否进入阶段 7 实作）。  
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

## 4. 官方默认模块与 default-*-providers（阶段 5 快照）

**产品术语**：随官方发行版提供、经 Cargo feature 可选链接的 **进程内 Builtin 实现**，统称为 **「官方默认〈领域〉模块」**（例：**官方默认记忆模块** ↔ `oclive_memory_builtin`）。README、PR 与 Cargo 讨论中仍可简称 **设施 crate** 或 **`*_builtin` crate**。**第九模块（专家模型设施）** 是内核托管的 ExpertGraph / 侧车装配等，**不是**本节的「官方默认××模块」，见 [MODULE_9_EXPERT_MODELS_FACILITY.md](./MODULE_9_EXPERT_MODELS_FACILITY.md)。

| 官方默认模块 | Crate | `default-*-providers` | runtime 中保留的薄层 / 基础能力 |
|--------------|-------|----------------------|----------------------------------|
| 官方默认记忆模块 | `oclive_memory_builtin` | `default-memory-providers` | `classic` 再导出；directory 示例 `memory.rank` |
| 官方默认情绪模块 | `oclive_emotion_builtin` | `default-emotion-providers` | `EmotionAnalyzer` / `EmotionResultExt`；`emotion.analyze` 示例 |
| 官方默认复杂情感模块 | `oclive_complex_emotion_builtin` | `default-complex-emotion-providers` | `affect_metrics_from_seven_dim`；`complex_emotion.resolve_turn` 示例 |
| 官方默认 Agent 模块 | `oclive_agent_builtin` | `default-agent-providers` | **`McpShellAgent`** 仍在 runtime；ReAct 在本 crate；`agent.process` 示例（契约演示） |

**Agent**：`kernel-agent` 控制 MCP 栈与轻量 **`McpShellAgent`**；**`default-agent-providers`** 单独控制是否链接 **`BuiltinReActAgent`**（`oclive_agent_builtin/providers`）。

## 5. 已实现（阶段 2 首包）

- Crate：`crates/oclive_kernel_core`
- 已迁入：`AppError` / `Result`、`models::memory`、`repository`（`MemoryRepository`、`FavorabilityRepository`、`ExpertModelsRepository`）
- `oclive_kernel_runtime` 对上述路径 **再导出**；移除 `tauri_invoke`（`AppError` 外置后不再在 runtime 内提供 `From<AppError> for InvokeError`，桌面显式 `to_frontend_error()`）。

## 6. 阶段 6 待定 — 事件与 Prompt 剥离评估（阶段 5-6 / 5-7 结论）

以下为 **2026-05** 依赖扫描结论：**不在当前阶段强制**创建 **官方默认事件 / Prompt 模块**（`oclive_event_builtin` / `oclive_prompt_builtin`），避免「为剥离而剥离」导致超大搬迁或 `runtime ↔ *_builtin` 循环依赖。先行记录迁移前置条件，待 **`oclive_kernel_models`** 或等价共享模型层就绪后再做。

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
| **算法主体** | `estimate_event_impact` → LLM `generate_tag` + JSON 解析 |
| **规则回退与检测** | `EventDetector`（`domain/event_detector.rs`，关键词 + `KnowledgeEventAugment`） |
| **人格公式** | `affect_policy::softness_coldness_volatility`、`PersonalityEngine::calculate_stability_index` |
| **工具** | `utils/json_loose::extract_json_object` |
| **外部抽象** | `LlmClient`（已在 core） |
| **模型 / DTO** | `Emotion`（core）；`Event` / `EventType`、`PersonalityVector`、`KnowledgeEventAugment`、`EventImpactEstimate`（**models**，runtime 再导出）；`PersonalitySource`（**`oclive_validation`**） |

**评估**：从 **官方默认事件能力**（当前仍在 runtime，尚无独立 `oclive_event_builtin`）视角，至少牵连 **规则检测、人格轴公式、JSON 工具、多种模型类型** 与 **一长串 `estimate` 参数**，独立 trait/DTO 边界 **明显超过 5**；且 **`KnowledgeEventAugment` 与知识索引管线耦合**。与 Agent/MCP **无硬耦合**（仅需 `LlmClient`）。**阶段 6-2（最小切线）**：已稳定 **trait + `EventImpactEstimate`**，并用 **`default-event-providers`** 控制 **`event_impact_ai` 模块与 `BuiltinEventEstimator*`**（关则桩 `DisabledEventEstimator`，见 **§6.4**）。整仓搬迁 `EventDetector` + `event_impact_ai` 算法主体仍为后续工作。

### 6.2 Prompt（`PromptAssembler` / `PromptBuilder`）

| 类别 | 依赖项 |
|------|--------|
| **门面** | **`PromptAssembler` trait**（**`oclive_kernel_core::prompt`**，**阶段 6-3**）；runtime **`domain/prompt_assembler.rs`** 保留 **`BuiltinPromptAssembler*`**、`default_prompt_slot_*`、`RemotePromptAssemblerPlaceholder` |
| **输入 DTO** | **`PromptInput<'a>`**（**core**：`role_any` + **`PromptRolePromptSlice`** + 记忆 / 场景 / 事件等字段）；**`DEFAULT_REPLY_QUALITY_ANCHOR`**、**`effective_reply_quality_anchor`**（core） |
| **实现** | **`PromptBuilder`**（**`domain/prompt_builder.rs`**，**`#[cfg(feature = "default-prompt-providers")]`**）；体量仍大，未搬迁 |
| **模型** | `Memory`（core）；`PersonalityVector`、`EventType`、`PromptRolePromptSlice`（models）；完整 **`Role`**（runtime，`prompt_slice()` 填充切片）；侧车 JSON 仍序列化完整 **`Role`** |

**评估**：整段 **`build_prompt`** 与 **`Role` 全字段** 仍深度耦合。**阶段 6-3（最小切线）**：已稳定 **trait + `PromptInput` + 角色 Prompt 切片**；**`default-prompt-providers`** 控制 **`PromptBuilder` 编译**，关闭时 **`DisabledPromptAssembler`**（空串 / 无 topic hint）。完整拆分为 **官方默认 Prompt 模块**（独立 `oclive_prompt_builtin` 等）仍为后续选项。

### 6.3 验收（当前阶段）

- 无新增 **官方默认模块**（独立 `*_builtin` crate）；**`cargo check --workspace`** / **`cargo test --workspace`** 通过。
- **`default-prompt-providers`**：空数组；**开启**（`full`）时编译 **`PromptBuilder`**；**关闭**时不编译 **`PromptBuilder`**，builtin 槽与 Remote 占位回退 **`DisabledPromptAssembler`**。极薄检出：**`cargo check -p oclive_kernel_runtime --no-default-features`**。
- **`default-event-providers`**：行为同 **§6.4**（关闭时不编译 **`event_impact_ai`**，**`DisabledEventEstimator`**）。

### 6.4 阶段 6-2：事件门面最小切线（已实现）

- **`EventEstimator`**：`oclive_kernel_core::event_estimator`（`async_trait` + `LlmClient` + models DTO + `PersonalitySource`）。
- **`EventImpactEstimate`**：`oclive_kernel_models::event_impact`（serde，Remote JSON-RPC 与内置共用）。
- **`BuiltinEventEstimator` / `BuiltinEventEstimatorV2`**：仍在 **`oclive_kernel_runtime::domain::event_estimator`**，整段置于 **`#[cfg(feature = "default-event-providers")]`**；算法委托 **`estimate_event_impact`**。
- **`domain::event_impact_ai`**：仅在该 feature **开启**时编译（`domain/mod.rs` 条件模块），减轻 `--no-default-features` 依赖面。
- **桩**：**`DisabledEventEstimator`**（`disabled_default_providers.rs`），feature 关闭时由 **`default_event_slot_v1/v2`** 选用。

### 6.5 阶段 6-3：Prompt 门面最小切线（已实现）

- **`PromptAssembler`**：`oclive_kernel_core::prompt`（**`build_prompt` + `top_topic_hint(role_any: &dyn Any, …)`**；内置侧将 **`role_any` 断言为 `Role`**）。
- **`PromptInput`**： **`role_any`**（侧车 **`downcast_ref::<Role>`** 序列化）+ **`role_prompt: PromptRolePromptSlice`**（**Copy**，供 **`PromptBuilder`** 读本段人设字段）。
- **`PromptRolePromptSlice`**：`kernel_models::prompt_role`；**`Role::prompt_slice()`**（runtime）填充。
- **`PromptBuilder`**：**仅**在 **`default-prompt-providers`** 开启时编译；单测 **`prompt_builder` / `prompt_assembler`** 亦受同一 feature 约束。
- **桩**：**`DisabledPromptAssembler`**。

### 6.6 阶段 6 总览（定型）

| 模块 | Trait / 核心 DTO 基座 | `default-*-providers` 关闭时的行为 | 算法 / builtin 主体位置 |
|------|----------------------|-----------------------------------|-------------------------|
| Memory | `MemoryRetrieval`（core） | `DisabledMemoryRetrieval` | **官方默认记忆模块** `oclive_memory_builtin` |
| User 情绪 | `UserEmotionAnalyzer`（core） | `DisabledUserEmotionAnalyzer` | **官方默认情绪模块** `oclive_emotion_builtin` |
| 复杂情感 | `ComplexEmotionProvider`（core） | `DisabledComplexEmotionProvider` | **官方默认复杂情感模块** `oclive_complex_emotion_builtin` |
| 事件 | **`EventEstimator`**（core）+ **`EventImpactEstimate`**（models） | **`DisabledEventEstimator`**；**不编译 `event_impact_ai`** | **`event_impact_ai`** + **`EventDetector`**（runtime；尚未拆独立 crate） |
| Prompt | **`PromptAssembler`**（core）+ **`PromptInput`** / **`PromptRolePromptSlice`** | **`DisabledPromptAssembler`**；**不编译 `PromptBuilder`** | **`PromptBuilder`**（runtime；尚未拆独立 crate） |
| Agent | **`AgentProvider`**（core） | **`NoopAgent`** / **`McpShellAgent`**（视 **`kernel-agent`**） | **官方默认 Agent 模块** `oclive_agent_builtin`（`BuiltinReActAgent`） |

**`full`**（默认）仍为官方一体化能力组合；嵌入式 / SKU 使用 **`default-features = false`** 并按 **[LIGHTWEIGHT_PROFILE.md](./LIGHTWEIGHT_PROFILE.md)** 开启子特性。

**后续（非承诺）**：按需增设 **官方默认事件 / Prompt 模块**（如 **`oclive_event_builtin`** / **`oclive_prompt_builtin`**），或继续下沉 **`Role` / `PromptInput` 字段**——须在 **`oclive_validation`** 与 OOCP 契约侧同步版本策略。

### 6.7 阶段 6 后续 / 阶段 7：事件与 Prompt **完整剥离**可行性评估（2026-05）

> 目的：在 trait / DTO 已稳定（§6.4–6.5）的前提下，判断 **`BuiltinEventEstimator*` + `event_impact_ai` 全链路** 与 **`BuiltinPromptAssembler*` + `PromptBuilder` 全链路** 是否可迁入独立设施 crate（`oclive_event_builtin` / `oclive_prompt_builtin`），**不**在本节执行代码搬迁。

#### 6.7.1 Runtime 中仍未剥离的实现清单

| 领域 | 文件（`oclive_kernel_runtime/src/domain/` 为主） | 职责摘要 | 关键符号 |
|------|---------------------------------------------------|----------|----------|
| **事件 · 门面与 Builtin 槽** | `event_estimator.rs` | 再导出 `EventEstimator`；**`BuiltinEventEstimator` / `BuiltinEventEstimatorV2`**；`default_event_slot_*`；**`RemoteEventEstimatorPlaceholder`** | `BuiltinEventEstimator::estimate` → `estimate_event_impact` |
| **事件 · LLM + 规则主体** | `event_impact_ai.rs`（**`#[cfg(feature = "default-event-providers")]`**） | `estimate_event_impact`、JSON 解析、LLM 失败回退 | `estimate_event_impact`、`parse_event_impact_ai_output`、`event_impact_ai_enabled` |
| **事件 · 规则检测** | `event_detector.rs` | 关键词 / 情绪组合分类；**`KnowledgeEventAugment`** 合并 | `EventDetector::detect_with_augment`、`get_impact_factor`、`get_confidence` |
| **事件 · 人格轴辅助** | `affect_policy.rs` | `softness_coldness_volatility`（供 impact 软化） | 被 `event_impact_ai` 调用 |
| **事件 · 稳定性指数** | `personality_engine.rs`（部分 API） | **`PersonalityEngine::calculate_stability_index`** | 被 `event_impact_ai` 调用 |
| **事件 · 工具** | `utils/json_loose.rs` | `extract_json_object` | 被 `event_impact_ai` 调用 |
| **Prompt · 门面与 Builtin 槽** | `prompt_assembler.rs` | **`BuiltinPromptAssembler` / `BuiltinPromptAssemblerV2`**；`default_prompt_slot_*`；**`RemotePromptAssemblerPlaceholder`** | `build_prompt` / `top_topic_hint` → `PromptBuilder` |
| **Prompt · 拼装主体** | `prompt_builder.rs`（**`#[cfg(feature = "default-prompt-providers")]`**） | **`PromptBuilder::build_prompt`**、段落装配、**`top_topic_hint(role, scene_id)`** | 依赖完整 **`Role`**（仅 `top_topic_hint` 读 `memory_config.topic_weights`）；其余字段来自 **`PromptInput` / `PromptRolePromptSlice`** |

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
| **可行性含义** | **共享类型与宿主注册方式已解耦**，不构成「迁入设施 crate」的阻碍；阻碍在 **算法模块之间的 runtime 内聚**（见下表）。 |

#### 6.7.3 `PromptInput` / `Role` — 依赖结论

| 检查项 | 结论 |
|--------|------|
| **`PromptInput` / `PromptRolePromptSlice`** | 契约在 **`oclive_kernel_core::prompt`** + **`kernel_models::prompt_role`**（§6.5）；字段已稳定，**`build_prompt` 主路径仅依赖 DTO + `Memory`（core）+ models 枚举**。 |
| **`Role` 耦合点** | **`PromptBuilder::top_topic_hint(&Role, scene_id)`** 仍读 **`Role.memory_config.topic_weights`**；完整 **`Role`** 依设计留在 **runtime**（§6.0「刻意未迁入 models」）。**`BuiltinPromptAssembler::top_topic_hint`** 对 `role_any` **向下转型为 `Role`**。 |
| **可行性含义** | 若设施 crate 仅接收 **`PromptInput`**：主拼装可迁出；**除非**将「场景话题权重」提升为 **`PromptInput` 可选字段**或独立快照类型，否则 **`top_topic_hint` 仍须依赖 `Role` 或 runtime 适配层**。 |

#### 6.7.4 综合评估表（是否进入阶段 7 实作）

| 剥离目标 | 阶段 7 结论 | 可行性分析摘要 |
|----------|-------------|----------------|
| **事件：`BuiltinEventEstimator*` 壳层** | **阶段 7 可执行（低价值）** | 壳层仅委托 `estimate_event_impact`；单独迁壳**无独立意义**，通常与算法同迁。 |
| **事件：`estimate_event_impact` + `EventDetector` + 人格轴辅助 + json 工具** | **阶段 7 待定** | **强耦合链**：`event_impact_ai` → `EventDetector` + `affect_policy::softness_coldness_volatility` + **`PersonalityEngine::calculate_stability_index`** + `utils/json_loose`。迁出需 **一并搬迁或抽象** 上述模块，或引入 **`oclive_event_builtin` 对 `oclive_kernel_runtime` 的依赖**（**禁止**：循环依赖风险）。更现实路径：先抽 **无状态纯函数 / 小 crate**（如 `json_loose`、detector 规则子集），再迁 LLM 编排。 |
| **Prompt：`BuiltinPromptAssembler*` 壳层** | **阶段 7 可执行（低价值）** | 同事件，壳层薄；单独迁出意义有限。 |
| **Prompt：`PromptBuilder` 全文（~800+ 行）** | **阶段 7 待定** | **主路径**已 **`PromptInput`** 化，迁出 **技术可行**；**`top_topic_hint` 仍绑定 `Role`**，设施 crate **不能**在「零依赖 runtime」前提下实现该方法。可选前置：**扩展 `PromptInput` 或新增 `TopicWeightsSnapshot`**，由编排层从 `Role` 填充，再迁 `PromptBuilder`。 |
| **PluginHost / trait / DTO** | **已就绪（非阻碍）** | 注册与类型边界已满足独立 Builtin crate 接入形态。 |

**小结**：**接口与模型层已可支撑**未来 `oclive_event_builtin` / `oclive_prompt_builtin`，但 **算法主体仍与 runtime 域模块交织（事件）或与完整 `Role` 交织（Prompt 话题提示）**，故 **整段 Builtin 实现「完整剥离」标记为阶段 7 待定**；若接受 **编排层传入额外 DTO** 或 **分步下沉工具模块**，可再开独立设计 PR 将子表中的「待定」逐项改为「可执行」。

## 7. 参考

- [KERNEL_BOUNDARY.md](./KERNEL_BOUNDARY.md)
- [KERNEL_BASELINE_V1.md](./KERNEL_BASELINE_V1.md)
- [LIGHTWEIGHT_PROFILE.md](./LIGHTWEIGHT_PROFILE.md)
