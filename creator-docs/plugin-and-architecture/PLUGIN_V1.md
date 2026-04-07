# PLUGIN_V1 — 编排层契约与后端枚举

本文档描述宿主（Tauri / `chat_engine`）与可替换子系统之间的 **v1 契约**：类型命名、DTO 形状、`settings.json` 中的后端枚举。实现以源码为准：`src-tauri/src/domain/*_*.rs`、`src-tauri/src/models/plugin_backends.rs`。

**全库文档索引**：[../getting-started/DOCUMENTATION_INDEX.md](../getting-started/DOCUMENTATION_INDEX.md)。包版本与 `schema_version` 见 **[../role-pack/PACK_VERSIONING.md](../role-pack/PACK_VERSIONING.md)**。HTTP 侧车 JSON-RPC 全文见 **[REMOTE_PLUGIN_PROTOCOL.md](REMOTE_PLUGIN_PROTOCOL.md)**；创作者总览见 **[CREATOR_PLUGIN_ARCHITECTURE.md](CREATOR_PLUGIN_ARCHITECTURE.md)**。

## 设计约束

- **v1 插件 = 编译期枚举**：通过 `settings.json` 选择实现，无动态 `cdylib`。
- **默认实现**即当前内置逻辑；换后端时 **API 字段名不变**（尤其 `SendMessageResponse.reply`）。
- **Remote**：宿主已实现 **HTTP JSON-RPC**（见 [REMOTE_PLUGIN_PROTOCOL.md](REMOTE_PLUGIN_PROTOCOL.md)）；未配置 `OCLIVE_REMOTE_*` URL 时回退 **builtin**（或进程内 LLM）并写日志。

---

## `send_message` 编排顺序（与 `chat_engine`）

共景主路径见源码 [`chat_engine/co_present.rs`](../../src-tauri/src/domain/chat_engine/co_present.rs) 的 `process_co_present`。入口为 [`chat_engine::process_message`](../../src-tauri/src/domain/chat_engine/mod.rs)（异地分支为 `process_remote_stub` / `process_remote_life`，事件链有简化）。与 **PLUGIN_V1** 子系统相关的顺序如下（与 DTO 流一致）：

1. **`PluginHost`**：[`state::resolved_plugins_for`](../../src-tauri/src/state/mod.rs) → [`PluginHost::resolve_for_role`](../../src-tauri/src/domain/plugin_host.rs)，按 `role.plugin_backends` 绑定 `memory` / `emotion` / `event` / `prompt` / `llm`。
2. **用户情绪**：`pl.emotion.analyze` → `EmotionResult`，对外为响应中的 `emotion`（`EmotionDto`）。
3. **人格微调**：`PersonalityEngine::adjust_by_user_emotion`（消费用户情绪，非独立 trait 子系统）。
4. **知识块**（可选）：包内 `knowledge_index` 检索；可与事件估计的 augment 合并。
5. **事件影响**：`pl.event.estimate` → `EventImpactEstimate`；随后 `PersonalityEngine::evolve_by_event`。
6. **记忆检索**：仓储读出候选 → 场景加权 → `pl.memory.rank_memories`（`MemoryRetrievalInput`）。
7. **好感与关系阶段**：`compute_favor_and_relation`（输入含事件类型与影响因子等）。
8. **Prompt**：`pl.prompt.top_topic_hint` + `pl.prompt.build_prompt`（`PromptInput`）。
9. **主 LLM**：`pl.llm.generate` 等；后续含 bot 侧情绪、立绘、短期记忆写入、位移意图等（见同文件后半段）。

门面与枚举的单一事实来源：`plugin_host.rs`、`models/plugin_backends.rs`、本文各节表格。

---

## 记忆检索 `MemoryRetrieval`

### 输入：`MemoryRetrievalInput`

| 字段 | 类型 | 说明 |
|------|------|------|
| `memories` | `&[Memory]` | 已由仓储读出并经场景权重等处理后的候选集 |
| `user_query` | `&str` | 当前用户句（用于 builtin_v2 等关键词加权） |
| `scene_id` | `Option<&str>` | 当前场景 id；可选参与未来检索策略 |
| `limit` | `usize` | 注入主 prompt 的最大条数 |

### 输出

