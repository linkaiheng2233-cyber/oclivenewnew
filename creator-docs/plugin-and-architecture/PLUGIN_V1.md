# PLUGIN_V1 — 编排层契约与后端枚举（蓝图 v2/v3/v4 · legacy 六槽）

**SSOT 范围**：六槽 DTO、后端枚举、解析与 Stable 固定 stage 中的槽位调用；Event Ring wire 见独立契约。
**最后更新**：2026-09-05。

> **2026-06-10 起**：`builtin_v2` 为 **已废弃 wire alias**（serde 读兼容），行为等同 `builtin`；四槽无独立 V2 实现（D-SLOT-01）。下文 legacy 表中 `builtin_v2` 行仅作迁移对照。

**插件作者学习路径**：[PLUGIN_AUTHOR_LEARNING_PATH.md](PLUGIN_AUTHOR_LEARNING_PATH.md)

**当前权威**：角色包 **`pipeline.ocblueprint` → `slot_registry`**（见 [ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md)）。本文档描述宿主（Tauri / `chat_engine`）与可替换子系统之间的 **编排契约**：DTO 形状、槽位门面 trait、蓝图实例解析；下文 **legacy** 段落中的 `settings.json` → `plugin_backends` 仅用于 **v1（已废弃）** 迁移对照。实现以源码为准：`slot_resolver.rs`、`plugin_host.rs`、`kernel/crates/oclive_kernel_types/src/models/plugin_backends.rs`。

**全库文档索引**：[../getting-started/DOCUMENTATION_INDEX.md](../getting-started/DOCUMENTATION_INDEX.md)。**架构总览（单核双态 · 后端/插件/设施 · `{专名}设施子模块`）**：[../getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md](../getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md)。**以内核为中心、模块环绕的总览（图 + Mermaid）**：[../getting-started/KERNEL_AND_MODULES_ARCHITECTURE.md](../getting-started/KERNEL_AND_MODULES_ARCHITECTURE.md)。Event Ring 的信封、注册与主动授权见 **[EVENT_RING.md](EVENT_RING.md)**。包版本与 `schema_version` 见 **[../role-pack/PACK_VERSIONING.md](../role-pack/PACK_VERSIONING.md)**。HTTP 侧车 JSON-RPC 全文见 **[REMOTE_PLUGIN_PROTOCOL.md](REMOTE_PLUGIN_PROTOCOL.md)**；创作者总览见 **[CREATOR_PLUGIN_ARCHITECTURE.md](CREATOR_PLUGIN_ARCHITECTURE.md)**。**目录式进程插件**（蓝图实例 `backend: directory`、整壳、`directory_plugin_invoke` 等）见 **[DIRECTORY_PLUGINS.md](DIRECTORY_PLUGINS.md)**。

## 蓝图角色包（`pipeline.ocblueprint`）

自 **schema_version: 2** 起，角色包 **SSOT** 为 [`pipeline.ocblueprint`](../role-pack/ROLE_PACK_SPEC.md) 的 **`slot_registry`**（开放多实例键），不再使用 `settings.json` → `plugin_backends` 六键固定形状。宿主经 **`SlotResolver` / `SlotRunner`** 按实例解析；同 `type` 折叠为 `PluginBackends` 时 **last-wins**（`position` 最大者优先）。

| v1（legacy） | v2 蓝图 |
|--------------|---------|
| `plugin_backends.memory` … `agent` | `slot_registry.{实例键}.type` + `backend` |
| `directory_plugins.{module}` | 各 directory 槽的 `plugin` / `plugins` |
| 复杂情感设施子模块（第 1 号，无六槽键） | 编排行内；架构图可选 `slot_registry` 中 `type: complex_emotion` 实例 |

架构图 **写盘**：`save_role_slot_registry`（Tauri）更新包内 `slot_registry`；写盘后宿主 **`invalidate_role_cache`** 并重新 **`load_role`**。工具栏可 **添加/删除** 槽位实例；**至少保留一个 `type: llm`**，**最后一个 llm 不可删除**（与 `oclive_validation` 一致）。会话覆盖仍为 `set_session_slot_override`（内存，不写盘）。

---

## 设计约束

- **可替换后端 = 编译期枚举 + 蓝图实例**：**v2/v3/v4** 通过 **`slot_registry`** 声明多实例（新 Stable 包使用 v4）；**legacy v1** 通过 `settings.json` → `plugin_backends`（勿在新包中使用）。无动态 `cdylib`。
- **默认实现**即当前内置逻辑；换后端时 **API 字段名不变**（尤其 `SendMessageResponse.reply`）。
- **Remote**：宿主已实现 **HTTP JSON-RPC**（见 [REMOTE_PLUGIN_PROTOCOL.md](REMOTE_PLUGIN_PROTOCOL.md)）；未配置 `OCLIVE_REMOTE_*` URL 时回退 **builtin**（或进程内 LLM）并写日志。
- **Directory**：由宿主扫描插件根下的 `*/manifest.json`，以子进程提供与 Remote 相同的 JSON-RPC wire；当前蓝图实例用 `plugin` / `plugins` 指向 `manifest.id`，legacy v1 才用 `plugin_backends.directory_plugins`（[DIRECTORY_PLUGINS.md](DIRECTORY_PLUGINS.md)）。

## 架构图（当前蓝图与运行时折叠）

当前包配置先形成会话级有效 **`slot_registry`**；宿主再把六种稳定类型折叠为运行时 **`PluginBackends`**，并保留实例注册表给 `SlotRunner` 与复杂情感设施使用。运行时结构体 **`PluginBackends`**（[`plugin_backends.rs`](../../kernel/crates/oclive_kernel_types/src/models/plugin_backends.rs)）仍只有 **六** 个枚举字段；其中 `directory_plugins` 是折叠后的兼容视图，不是角色包当前 SSOT。编排层通过 **`PluginHost`** 将每槽绑定到具体 **`Arc<dyn …>`** 实现，再由 **`chat_engine`** 按 **`send_message` 编排顺序**（见下一节）调用。**`complex_emotion`** 可作为开放注册表中的设施类型被解析，但**不是**第七个稳定槽。

