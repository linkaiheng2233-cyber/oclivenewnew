# 实验核 Method 注册表（pipeline.experimental）

`pipeline.experimental` 中的每一步 `action` 须为：

```text
slot.<registry_key>.<method>
```

- `registry_key`：本角色 `slot_registry` 中的实例键（如 `emotion`、`llm_2`）。
- `method`：下表所列之一；**未列出则实验核返回错误并静默降级到稳定核**（`co_present`）。

稳定核 **不** 解释 `pipeline.stable`；宿主恒为 `process_co_present` 硬编码路径。

---

## 七槽 Method 一览

| `type` | `method` | 共景阶段 | 说明 |
|--------|----------|----------|------|
| `memory` | `retrieve` | `memory_rank` | 加载近期记忆并按场景加权、排序（`MemoryRetrievalInput`） |
| `emotion` | `analyze` | `user_emotion_analyze` | 分析用户消息情绪（`EmotionResult`） |
| `event` | `detect` | `event_estimate` | 估计本回合事件类型与 impact（需先有情绪/性格上下文） |
| `prompt` | `assemble` | `build_prompt` | 组装主对话 Prompt 字符串（不调用 LLM） |
| `llm` | `generate` | `llm_generate` | 标记回合须完成生成；实验链结束后走完整 `co_present`（含 LLM） |
| `agent` | `process` | `agent_process` | 调用 Agent；若 `handled` 则直接返回 Agent 回复 |
| `complex_emotion` | `resolve_turn` | `complex_emotion_resolve_turn` | 解析复杂情感 `narrative_hint`（写会话缓存） |

---

## 分 Method 说明

### `retrieve`

- **输入**：当前用户消息、`scene_id`、会话 `srid`、已解析的 `ResolvedRolePlugins`。
- **输出**：排序后的相关记忆列表（仅内存，实验步本身不写库）。
- **示例**：`slot.memory.retrieve`

### `analyze`

- **输入**：用户消息文本。
- **输出**：`EmotionResult`（供后续 `detect` / `assemble` / `resolve_turn` 使用）。
- **示例**：`slot.emotion.analyze`

### `detect`

- **输入**：用户消息、用户情绪、性格向量、近期上下文、可选知识增强。
- **输出**：事件估计（类型、impact、confidence）；可能调整性格向量（非 Profile 模式）。
- **示例**：`slot.event.detect`

### `assemble`

- **输入**：性格、记忆、关系、场景、情绪 Prompt 段、可变性格档案等（见 `PromptInput`）。
- **输出**：Prompt 字符串（缓存于实验上下文；最终仍由稳定核 `generate` 使用）。
- **示例**：`slot.prompt.assemble`

### `generate`

- **输入**：无额外参数；表示本回合须经 LLM 完成。
- **输出**：触发实验成功后调用完整 `co_present`（与今日单核回复契约一致）。
- **示例**：`slot.llm.generate`
- **注意**：实验 pipeline **须**包含至少一步 `generate`，或 Agent `process` 短路成功。

### `process`

- **输入**：`AgentInput`（role_id、session、message、model）。
- **输出**：若 Agent `handled`，直接返回 `SendMessageResponse`；否则继续后续步骤。
- **示例**：`slot.agent.process`

### `resolve_turn`

- **输入**：`ComplexEmotionInput`（含上一轮对话、`previous_narrative_hint`、七维情感指标等）。
- **输出**：`ComplexEmotionOutput`；更新 `AppState` 会话级 `narrative_hint`（失败时可被快照回滚）。
- **示例**：`slot.complex_emotion.resolve_turn`

---

## CLI 查询

```bash
cargo run -p oclive-cli -- explain DUAL_CORE
cargo run -p oclive-cli -- explain slot.emotion.analyze
```

---

## 英文

[METHOD_REGISTRY.en.md](./METHOD_REGISTRY.en.md)

## 相关文档

- [DEVELOPER_GUIDE.md](./DEVELOPER_GUIDE.md)
- [RFC 双核双态](../rfc/RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md)
- [handoff/DUAL_CORE_CURSOR_HANDOFF.md](../../handoff/DUAL_CORE_CURSOR_HANDOFF.md)
