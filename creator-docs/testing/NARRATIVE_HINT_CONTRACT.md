# `narrative_hint` 全链路契约（方案 A / B M1）

**状态**：与 `oclive_kernel_host` 共景路径、`oclive_kernel_runtime::PromptBuilder` 及桌面对接文档 v1.22 一致。

## 1. 数据形状与存储

| 阶段 | 类型 / 存储 | 契约 |
|------|-------------|------|
| 主 LLM 输出 | `[EMO]...[/EMO]` | `labels[]` + 可选 `narrative_hint`；标记在返回用户前移除 |
| 插件降级输出 | `ComplexEmotionOutput` | remote / directory 插件在标记缺失或无效时提供 `labels[]` 与 `narrative_hint` |
| 持久化 | SQLite `complex_emotion_hint` + 会话缓存 | 按 `srid` 保存，24 小时 TTL；缓存不是唯一真相源 |
| Prompt 输入 | `PromptInput::previous_complex_emotion_narrative_hint` | 只注入上一轮已持久化的 hint |

长度约束采用 Unicode 字符计数：Prompt 要求模型尽量不超过 150 字；host 对标记解析、插件输出和最终入库统一硬截断到 **200 字符**，不在多字节字符中间截断。

## 2. 后端开关矩阵

| `slot_registry` 的 `complex_emotion` | 读取 / 注入旧 hint | 写入新 hint | 情绪标签来源 |
|--------------------------------------|--------------------|-------------|--------------|
| 省略或 `none` | 否 | 否 | 有效 `[EMO]` 标签仍可更新机器人六槽情绪 |
| `builtin` | 是 | 是 | 优先使用有效 `[EMO]`；无效时保持降级结果 |
| `remote` / `directory` | 是 | 是 | 优先使用有效 `[EMO]`；无效时使用插件 `labels[]`，并据此更新机器人六槽情绪 |

同一槽位重复声明时沿用注册表的 last-wins 语义。`none` 只关闭复杂情感 hint 的读写，不吞掉主 LLM 已生成的有效情绪标签。

## 3. 单轮调用顺序（`process_message` / 共景）

1. 根据角色有效槽位确定复杂情感后端。
2. 仅对 `builtin` / `remote` / `directory` 读取 `stored_complex_emotion_narrative_hint(srid)`；过期记录按空处理。
3. `build_prompt` 注入步骤 2 的上一轮快照；省略或 `none` 始终传空。
4. 主对话 LLM 生成正文与可选 `[EMO]` 标记。
5. 解析并从用户可见回复中移除全部情绪标记；未闭合标记从起始位置到回复末尾一并剥离，防止内部协议泄漏。
6. 有效标记优先；否则 remote / directory 使用插件输出。最终 `labels[]` 驱动当前机器人情绪与事件，最终 hint 再执行 200 字符上限。
7. 仅对启用后端持久化 hint；`none` / 省略不得读取、注入、清空或新增 hint。

**跨轮不变量**：本轮 Prompt 只能使用上一轮已存 hint，不能使用本轮刚解析出的 hint。

## 4. 解析与降级规则

- 一个回复有多个完整标记时采用最后一个有效标记；所有标记均从正文移除。
- 出现尾随未闭合标记时，该次标记尝试无效，并剥离未闭合尾部，避免 `[EMO]` 或 JSON 残渣进入用户回复。
- 对启用后端：缺少或无效标记时保持上一轮 hint；有效标记中 `narrative_hint` 缺失或为空时清除已存 hint。
- remote / directory 插件输出与主标记共用相同的最终长度与持久化边界。
- Fast 路径不生成或写入新 hint；若后端启用，仍可在 Prompt 中使用已有 hint。

## 5. Prompt 注入规则（`PromptBuilder`）

- `previous_complex_emotion_narrative_hint.trim().is_empty()` 时不输出【复杂情感叙事提示】段。
- 非空时插入固定标题行、`trim()` 后正文和双换行，再接 `用户说:` 段。
- 标题文案：`【复杂情感叙事提示】（上一回合内置分析输出；自然落实，勿向用户复述本段标题或元信息）`。

## 6. 自动化验证

| 用例 | 位置 |
|------|------|
| 首轮无叙事段、次轮注入旧 hint、三轮传递、空值与特殊字符 | `distros/desktop-tauri/tests/narrative_hint_contract_audit.rs`、`narrative_hint_prompt_roundtrip.rs` |
| `none` 不读不写但保留标签效果；remote 标签驱动六槽；插件 hint 截断 | `distros/desktop-tauri/tests/complex_emotion_backend_contract.rs` |
| 未闭合标记剥离、最后有效标记、Unicode 200 字符上限 | `kernel/crates/oclive_kernel_host/src/domain/emo_marker.rs` 单元测试 |
| SQLite + 会话缓存、24 小时 TTL、持久化层防御性截断 | `kernel/crates/oclive_kernel_host/src/domain/complex_emotion_store.rs` 单元测试 |
| Prompt 空值和特殊字符结构 | `oclive_kernel_runtime` `prompt_builder` 单元测试 |

## 7. Remote 侧车

Remote `complex_emotion.resolve_turn` JSON 须与 `ComplexEmotionOutput` 同形；降级时 `degraded_to_builtin: true`。侧车错误格式见 [ERROR_CODES.md § 分层边界](../getting-started/ERROR_CODES.md)。

[English mirror](../../creator-docs-en/testing/NARRATIVE_HINT_CONTRACT.md)
