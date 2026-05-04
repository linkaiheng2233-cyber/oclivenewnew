# Kernel V2：极薄内核与默认实现分层

> 状态：**阶段 5 主体已落地**；**阶段 6-1** `oclive_kernel_models` 已落地（见 **§6.0**）。事件 / Prompt 剥离评估见 **§6**。  
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

**刻意未迁入**：完整 `Role`、`PromptInput`、编排绑定的大型结构仍驻 runtime（远程 `prompt.build_prompt` 等需完整角色契约）。

#### `kernel_core` 与 `oclive_kernel_models`（阶段 6-1）

| Crate | 职责 | 依赖关系（6-1） |
|-------|------|----------------|
| `oclive_kernel_models` | 共享 **纯数据**（事件 / 性格向量 / 进化与用户关系片段 / 知识增强片段） | **不**依赖 `kernel_core` |
| `oclive_kernel_core` | `AppError`、`Emotion`、`Memory`、`LlmClient`、各类 **trait**、仓库端口 | **截至 6-1** **不**依赖 `kernel_models`，可与 models **并行**演进 |
| `oclive_kernel_runtime` | 内置引擎、存储、HTTP、插件宿主 | 依赖 **core + models** |

### 6.1 事件（`EventEstimator` / `event_impact_ai`）

| 类别 | 依赖项 |
|------|--------|
| **门面** | `EventEstimator` trait、`EventImpactEstimate`（定义于 `domain/event_estimator.rs` / `event_impact_ai.rs`，trait 仍在 runtime） |
| **算法主体** | `estimate_event_impact` → LLM `generate_tag` + JSON 解析 |
| **规则回退与检测** | `EventDetector`（`domain/event_detector.rs`，关键词 + `KnowledgeEventAugment`） |
| **人格公式** | `affect_policy::softness_coldness_volatility`、`PersonalityEngine::calculate_stability_index` |
| **工具** | `utils/json_loose::extract_json_object` |
| **外部抽象** | `LlmClient`（已在 core） |
| **模型 / DTO** | `Emotion`（core）、`Event` / `EventType`、`PersonalityVector`、`PersonalitySource`、`KnowledgeEventAugment`（runtime `models`） |

**评估**：从设施 crate 视角，至少牵连 **规则检测、人格轴公式、JSON 工具、多种模型类型** 与 **一长串 `estimate` 参数**，独立 trait/DTO 边界 **明显超过 5**；且 **`KnowledgeEventAugment` 与知识索引管线耦合**。与 Agent/MCP **无硬耦合**（仅需 `LlmClient`）。**建议阶段 6**：先收敛 **`models` 下沉 core** 或独立 **`oclive_kernel_models`**，再整体搬迁 `EventDetector` + `event_impact_ai` + `BuiltinEventEstimator*`。

### 6.2 Prompt（`PromptAssembler` / `PromptBuilder`）

| 类别 | 依赖项 |
|------|--------|
| **门面** | `PromptAssembler` trait（runtime）、`BuiltinPromptAssembler` / `V2` |
| **实现** | `PromptBuilder::build_prompt`、`PromptInput<'_>`（`domain/prompt_builder.rs`，体量极大） |
| **模型** | `Role`、`Memory`、`PersonalityVector`、`PersonalitySource`、`EventType` 及大量角色包字段 |

**评估**：Prompt 与 **编排上下文**、**角色磁盘模型** 绑定最深；剥离成本高于事件模块。**建议阶段 6**：与 **`Role` / `PromptInput` 契约** 一并设计（或目录插件仅暴露「片段拼装」而非整段 `build_prompt`）。

### 6.3 验收（当前阶段）

- 无新增设施 crate；**`cargo check --workspace`** / **`cargo test --workspace`** 仍以现有树为准。
- **`default-event-providers`** / **`default-prompt-providers`** 保持指向 runtime 内置实现（空 feature 数组），行为不变。

## 7. 参考

- [KERNEL_BOUNDARY.md](./KERNEL_BOUNDARY.md)
- [KERNEL_BASELINE_V1.md](./KERNEL_BASELINE_V1.md)
- [LIGHTWEIGHT_PROFILE.md](./LIGHTWEIGHT_PROFILE.md)