### 模块编号对照（与架构总览一致）

| 编号 | `slot_registry.type`（legacy 六键） | 类型 |
|------|------------------------------------|------|
| 第 1 模块 | `memory` | 后端模块 |
| 第 2 模块 | `emotion` | 后端模块 |
| 第 3 模块 | `event` | 后端模块 |
| 第 4 模块 | `prompt` | 后端模块 |
| 第 5 模块 | `llm` | 后端模块 |
| 第 6 模块 | `agent` | 后端模块 |
| 第 1 设施子模块 | `complex_emotion`（无 legacy 六键） | 复杂情感设施子模块 |
| 第 2 设施子模块 | （无稳定槽类型；编排行内） | 专家模型设施子模块（专家路由） |

**后端模块插件模块**（Remote / directory 等）挂在 **第 K 模块** 上，**不**占用第 7 模块号。完整规定见 [OCLIVE_ARCHITECTURE_OVERVIEW.md](../getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md)。

```mermaid
flowchart TB
  subgraph pack["角色包 / 会话覆盖"]
    BP["pipeline.ocblueprint → slot_registry<br/>实例 type · backend · plugin(s)"]
    SO["会话 slot override<br/>按实例键覆盖 · 仅内存"]
    LEG["legacy v1 settings.json<br/>plugin_backends · 仅迁移兼容"]
  end

  subgraph resolve["宿主解析链"]
    EFF["EffectiveSessionConfig<br/>合成有效 slot_registry"]
    FOLD["六类型 last-wins 折叠<br/>PluginBackends 兼容视图"]
    PH["PluginHost / SlotRunner<br/>实例与 trait 绑定"]
  end

  subgraph orch["编排"]
    CE["chat_engine<br/>process_message / co_present"]
  end

  BP --> EFF
  SO --> EFF
  LEG -.-> FOLD
  EFF --> FOLD
  EFF --> PH
  FOLD --> PH
  PH --> CE

  subgraph slots["六条门面线 ResolvedRolePlugins"]
    M["MemoryRetrieval"]
    EM["UserEmotionAnalyzer"]
    EV["EventEstimator"]
    PR["PromptAssembler"]
    LL["LlmClient"]
    AG["AgentProvider<br/>（MCP 等见 AGENTS.md）"]
  end

  PH --> slots
  slots --> CE

  subgraph shapes["实现形态（每槽枚举见本文各节表）"]
    BIN["builtin / ollama<br/>进程内 Rust"]
    REM["remote<br/>HTTP JSON-RPC + OCLIVE_REMOTE_*"]
    DIR["directory<br/>扫描插件根；子进程，同协议 wire"]
    LOC["memory: local<br/>_local_plugins"]
  end

  slots -.-> shapes
```

### 多实例执行真值

`PluginBackends` 的同类型 **last-wins 折叠只是兼容视图**，不等于每种槽的实际执行合并策略。`SlotRunner` / `PluginHost` 当前行为如下：

| 类型 | 当前执行策略 |
|------|--------------|
| `memory` | 串行检索，按 memory id 去重后重新排序/截断 |
| `emotion` / `event` / `prompt` / `complex_emotion` | 串行 last-wins |
| `llm` 非流式 | 读取最后一个 LLM 实例的 `policy`：`ensemble` = 串行 last-wins（默认）、`fastest` = 并发首个成功、`fallback` = 按顺序首个成功 |
| `llm` 流式 | 当前三种 `policy` 都归一为串行 last-wins，只让最后实例输出 token，避免并发流混写 |
| `agent` | 当前执行折叠后的单一 provider；多实例/`plugins[]` 只形成 `merged_agent_directory_plugin_ids` 诊断集合，`wrap_agent_if_merged` 为 no-op。工具并集登记为 `K-AGENT-MERGE-01` |

---

## `send_message` 固定 stage 与槽位调用（与 `chat_engine`）

Stable 回合入口为 [`chat_engine::process_message`](../../kernel/crates/oclive_kernel_host/src/domain/chat_engine/process_message.rs)，由 [`dispatch_turn`](../../kernel/crates/oclive_kernel_host/src/domain/chat_engine/dispatch.rs) 选择 remote stub、remote-life 或 [`turn_pipeline::execute_turn`](../../kernel/crates/oclive_kernel_host/src/domain/chat_engine/turn_pipeline/mod.rs)。共景 middle 位于 [`co_present/run_middle.rs`](../../kernel/crates/oclive_kernel_host/src/domain/chat_engine/turn_pipeline/co_present/run_middle.rs)。下面描述的是**内核固定 stage 中的调用关系**，不是六个槽首尾相接的线性管道：

