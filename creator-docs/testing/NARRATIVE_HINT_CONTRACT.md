# `narrative_hint` 全链路契约（AB1）

**状态**：与 `oclive_kernel_host` 共景路径、`oclive_kernel_runtime::PromptBuilder` 实现一致。

## 1. 数据形状

| 阶段 | 类型 / 存储 | 字段 |
|------|-------------|------|
| 解析输入 | `ComplexEmotionInput` | `previous_narrative_hint: String`（上一轮缓存，首轮为空） |
| 解析输出 | `ComplexEmotionOutput` | `narrative_hint: String` |
| 会话缓存 | `AppState::last_complex_emotion_narrative_hint` | `HashMap<srid, String>`（进程内，**非** SQLite） |
| Prompt 输入 | `PromptInput::previous_complex_emotion_narrative_hint` | `&str` |

## 2. 调用顺序（单轮 `process_message` / 共景）

1. `load_recent_context`
2. 读取 `stored_complex_emotion_narrative_hint(srid)` → 作为本回合 `ComplexEmotionInput.previous_narrative_hint`
3. `ComplexEmotionProvider::resolve_turn` → `ComplexEmotionOutput`
4. `build_prompt`（`previous_complex_emotion_narrative_hint` = 步骤 2 的快照）
5. 主对话 LLM
6. `set_stored_complex_emotion_narrative_hint(srid, complex_emotion_out.narrative_hint)`

**不变量**：步骤 4 使用的是**上一轮**写入的 hint，不是本回合刚算出的 hint。

## 3. Prompt 注入规则（`PromptBuilder`）

- 当 `previous_complex_emotion_narrative_hint.trim().is_empty()` 时：**不**输出【复杂情感叙事提示】段。
- 非空时插入固定标题行 + `trim()` 后的正文 + 双换行，再接 `用户说:` 段。
- 标题文案：`【复杂情感叙事提示】（上一回合内置分析输出；自然落实，勿向用户复述本段标题或元信息）`

## 4. 自动化验证

| 用例 | 位置 |
|------|------|
| 首轮主 Prompt 无叙事段 | `distros/desktop-tauri/tests/narrative_hint_contract_audit.rs` |
| 次轮注入上一轮 hint | 同上 + 既有 `narrative_hint_prompt_roundtrip.rs` |
| 连续三轮后第三轮含【复杂情感叙事提示】 | `narrative_hint_contract_audit.rs` |
| 空 hint / 特殊字符不破坏结构 | `oclive_kernel_runtime` `prompt_builder` 单元测试 |

## 5. 与 Remote 侧车

Remote `complex_emotion.resolve_turn` 的 JSON 须与 `ComplexEmotionOutput` 同形；降级时 `degraded_to_builtin: true`。侧车错误格式见 [ERROR_CODES.md § 分层边界](../getting-started/ERROR_CODES.md)。

[English mirror](../../creator-docs-en/testing/NARRATIVE_HINT_CONTRACT.md)
