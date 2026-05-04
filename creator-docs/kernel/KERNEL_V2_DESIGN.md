# Kernel V2：极薄内核与默认实现分层

> 状态：**阶段 5 主体已落地**；**6-1** 共享 models、**6-2** 事件门面稳定见 **§6.0 / §6.4**。事件 / Prompt 完整剥离仍评估中（**§6.1–6.2**）。  
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
| 5 | `default-*-providers` 细化：设施 crate（memory / emotion / complex_emotion / **agent ReAct**）+ directory 示例；`kernel-agent` 与 **`default-agent-providers`** 分离（MCP 基础 vs 进程内 Builtin ReAct）；极薄 `cargo check -p oclive_kernel_runtime --no-default-features` |

## 4. 设施 crate 与默认提供者（阶段 5 快照）

| 设施 crate | `default-*-providers` | runtime 中保留的薄层 / 基础能力 |
|------------|----------------------|----------------------------------|
| `oclive_memory_builtin` | `default-memory-providers` | `classic` 再导出；directory 示例 `memory.rank` |
| `oclive_emotion_builtin` | `default-emotion-providers` | `EmotionAnalyzer` / `EmotionResultExt`；`emotion.analyze` 示例 |
| `oclive_complex_emotion_builtin` | `default-complex-emotion-providers` | `affect_metrics_from_seven_dim`；`complex_emotion.resolve_turn` 示例 |
| `oclive_agent_builtin` | `default-agent-providers` | **`McpShellAgent`** 仍在 runtime；ReAct 在设施 crate；`agent.process` 示例（契约演示） |

**Agent**：`kernel-agent` 控制 MCP 栈与轻量 **`McpShellAgent`**；**`default-agent-providers`** 单独控制是否链接 **`BuiltinReActAgent`**（`oclive_agent_builtin/providers`）。

## 5. 已实现（阶段 2 首包）

- Crate：`crates/oclive_kernel_core`
- 已迁入：`AppError` / `Result`、`models::memory`、`repository`（`MemoryRepository`、`FavorabilityRepository`、`ExpertModelsRepository`）
- `oclive_kernel_runtime` 对上述路径 **再导出**；移除 `tauri_invoke`（`AppError` 外置后不再在 runtime 内提供 `From<AppError> for InvokeError`，桌面显式 `to_frontend_error()`）。

## 6. 阶段 6 待定 — 事件与 Prompt 剥离评估（阶段 5-6 / 5-7 结论）

以下为 **2026-05** 依赖扫描结论：**不在当前阶段强制**创建 `oclive_event_builtin` / `oclive_prompt_builtin`**，**避免「为剥离而剥离」**导致超大搬迁或 `runtime ↔ facility` 循环依赖。先行记录迁移前置条件，待 **`oclive_kernel_models`** 或等价共享模型层就绪后再做。

### 6.0 阶段 6-1 固化：`oclive_kernel_models` 迁入清单

以下类型已迁至 **`crates/oclive_kernel_models`**（纯数据：无 `*Engine` / `*Repository` / `PluginHost` / I/O）。`oclive_kernel_runtime` 通过 `pub use oclive_kernel_models::...` 薄再导出，既有 `crate::models::*` 路径可保持不变。

| 类型 | 原 `kernel_runtime` 位置 | 现定义位置 |
|------|--------------------------|------------|
| `EventType`、`Event` | `src/models/event.rs`（曾本地定义） | `kernel_models::event` |
| `PersonalityVector`（含 `clamp` / `effective_from_core_delta` / `to_json_vec` 等） | `src/models/personality.rs` | `kernel_models::personality` |
| `PersonalityDefaults`、`EvolutionBounds`、`EvolutionConfig`、`MemoryConfig`、`UserRelation` | `src/models/role.rs`（与 `Role` 混排） | `kernel_models::role_config` |
| `KnowledgeEventAugment` | `src/models/knowledge.rs`（曾与 `KnowledgeIndex` / merge 逻辑混写） | `kernel_models::knowledge_augment` |
| `EventImpactEstimate` | `domain/event_impact_ai.rs`（曾与 LLM/规则算法混写） | `kernel_models::event_impact`（**阶段 6-2**） |

**刻意未迁入**：完整 `Role`、`PromptInput`、编排绑定的大型结构仍驻 runtime（远程 `prompt.build_prompt` 等需完整角色契约）。