1. **有效配置与 `PluginHost`**：`EffectiveSessionConfig` 先把包内 `slot_registry` 与会话实例覆盖合成有效注册表，再将六种稳定类型折叠成 `PluginBackends` 兼容视图；[`PluginHost`](../../kernel/crates/oclive_kernel_host/src/domain/ports/plugin_host.rs) 据此绑定 **`memory` / `emotion` / `event` / `prompt` / `llm` / `agent`** 六条**后端模块**线，`SlotRunner` 保留实例语义。legacy v1 才直接从 `role.plugin_backends` 起步。宿主构造 `PluginHost` 需传入 **应用数据根目录**（`PathBuf`），用于扫描 **`{app_data}/mcp-servers/*.json`** 等；集成烟测见 [`distros/desktop-tauri/tests/plugin_backends_v2_resolve.rs`](../../distros/desktop-tauri/tests/plugin_backends_v2_resolve.rs)。
2. **Agent（第 6 模块）**：在 preflight 之后、普通共景 pre 之前，只对普通用户、非 staged 回合尝试；`handled=true` 时由 `build_minimal_response` 返回最小响应并短路 Stable 闲聊链。该分支会独立调用情绪槽并保存最小状态/聊天，但不执行普通共景的回复后处理与 `reply_mode`。
3. **pre**：第 2 模块 `emotion.analyze`（仅用户消息）与第 1 模块 `memory.rank_memories`，并加载人格、关系、身份和近期上下文。
4. **middle 前半**：先用规则得到事件初估，再由该初估和其他证据决定 Turn Thinking；同时处理 Fast 本地情绪强度降级与知识检索。只有策略允许时，第 3 模块 `event.estimate` 才调用 LLM/插件来替换规则初估。第 4 模块稍后只接收上一轮 hint 的去内容连续性信号，不消费本轮尚未产生的 hint。
5. **Event Ring**：对话估计通过 `kernel.chat.event_impact.estimated` 兼容桥；memory 可提出 `kernel.memory.recall.candidate`，采纳后生成单轮 `kernel.memory.recollection.activated`。wire 与权威边界见 [EVENT_RING.md](EVENT_RING.md)。
6. **Prompt（第 4 模块）**：`top_topic_hint` + `build_prompt` / `build_prompt_segments`，消费已经确定的角色、情绪、事件、关系、记忆和外部观察上下文。
7. **主 LLM（第 5 模块）**：`generate` / `generate_stream` 产生原始 `reply` 与可选 `[EMO]`。
8. **post**：剥离 `[EMO]`，按“有效主模型标记优先、remote/directory 仅在标记缺失或无效时兜底”解析本轮角色回复情绪与复杂情感；先让语义回复参与情绪、关系、记忆、立绘等状态计算和持久化，并在设施启用时保存下一轮 hint，再依次执行单一回复后处理器、普通共景 `reply_mode`、聊天写入与 `SendMessageResponse` 组装。后处理是独立通道，不是第七槽；当前没有任意多处理器链。

门面与枚举的单一事实来源：`plugin_host.rs`、`models/plugin_backends.rs`、本文各节表格。

---

## 记忆检索 `MemoryRetrieval`

### 输入：`MemoryRetrievalInput`

| 字段 | 类型 | 说明 |
|------|------|------|
| `memories` | `&[Memory]` | 已由仓储读出并经场景权重等处理后的候选集 |
| `user_query` | `&str` | 当前用户句（用于关键词加权等检索策略） |
| `scene_id` | `Option<&str>` | 当前场景 id；可选参与未来检索策略 |
| `limit` | `usize` | 注入主 prompt 的最大条数 |

### 输出

- **排序后的** `Vec<Memory>`，长度不超过 `limit`。
- 结构化上下文 `MemoryContext`（`build_context`）与 `models::MemoryContext` 一致：`memories` + `total_tokens` 估计。

### 后端枚举 `memory`（蓝图 `type: memory`；legacy 键同名）

| 值 | 含义 |
|----|------|
| `builtin` | 按 `importance * weight` 排序取 Top-K（与历史 `MemoryEngine::get_relevant_memories` 一致） |
| `builtin_v2` | **已废弃 wire alias**（2026-06-10 起读入等同 `builtin`） |
| `remote` | HTTP `memory.rank`（需 `OCLIVE_REMOTE_PLUGIN_URL`；失败回退 `builtin`） |
| `directory` | HTTP `memory.rank` 指向实例 `plugin` 对应 manifest 子进程 URL（legacy 由 `directory_plugins.memory` 折叠；失败回退 `builtin`） |
| `local` | 使用已注册的本地 memory provider（`<roles 根>/_local_plugins/*.json`）；**当前阶段**排序仍委托 `builtin` 逻辑，多 provider 时按 `provider_id` 字典序取第一个并打警告（见 [LOCAL_PLUGIN_BRIDGE_SPEC.md](LOCAL_PLUGIN_BRIDGE_SPEC.md)） |
| `none` | Noop：不参与记忆检索与排序 |

与 memory 实例相关的可选字段（括号内为 legacy 折叠位置）：

| 字段 | 类型 | 说明 |
|------|------|------|
| `local_memory_provider_id` | `string`（可选） | 仅 `backend: local` 时有意义；当前写在 memory 实例上（legacy 与六键同级） |
| `plugin` / `plugins` | `string` / `string[]` | 当前 directory 实例的插件 id；legacy 才折叠为 `directory_plugins.{type}` |

---

## 用户情绪 `UserEmotionAnalyzer`

### 输出

与 `EmotionResult` / `EmotionDto` 对齐：

- 七维分数：`joy`, `sadness`, `anger`, `fear`, `surprise`, `disgust`, `neutral`（`f32` / `f64` 在各自层约定）。
- 主导情绪通过既有 `Emotion` 枚举映射；**不得**引入未在 `models/emotion.rs` 定义的变体名对外暴露。

### 后端枚举 `emotion`

| 值 | 含义 |
|----|------|
| `builtin` | 关键词启发式（现有 `EmotionAnalyzer`） |
| `builtin_v2` | **已废弃 wire alias**（读入等同 `builtin`） |
| `remote` | HTTP `emotion.analyze`（需 `OCLIVE_REMOTE_PLUGIN_URL`；失败回退 builtin） |
| `directory` | HTTP `emotion.analyze` 指向实例 `plugin` 对应插件 URL（失败回退 builtin） |
| `none` | Noop：跳过用户句情绪分析 |