- **排序后的** `Vec<Memory>`，长度不超过 `limit`。
- 结构化上下文 `MemoryContext`（`build_context`）与 `models::MemoryContext` 一致：`memories` + `total_tokens` 估计。

### 后端枚举 `memory`（`settings.json` → `plugin_backends.memory`）

| 值 | 含义 |
|----|------|
| `builtin` | 按 `importance * weight` 排序取 Top-K（与历史 `MemoryEngine::get_relevant_memories` 一致） |
| `builtin_v2` | 在 builtin 思路上增加 **用户查询与正文** 的轻量重合加权（第二套内置） |
| `remote` | HTTP `memory.rank`（需 `OCLIVE_REMOTE_PLUGIN_URL`；失败回退 `builtin`） |

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
| `builtin_v2` | 第二套内置：强中性输出（`BuiltinUserEmotionAnalyzerV2`；用于验证枚举可切换） |
| `remote` | HTTP `emotion.analyze`（需 `OCLIVE_REMOTE_PLUGIN_URL`；失败回退 builtin） |

---

## 事件影响 `EventEstimator`

### 输入（概念）

与 `estimate_event_impact` 一致：`LlmClient`、`ollama_model`、用户句、`Emotion`、`PersonalityVector`、近期轮次与事件列表。

### 输出：`EventImpactEstimate`

- `event_type: EventType`
- `impact_factor: f64`
- `confidence: f32`

### 后端枚举 `event`

| 值 | 含义 |
|----|------|
| `builtin` | 现有 `event_impact_ai::estimate_event_impact` 链（含环境开关与规则回退） |
| `builtin_v2` | 第二套内置：在 builtin 结果上将 `impact_factor` ×0.5（`BuiltinEventEstimatorV2`） |
| `remote` | HTTP `event.estimate`（需 `OCLIVE_REMOTE_PLUGIN_URL`；失败回退 builtin） |

---

## Prompt 组装 `PromptAssembler`

### 输入 / 输出

- 输入：`PromptInput`（与 `PromptBuilder` 一致，**五个参数级字段**在 `PromptInput` 结构体上；`build_prompt` 最后一参在实现内部为 `&PromptInput`）。
- 输出：`String`（主对话 system/user 拼装结果）。

附加：`top_topic_hint(role, scene_id) -> Option<String>` 与现 `PromptBuilder::top_topic_hint` 对齐。

### 后端枚举 `prompt`

| 值 | 含义 |
|----|------|
| `builtin` | 现有 `PromptBuilder` |
| `builtin_v2` | 第二套内置：在 builtin 正文前追加 `[oclive:prompt:builtin_v2]` + 换行前缀（`BuiltinPromptAssemblerV2`） |
| `remote` | HTTP `prompt.build_prompt` / `prompt.top_topic_hint`（需 `OCLIVE_REMOTE_PLUGIN_URL`；失败回退 builtin） |

---

## 主对话 LLM `LlmClient`

### 职责

- `generate`：主回复、异地模式、独白等所有「生成型」调用。
- `generate_tag`：短输出分类（位移意图、立绘标签等）。

### 后端枚举 `llm`

| 值 | 含义 |
|----|------|
| `ollama` | 应用启动时注入的默认客户端（通常为 `OllamaClient` 包装） |
| `remote` | HTTP `llm.generate` / `llm.generate_tag`（需 `OCLIVE_REMOTE_LLM_URL`；未配置则委托进程内默认 LLM 并记日志） |

---

## `settings.json` 片段示例

```json
{
  "schema_version": 1,
  "plugin_backends": {
    "memory": "builtin",
    "emotion": "builtin",
    "event": "builtin",
    "prompt": "builtin",
    "llm": "ollama"
  }
}
```

省略 `plugin_backends` 时全部为 **builtin**（`llm` 默认为 `ollama`）。未知枚举值会导致角色包解析失败（须修正拼写）；未来可对字符串值做宽松别名时再文档化。

---

## 前端对齐

TypeScript 侧 `SendMessageResponse`（`src/utils/tauri-api.ts`）必须与 `models/dto.rs` 一致：**回复字段名为 `reply`**；`presence_mode`、`reply_is_fallback`、`schema`、`api_version` 用于展示策略（见 `src/utils/replyPresentation.ts`）。