#### `kernel_core` 与 `oclive_kernel_models`

| Crate | 职责 | 依赖关系 |
|-------|------|----------|
| `oclive_kernel_models` | 共享 **纯数据**（事件 / 性格向量 / 进化与用户关系片段 / 知识增强 / 事件估计 DTO） | **不**依赖 `kernel_core` |
| `oclive_kernel_core` | `AppError`、`Emotion`、`Memory`、`LlmClient`、各类 **trait**、仓库端口 | **阶段 6-1**：不依赖 `kernel_models`。**阶段 6-2 起**：**单向**依赖 `kernel_models`（仅 `EventEstimator` trait 签名引用纯数据类型）；**仍不**依赖完整 runtime |
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

**评估**：从设施 crate 视角，至少牵连 **规则检测、人格轴公式、JSON 工具、多种模型类型** 与 **一长串 `estimate` 参数**，独立 trait/DTO 边界 **明显超过 5**；且 **`KnowledgeEventAugment` 与知识索引管线耦合**。与 Agent/MCP **无硬耦合**（仅需 `LlmClient`）。**阶段 6-2（最小切线）**：已稳定 **trait + `EventImpactEstimate`**，并用 **`default-event-providers`** 控制 **`event_impact_ai` 模块与 `BuiltinEventEstimator*`**（关则桩 `DisabledEventEstimator`，见 **§6.4**）。整仓搬迁 `EventDetector` + `event_impact_ai` 算法主体仍为后续工作。

### 6.2 Prompt（`PromptAssembler` / `PromptBuilder`）

| 类别 | 依赖项 |
|------|--------|
| **门面** | `PromptAssembler` trait（runtime）、`BuiltinPromptAssembler` / `V2` |
| **实现** | `PromptBuilder::build_prompt`、`PromptInput<'_>`（`domain/prompt_builder.rs`，体量极大） |
| **模型** | `Role`、`Memory`、`PersonalityVector`、`PersonalitySource`、`EventType` 及大量角色包字段 |

**评估**：Prompt 与 **编排上下文**、**角色磁盘模型** 绑定最深；剥离成本高于事件模块。**建议阶段 6**：与 **`Role` / `PromptInput` 契约** 一并设计（或目录插件仅暴露「片段拼装」而非整段 `build_prompt`）。

### 6.3 验收（当前阶段）

- 无新增设施 crate；**`cargo check --workspace`** / **`cargo test --workspace`** 通过。
- **`default-prompt-providers`**：仍为空数组，内置 Prompt 仍在 runtime（行为不变）。
- **`default-event-providers`**：仍为空数组；**开启**（`full` 默认）时装配进程内事件估计；**关闭**时 **`event_impact_ai` 不参与编译**，builtin 槽与 Remote 占位回退 **`DisabledEventEstimator`**（`Ignore` / 零置信）。极薄检出：**`cargo check -p oclive_kernel_runtime --no-default-features`**。

### 6.4 阶段 6-2：事件门面最小切线（已实现）

- **`EventEstimator`**：`oclive_kernel_core::event_estimator`（`async_trait` + `LlmClient` + models DTO + `PersonalitySource`）。
- **`EventImpactEstimate`**：`oclive_kernel_models::event_impact`（serde，Remote JSON-RPC 与内置共用）。
- **`BuiltinEventEstimator` / `BuiltinEventEstimatorV2`**：仍在 **`oclive_kernel_runtime::domain::event_estimator`**，整段置于 **`#[cfg(feature = "default-event-providers")]`**；算法委托 **`estimate_event_impact`**。
- **`domain::event_impact_ai`**：仅在该 feature **开启**时编译（`domain/mod.rs` 条件模块），减轻 `--no-default-features` 依赖面。
- **桩**：**`DisabledEventEstimator`**（`disabled_default_providers.rs`），feature 关闭时由 **`default_event_slot_v1/v2`** 选用。

## 7. 参考

- [KERNEL_BOUNDARY.md](./KERNEL_BOUNDARY.md)
- [KERNEL_BASELINE_V1.md](./KERNEL_BASELINE_V1.md)
- [LIGHTWEIGHT_PROFILE.md](./LIGHTWEIGHT_PROFILE.md)