---

## 事件影响 `EventEstimator`

### 输入（概念）

与 `estimate_event_impact` 一致：`LlmClient`、`ollama_model`、用户句、`Emotion`、`PersonalityVector`、**`personality_source`（`vector` | `profile`，与包内 `evolution.personality_source` 一致）**、近期轮次与事件列表。Remote 的 `event.estimate` 在 JSON-RPC `params` 中与 `personality` 并列携带该字段（见 [REMOTE_PLUGIN_PROTOCOL.md](REMOTE_PLUGIN_PROTOCOL.md) §4.3）。

### 输出：`EventImpactEstimate`

- `event_type: EventType`
- `impact_factor: f64`
- `confidence: f32`

### 后端枚举 `event`

| 值 | 含义 |
|----|------|
| `builtin` | 现有 `event_impact_ai::estimate_event_impact` 链（含环境开关与规则回退） |
| `builtin_v2` | **已废弃 wire alias**（读入等同 `builtin`） |
| `remote` | HTTP `event.estimate`（需 `OCLIVE_REMOTE_PLUGIN_URL`；失败回退 builtin） |
| `directory` | HTTP `event.estimate` 指向实例 `plugin` 对应插件 URL（失败回退 builtin） |
| `none` | Noop：不产生新的事件影响估计 |

---

## Prompt 组装 `PromptAssembler`

### 输入 / 输出

- 输入：`PromptInput`（`PromptBuilder::build_prompt` 为**单参数 `&PromptInput<'_>`**，角色、性格、记忆、用户输入、情绪等所有上下文字段聚合在该结构体上）。
- 输出：`String`（主对话 system/user 拼装结果）。

附加：`top_topic_hint(role, scene_id) -> Option<String>` 与现 `PromptBuilder::top_topic_hint` 对齐。

### 后端枚举 `prompt`

| 值 | 含义 |
|----|------|
| `builtin` | 现有 `PromptBuilder` |
| `builtin_v2` | **已废弃 wire alias**（读入等同 `builtin`） |
| `remote` | HTTP `prompt.build_prompt` / `prompt.top_topic_hint`（需 `OCLIVE_REMOTE_PLUGIN_URL`；失败回退 builtin） |
| `directory` | 同上，指向实例 `plugin` 对应插件 URL（失败回退 builtin） |
| `none` | Noop；合法但会使健康共景路径缺少必要 Prompt 能力 |

---

## 主对话 LLM `LlmClient`

### 职责

- `generate`：主回复、异地模式、独白等所有「生成型」调用。
- `generate_tag`：短输出分类（位移意图、立绘标签等）。

### 后端枚举 `llm`

| 值 | 含义 |
|----|------|
| `ollama` | 应用启动时注入的默认客户端（通常为 `OllamaClient` 包装） |
| `remote` | HTTP `llm.generate` / `llm.generate_tag`（需 `OCLIVE_REMOTE_LLM_URL`；未配置则委托进程内默认 LLM 并记日志）。环境变量 **`OCLIVE_LLM_BACKEND=remote|ollama|directory`** 可在加载角色时覆盖本字段（例如由 **oclive-launcher** 注入）。 |
| `directory` | HTTP `llm.generate` / `llm.generate_tag` 指向实例 `plugin` 对应插件 URL（失败回退 **ollama**） |
| `none` | Noop；合法但无法生成正常主回复 |

---

## Agent 编排 `AgentProvider`

工具调度 / ReAct 等任务编排；与主对话 LLM 分离，由有效 `type: agent` 实例选择实现。详见仓库根 [`AGENTS.md`](../../AGENTS.md) 中 **Agent / Skill** 小节。

### 后端枚举 `agent`

| 值 | 含义 |
|----|------|
| `builtin` | 进程内 [`BuiltinReActAgent`](../../kernel/crates/oclive_kernel_host/src/domain/agent.rs)；可配合 MCP 工具（配置目录见上节 `PluginHost::new` 的 app data 根） |
| `remote` | HTTP JSON-RPC 侧车 **`agent.process`**（`OCLIVE_REMOTE_AGENT_URL` 或回退 `OCLIVE_REMOTE_PLUGIN_URL`）；协议见 [AGENT_REMOTE_PROTOCOL.md](AGENT_REMOTE_PROTOCOL.md)；失败 **降级 builtin** |
| `directory` | 子进程 JSON-RPC；实例用 `plugin` / `plugins` 指定 id（失败 **降级 builtin**） |
| `none` | `NoopAgentProvider`；普通对话继续走非 Agent 主链 |

---

## legacy v1 `settings.json` 片段（仅迁移对照）

```json
{
  "schema_version": 1,
  "plugin_backends": {
    "memory": "builtin",
    "emotion": "builtin",
    "event": "builtin",
    "prompt": "builtin",
    "llm": "ollama",
    "agent": "builtin"
  }
}
```

仅对 legacy v1：省略 `plugin_backends` 时，记忆 / 情绪 / 事件 / Prompt / **Agent** 为 **builtin**，**`llm` 为 `ollama`**。未知枚举值会导致角色包解析失败。新包应写 `pipeline.ocblueprint.slot_registry`，不要复制此片段作为当前格式。

---

## 会话级槽位覆盖（Tauri）

当前命令 **`set_session_slot_override`** 按 **`role_id` + 可选 `session_id` + `slot_key`** 修改会话内存中的实例补丁，**不写回角色包，也不跨进程持久化**。`load_role` / **`get_role_info`** 返回 `slot_registry_pack`、`slot_registry_effective` 与 `slot_session_overridden_keys`；六槽折叠兼容快照仍见 `plugin_backends_effective` / `plugin_backends_effective_sources`。实现见 [`slot_session.rs`](../../kernel/crates/oclive_kernel_host/src/service/role/slot_session.rs) 与 [`slots.rs`](../../kernel/crates/oclive_kernel_types/src/models/dto/slots.rs)。

