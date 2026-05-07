# Module 9：专家模型设施（命名与边界）

> **简称**：专家模型设施  
> **全称**：专家模型设施模块  
> 代码与 UI 中亦可见 **Expert Models / Module 9** 表述，与此处术语互参。

---

## 1. 它是什么

**专家模型设施**是 **内核侧** 的一套 **运行时设施**：在 **`role_runtime`** 上托管 **专家图（ExpertGraph）** 与 **Prompt 风格覆盖** 的 JSON，按 **角色默认 / 会话覆盖** 解析生效；将图 **安全编译** 为本地 **llama 侧车** 配置（`LlamaLocalPluginConfig`）和/或 **本会话云端 LLM**（`plugin_backends.llm = remote` + 可选 `expert_cloud_model_session_override`）；支持 **EventTrigger** 在回合后经标准 **memory** 写入长期记忆；并与主对话管线中的 Prompt 构建衔接（如 `PromptStyleOverride` 合并进角色视图）。

**可替换部分**主要是 **策略资产**：磁盘上的 **GGUF / LoRA**、图里 **节点与边** 的配置；**稳定部分**是 **存储、合并规则、编译与安全边界**（路径校验等），实现集中在 **`crates/oclive_kernel_runtime`**。

---

## 2. 与「六模块 + Agent（第七模块）」的区别

| 维度 | 六模块（含 `complex_emotion`）+ **第七模块 Agent** | **专家模型设施（Module 9）** |
|------|-----------------------------------------------|------------------------------|
| 配置入口 | 角色包 **`plugin_backends`**，经 **`PluginHost`** 解析为各 trait 实现 | **`role_runtime`** 上 **Expert / PromptStyle** 相关 JSON 列 + **`ExpertModelsRepository`** |
| 解决的问题 | **一轮对话管线**里：记忆、情绪、事件、Prompt、LLM、复杂情感、Agent 工具编排 **各走哪类后端** | **本地推理装配**：基座 + 多 LoRA + 可选 Prompt 风格节点；**侧车配置下发** |
| 「插拔」含义 | 换 **builtin / remote / directory** 等 **实现提供者** | 换 **图与权重文件**；内核提供 **托管与编译** |

因此：**专家模型设施模块** **不是** `PluginBackends` 里再多一个与 `memory` 同形的枚举槽位；它与 **路由型模块** 并列，属于 **内核托管的配置 / 资产型设施**。

---

## 3. 实现锚点（便于跳源码）

- 模型：`crates/oclive_kernel_runtime/src/models/expert_models.rs`（`ExpertGraph`、`ExpertNode`、`PromptStyleOverride` 等）
- 领域：`crates/oclive_kernel_runtime/src/domain/expert_models.rs`（图编译等）
- 持久化：迁移 **`018_expert_models.sql`**（`role_runtime` JSON 列）；`ExpertModelsRepository` / `SqliteExpertModelsRepository`
- 状态：`KernelAppState::effective_prompt_style_override` 等
- 桌面 invoke：`src-tauri/src/api/expert_models.rs`（API 薄层，业务公式仍在内核）

---

## 4. 与 Profile / 文档索引

- Profile 中 **六模块固定存在**、**Agent 为可选扩展** 的划分见 **[PROFILE_SCHEMA_v1.md](./PROFILE_SCHEMA_v1.md)**、**[MODULE_NONE_SEMANTICS.md](./MODULE_NONE_SEMANTICS.md)**。
- **专家模型设施** 的契约与 DTO 以 **`crates/oclive_kernel_runtime/src/models/dto.rs`** 中 Module 9 段落为准。
- 全库导航：**[../getting-started/DOCUMENTATION_INDEX.md](../getting-started/DOCUMENTATION_INDEX.md)**。
- **`.oclexpert` 分享 JSON 格式**（创作者导出/导入）：**[OCLEXPERT_FORMAT.md](./OCLEXPERT_FORMAT.md)**。

---

## 5. 命名约定（团队）

- 对外/对内短称：**专家模型设施**。
- 需与「后端路由模块」「插件槽」对举时：**专家模型设施模块**。
- 与 i18n / 历史文案中的 **Expert Models（Module 9）** 可互换理解，不必强行改 UI 字符串；新文档优先使用本页中文术语。
- **勿与 Kernel V2「官方默认××模块」混称**：后者指对话管线槽位上、随发行版提供的进程内默认实现（`oclive_memory_builtin` 等），见 [KERNEL_BOUNDARY.md](./KERNEL_BOUNDARY.md) §1.1、[KERNEL_V2_DESIGN.md](./KERNEL_V2_DESIGN.md) §4。第九模块解决的是 **ExpertGraph / 本地推理装配**，与 `plugin_backends` 枚举槽 **不同构**。