### 请求字段（摘要）

| 字段 | 说明 |
|------|------|
| `role_id` | 角色 id |
| `slot_key` | `slot_registry` 实例键，如 `memory`、`llm_primary` 或 `complex_emotion` |
| `backend` | 可选；非空字符串覆盖实例后端 |
| `plugin` / `plugins` | 可选；覆盖 directory 单插件 / 合并插件 id |
| `model` | 可选；覆盖该实例模型标识 |
| `session_id` | 可选；缺省为默认会话 |
| `local_memory_provider_id` | 可选；用于 memory 实例的本地记忆提供者标识 |

请求中的非空字段会与同一实例已有补丁合并，后写字段覆盖先写字段；若补丁五个可选字段全为空，宿主会清除该实例的整份会话覆盖。明确清除请优先调用 **`clear_session_slot_override`**；清空当前会话全部实例覆盖用 **`clear_all_session_slot_overrides`**。

前端封装见 [`setSessionSlotOverride`](../../distros/shared/src/api/settings.ts)、`clearSessionSlotOverride`、`clearAllSessionSlotOverrides` 与 `getRoleInfo`。

### legacy 六槽薄封装

旧命令 **`set_session_plugin_backend`** 仍作为默认六实例（`memory` / `emotion` / `event` / `prompt` / `llm` / `agent`）的薄封装存在，但要求角色包已有 `slot_registry`，内部会转调 `set_session_slot_override`。它不支持任意实例键，也不能设置 directory 插件 id；当前 directory 会话切换应直接用 `set_session_slot_override` 的 `plugin` / `plugins`。见 [DIRECTORY_PLUGINS.md](DIRECTORY_PLUGINS.md)。

---

## 前端对齐

TypeScript 侧 `SendMessageResponse`（`distros/shared/src/api/`）必须与 `models/dto/` 一致：**回复字段名为 `reply`**；`presence_mode`、`reply_is_fallback`、`schema`、`api_version` 用于展示策略（见 `distros/shared/src/utils/replyPresentation.ts`）。

---

## 前端 UI 模板配置（Plugin Manager V2）

为降低创作者接入成本，V2 面板支持通过 manifest 声明 UI 模板并按 schema 渲染。后端新增插件时，优先复用同类型模板。

### `ui_template` 可选值

| 值 | 适用场景 |
|---|---|
| `endpoint-config` | 需要填写服务地址（如远程 HTTP 侧车） |
| `provider-selector` | 从多个后端实现中选择一个（如 builtin / remote / directory） |
| `slot-selector` | 以“槽位”语义选择后端（面向非技术用户） |
| `switch-toggle` | 布尔开关（如启用远程模式） |

### `ui_schema` 字段定义（示例）

`ui_schema.fields` 建议使用数组，每个字段建议包含以下键：

| 键 | 类型 | 说明 |
|---|---|---|
| `key` | `string` | 配置字段唯一键 |
| `label` | `string` | 展示给用户的标题 |
| `type` | `string` | 字段类型（如 `text` / `select` / `switch`） |
| `required` | `boolean` | 是否必填 |
| `default` | `any` | 默认值 |

示例：

```json
{
  "ui_schema": {
    "fields": [
      {
        "key": "endpoint_url",
        "label": "服务地址",
        "type": "text",
        "required": true,
        "default": "http://127.0.0.1:8000"
      },
      {
        "key": "backend",
        "label": "后端方案",
        "type": "select",
        "required": true,
        "default": "builtin"
      }
    ]
  }
}
```

### `provides` 与 `category`

目录插件 **`manifest.json` → `provides`**：声明本插件可挂载的槽位类型（字符串数组）。宿主与 `oclive-cli plugin create --provides` 支持的合法值包括：

| `provides` 值 | 说明 |
|---------------|------|
| `memory` | 记忆检索 |
| `emotion` | 用户情绪分析 |
| `event` | 事件影响估计 |
| `prompt` | Prompt 组装 |
| `llm` | 主 LLM |
| `agent` | Agent / MCP 工具链 |
| **`complex_emotion`** | **复杂情感**（共景 `narrative_hint`）；蓝图 v2 中对应 `slot_registry` 的 `type: complex_emotion`，`backend: directory` 时须在 `provides` 中含此项 |
| **`reply_post_process`** | **Reply Post-Processor**（**独立通道** `reply_post_process` · 非六槽）；`config.json` → `reply_post_processor.backend=directory` 时须在 `provides` 中含此项；RPC `reply_post_process.process` |
| **`theater_director`** | **Theater Scene Director**（**独立通道** `theater_director` · 非六槽 · **已交付**）；`distro.oclive.toml` → `[theater].director_plugin`；开发 env `OCLIVE_THEATER_DIRECTOR_PLUGIN`；`provides` 须含此项；RPC **`theater.build_prompt`**（见下节） |
| **`voice.asr`** | **Voice ASR Input**（**独立通道** `voice.asr` · 非六槽 · **Windows 已交付**）；宿主 `chat_toolbar` + **`plugin_rpc_invoke`**（`ui_slots` 桥接）；`provides` 须含此项；RPC **`voice.probe`** / **`voice.transcribe`** / **`voice.import_model`** / **`voice.list_profiles`** / **`voice.speak`**（见 [RFC §4.1](../rfc/RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md#41-voiceasr-插件通道windows-已交付--宿主侧)） |

解析时 [`SlotResolver`](../../kernel/crates/oclive_kernel_host/src/domain/slot_resolver.rs) 会校验 directory 插件是否声明 `provides` 含目标能力（含 `complex_emotion`）。**独立通道**项由专用 Resolver 解析（如 [`resolve_reply_post_processor`](../../kernel/crates/oclive_kernel_host/src/domain/reply_post_processor.rs) 校验 `reply_post_process`；[`resolve_theater_director`](../../kernel/crates/oclive_kernel_host/src/domain/theater_director.rs) 校验 `theater_director`），**不**经六槽 `SlotResolver`。**`voice.asr`** 由宿主 UI 经 **`plugin_rpc_invoke`** 调本插件 RPC，**无**内核 `resolve_*`（调试面板仍可用 Tauri `directory_plugin_invoke`）。

#### Capability Registry v1（蓝图 v4 · 只读计划）

- `provides` 也可广告命名空间化的 v4 capability；但**只有宿主已登记消费者**时，Plan Compiler 才会选择 Provider。单独写入任意字符串不会扩张内核能力。
- 当前目录 Provider 必须通过 manifest `schema_version: 1` 校验、声明目标 capability、包含可执行 `process`，并满足插件依赖、角色级启停状态与高危授权；旧 manifest 省略 `permissions` 但含 `process` 时仍按 `process:spawn` 授权处理。
- Provider `version` 会进入诊断快照；v4 外壳当前没有 Provider API semver range，不能把版本显示误当成 API 兼容承诺。未来新增兼容字段时须先扩展本契约。
- 首个已登记的 v4 消费者是 Chat Pro `voice.asr`。其它 capability 在有真实消费者与调用链之前会结构化降级/阻断。
- 两个入口都不会 spawn Provider 或改写角色包。`oclive doctor execution-plan` / 纯 Plan Compiler 不探测设备，返回 `resource_coordination: not_evaluated` 且省略 `resource_plan`；桌面 `get_execution_plan_diagnostics` 会刷新 Resource Coordinator 并附上只读候选计划，但不会因查看诊断而执行转换或启动模型。

公共 DTO 与实现锚点见 [`models/execution_plan.rs`](../../kernel/crates/oclive_kernel_types/src/models/execution_plan.rs) · [`capability_registry.rs`](../../kernel/crates/oclive_kernel_host/src/infrastructure/capability_registry.rs) · [`execution_plan.rs`](../../kernel/crates/oclive_kernel_host/src/domain/execution_plan.rs)。

### 社区 TTS 目录插件（`com.user.tts.*`）

**非** K-VOICE-02 官方引擎产品化；**不**扩大运行时权限面或宿主全局 RPC 白名单。社区 TTS 侧车与官方 `com.oclive.voice.asr` 共用 **`voice.*` 方法命名空间**与同一执法路径（[`validate_rpc_method_for_manifest`](../../distros/desktop-tauri/src/api/plugin_bridge.rs)）。

| 项 | 契约 |
|----|------|
| **插件 ID 约定** | `com.user.tts.*`（创作者命名空间；例 `com.user.tts.xtts-sidecar`） |
| **桥接门禁** | manifest **`bridge.invoke`** 须含 **`plugin_rpc_invoke`**（见 [DIRECTORY_PLUGINS.md](DIRECTORY_PLUGINS.md)） |
| **RPC 门禁** | 方法 ∈ **本插件** manifest **`rpcMethods`**；须含 **`process`** 块；**按插件 allowlist** 执法，非宿主全局方法表 |
| **`provides`** | **无**独立 `voice.tts` token。纯 TTS 侧车**无需**声明 `voice.asr`；若同时承接 ASR UI 通道，可声明 **`voice.asr`**（与官方相同 token，**不**新增权限面） |
| **推荐 `rpcMethods`（最小）** | 至少 **`voice.speak`**；典型侧车亦声明 **`voice.probe_tts`** · **`voice.warm`** · **`voice.list_tts_adapters`**。完整 `voice.*` 列见 [RFC §4.1](../rfc/RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md#41-voiceasr-插件通道windows-已交付--宿主侧)（须在自身 manifest 逐条声明方可 invoke） |

宿主 UI 或 `ui_slots` 经 **`plugin_rpc_invoke`** 调用已声明方法；未在 `rpcMethods` 中声明的方法一律拒绝。统一资源协调当前只内置识别官方 `com.oclive.voice.asr` 的 `bundled-cosyvoice2-zh`；社区 `com.user.tts.*`、用户自建 HTTP 与云 TTS 保持各自责任边界，不会仅凭相同 `voice.*` 方法名就被冒充为宿主管理的 GPU 运行时。

### Reply Post-Processor · 润色场景（可选 · 非默认）

- **builtin**：仅格式治理（空白、引号、`max_chars`）；**不做 LLM 润色**。
- **directory / remote**：承接 **可选 LLM 润色**；契约方法 `reply_post_process.process`，参数含 `raw_reply`、`user_message`、`role_id`、`scene_id`、`locale`；返回 `display_reply` 与可选 `diagnostic`。
- **脚手架**：[`examples/reply-post-process-polish/`](../../examples/reply-post-process-polish/)（pass-through 默认；在 `rpc_server.mjs` 内替换 `polishReply` 接入你的模型）。
- **设计契约**：[RFC_USER_IDENTITY_AND_REPLY_POST_PROCESSOR](../rfc/RFC_USER_IDENTITY_AND_REPLY_POST_PROCESSOR.md)。
- **与 Prompt 分工**：生成阶段使用蓝图有效回复质量锚点（Stable v4 `runtime_config.reply_quality_anchor`；v2 兼容 `meta`）；润色在后处理阶段，默认 **`reply_post_processor.enabled: false`**。

### Theater Scene Director · `theater.build_prompt`（独立通道 · 已交付）

- **入口**：`generate_theater_scene` / `POST /theater/scene`（**不进** `process_message`）
- **配置**：`distro.oclive.toml` → `[theater].director_plugin = "com.oclive.theater_director_official"`；开发 env **`OCLIVE_THEATER_DIRECTOR_PLUGIN`**
- **directory**：`provides: theater_director`；RPC **`theater.build_prompt`**
- **params**：[`TheaterPromptBuildInput`](../../kernel/crates/oclive_kernel_contracts/src/theater_director.rs)（`mode`：`patch` | `ripple` | `cast_adapt` | `cast_rewrite` | `cast_rewrite_minimal`；persona、beats、tweak、fork 等快照字段）
- **result**：`{ "prompt": "<非空字符串>" }`（长度上限 32 768）；RPC 失败或空串 → 内核 **builtin** 模板，不 500
- **官方插件**：[`distros/chat-pro/plugins/com.oclive.theater_director_official/`](../../distros/chat-pro/plugins/com.oclive.theater_director_official/) · 最小示例 [`examples/directory-plugin-theater-director-minimal/`](../../examples/directory-plugin-theater-director-minimal/)
- **自定义 prompt pack**：Fork 官方插件 → 改 `prompts/`（入口 `prompts/index.mjs`；风格一句切换见 `drama_guardrails.mjs`）→ 新 `manifest.id` → `{app_data}/plugins/<id>/` + `[theater].director_plugin` 或 **`OCLIVE_THEATER_DIRECTOR_PLUGIN`**。详见官方插件 [`README.md`](../../distros/chat-pro/plugins/com.oclive.theater_director_official/README.md) 与 [`handoff/theater/PLAYTEST_MATRIX.md`](../../handoff/theater/PLAYTEST_MATRIX.md)。
- **放置指南**：[PLUGIN_PLACEMENT_GUIDE.md](PLUGIN_PLACEMENT_GUIDE.md)

**`category`**（单值，可选）：供插件工作台左栏分类，建议与 `provides` 主槽一致，例如 `llm`、`complex_emotion`。

### `description_zh` 字段

用于卡片大白话展示，面向创作者与普通用户，建议一句话说明“这项配置会影响什么”。

示例：

```json
{
  "description_zh": "决定回复由本地模型还是远程服务生成。"
}
```

### 完整 manifest 示例（节选）

```json
{
  "id": "example.llm.remote.bridge",
  "name": "示例 LLM 远程桥",
  "version": "0.1.0",
  "category": "llm",
  "description_zh": "用于把回复生成切换到远程 HTTP 服务。",
  "ui_template": "endpoint-config",
  "ui_schema": {
    "fields": [
      {
        "key": "endpoint_url",
        "label": "服务地址",
        "type": "text",
        "required": true,
        "default": "http://127.0.0.1:8000"
      }
    ]
  },
  "plugin_backends": {
    "llm": "remote"
  }
}
```

### `RoleInfo` / `RoleData` 与本地 HTTP `POST /chat`

- Tauri **`get_role_info`**（`GetRoleInfoRequest`，可选 **`session_id`**）、**`load_role`** 返回体含 **`personality_source`**：JSON 字符串 **`vector`** | **`profile`**，与角色包 **`evolution.personality_source`** 一致（见 `kernel/crates/oclive_kernel_types/src/models/dto/`）。
- 启动参数 **`--api`** 时，**`POST /chat`** 成功响应在扁平化的 `SendMessageResponse` 字段之外另含 **`personality_source`**（同上），便于编写器试聊等工具区分人格模式；实现见 `kernel/crates/oclive_kernel_host/src/http_api.rs`。
- Remote **`prompt.build_prompt`**：`params` 中含完整 **`role`**（其 `evolution_config.personality_source` 亦可读），并另含顶层 **`personality_source`** 与 `personality` 并列，侧车无需仅从嵌套 `role` 解析（`kernel/crates/oclive_kernel_host/src/infrastructure/remote_plugin/prompt_http.rs`）。

---

## 权限规范（目录插件 · A4.2）

目录插件 **`manifest.json`** 可选字段 **`permissions`** 声明宿主侧需启用的高危能力。校验 crate **`oclive_validation::plugin_permissions`** 与运行时 **`high_risk_grants.json`** 使用**同一套权限标识**；运行时实际检查的标识为权威来源。

| 权限标识 | 说明 | 是否需要用户授权 | 默认值 |
|----------|------|------------------|--------|
| `process:spawn` | 允许宿主为该插件 spawn 子进程（`process` 块） | 是 | 未授权 |
| `network:*` | 允许 Remote 后端或插件侧出站 HTTP（见下） | 是 | 未授权 |
| `mcp:http` | 允许 MCP server `transport=http` 出站 | 是（按 server `id`） | 未授权 |
| `mcp:stdio` | 允许 MCP server `transport=stdio` 子进程 | 是（按 server `id`） | 未授权 |

### `manifest.json` 格式

```json
{
  "schema_version": 1,
  "id": "com.example.myplugin",
  "version": "1.0.0",
  "permissions": ["process:spawn", "network:*"],
  "process": {
    "command": "node",
    "args": ["rpc_server.mjs"]
  }
}
```

- **`permissions` 省略**：视为 **`[]`**，校验**不报错**（无显式高危声明）。
- **旧版兼容**：若省略 `permissions` 且存在 **`process`** 块，宿主仍按 **`process:spawn`** 路径要求用户在 **`high_risk_grants.json`** 中授权该插件 `id`（与 A4.1 行为一致）；**新插件应显式声明** `process:spawn`。
- **Remote 侧车**：`plugin_backends.* = remote` 且配置了 `OCLIVE_REMOTE_*` 时，出站 JSON-RPC 前检查 **`network:*`**；grant **`id`** 为 **`remote:plugin`**（共用 plugin 端点）或 **`remote:llm`**（LLM 端点）。
- **MCP**：与目录插件 manifest 无关；按 `{app_data}/mcp-servers/*.json` 的 server **`id`** 检查 **`mcp:http`** / **`mcp:stdio`**。
- **持久化**：`{app_data}/high_risk_grants.json` 顶层键与权限标识一致（如 `"process:spawn": ["com.example.myplugin"]`）。Tauri **`grant_high_risk_capability`** 的 `kind` 接受规范标识；旧键名 `mcp_http` 等仍可读。

### `slot_attachment`（可选 · 自动装配蓝图）

插件作者在 **`manifest.json`** 中声明 **`slot_attachment`**（单对象或数组），安装时由 **`oclive plugin install <id> --role <pack-dir>`** 写入角色包 **`pipeline.ocblueprint` → `slot_registry`**。主应用默认**不**展示架构图；高级槽位编辑见 **`oclive plugin manage`**（含 **`--tui`**）。

```json
{
  "provides": ["llm"],
  "slot_attachment": {
    "type": "llm",
    "backend": "directory",
    "label": "llama.cpp LLM",
    "position": 6
  }
}
```

| 字段 | 说明 |
|------|------|
| `type` | 注册表类型：六种稳定槽 `memory` / `emotion` / `event` / `prompt` / `llm` / `agent`，或设施类型 `complex_emotion`；后者不是第七稳定槽 |
| `backend` | 可选，默认 `directory`；必须属于对应 `type` 的既有 backend 枚举，安装前按最终蓝图同一规则校验 |
| `label` | 可选，蓝图实例展示名 |
| `position` | 可选，实例排序；缺省 `0` |

校验：`kernel/crates/oclive_validation/src/plugin_slot_attachment.rs`。未声明 `slot_attachment` 时仅复制插件目录，需手动 **`oclive plugin manage link`**。

`openai_compatible` 是 LLM Remote 实现方式，不是 `slot_registry.backend` 枚举；此类插件应声明 `backend: "remote"`，端点与协议按 [REMOTE_PLUGIN_PROTOCOL.md](REMOTE_PLUGIN_PROTOCOL.md) 配置。自动装配不得生成最终蓝图无法通过的 backend。

### 主应用：极简插件管理（唯一入口）

**Ctrl+Shift+F** 打开**已安装插件列表**（名称、版本、开关、卸载；**安装插件** / **浏览市场**）。**不**包含架构图、蓝图编辑或槽位连线。

**UI 插槽**：`manifest.json` 的 **`ui_slots`**（宿主扫描为 catalog 的 `uiSlotNames`）在**启用插件**时弹出 **位置选择**；用户勾选后写入 `disabled_slot_contributions` / `slot_order`（见 `plugin_state`）。无 `ui_slots` 声明则启用时不弹窗。

**高级管理**：`oclive plugin manage`（可选 `--tui`）；`slot_attachment` + `plugin install --role` 自动装配蓝图。V1/V2/架构图 Vue 组件**保留源码、不在主应用挂载**。

### `plugin_dependencies`（可选）

目录插件 `manifest.json` 可增加 **`plugin_dependencies`**：字符串数组，列出须先安装的其它插件 **`id`**。CLI **`oclive plugin install <id>`** 会拓扑排序后按序复制安装；**循环依赖**报错；**`plugin uninstall`** 若仍有其它插件声明依赖该 id 会给出警告。

```json
{
  "id": "com.example.aggregator",
  "plugin_dependencies": ["com.oclive.example.llamacpp_llm"]
}
```

校验：`kernel/crates/oclive_validation/src/plugin_dependencies.rs`。

实现与测试：`kernel/crates/oclive_validation/src/plugin_permissions.rs`、`kernel/crates/oclive_kernel_host/src/infrastructure/high_risk_grants.rs`、集成测 `distros/desktop-tauri/tests/permission_three_way_consistency.rs`。

### 发布到社区索引（GitHub · 链接策展）

目录插件上架 **不** 走 Supabase 社区站表单，也 **不向 oclive 官方上传插件包**。阶段 A：**作者在 Git 托管源码**，索引只登记 **`git` 链接**，由维护者 PR 审核后写入 `plugins.json`；用户粘贴分享链接在桌面 **插件市场** 加载目录或安装单仓。

| 步骤 | 说明 |
|------|------|
| 1 | 插件以独立 Git 仓库发布，或 monorepo 子目录 + 索引 **`gitSubdir`**；仓库内 **README / manifest 须写清**（见 **[PLUGIN_MARKET_SUBMISSION.md](PLUGIN_MARKET_SUBMISSION.md)**） |
| 2 | 向主仓 [`data/plugins.json`](../../data/plugins.json) 提 PR，字段与 `PluginIndexEntry` 对齐 |
| 3 | 合并后同步 [awesome-oclive-plugins](https://github.com/linkaiheng2233-cyber/awesome-oclive-plugins) 的 `plugins.json` |
| 4 | 将 **plugins.json 的 raw 链接** 或 **仓库链接** 发给用户；用户在 **插件市场** 粘贴加载，或用 **`oclive market install <id>`** |

投稿责任、审核清单与用户粘贴链接流程：**[PLUGIN_MARKET_SUBMISSION.md](PLUGIN_MARKET_SUBMISSION.md)**。  
维护与字段全文、环境变量、缓存路径：**[../../handoff/GITHUB_PLUGIN_INDEX_LINE.md](../../handoff/GITHUB_PLUGIN_INDEX_LINE.md)**。

本地已安装插件 discovery：`oclive plugin search [--provides <slot>] [keyword] -o <plugins-dir>`（按 `manifest.json` 的 **`provides`** 过滤）。

---

[English](../../creator-docs-en/plugin-and-architecture/PLUGIN_V1.md)
