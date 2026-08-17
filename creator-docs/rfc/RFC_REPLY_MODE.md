# RFC：回复模式（Reply Mode）— 独立通道能力增强

| 元数据 | 值 |
|--------|-----|
| 状态 | **Draft v1**（设计已确认，本地回归已覆盖 · 真机体验与分隔符命中率待人工验证） |
| 受众 | 内核 / 前端 / 编写器 / 角色包作者 |
| 前置 | [RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md](RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md) · [MODULE_MAP_AND_HANDOFF.md](../../handoff/MODULE_MAP_AND_HANDOFF.md) · [ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md) |
| 权威中文名 | **回复模式** |
| 权威英文名 | **Reply Mode**（注册表 id `reply_mode`） |

[English summary in §0](#0-english-summary)

---

## 0. English summary

`reply_mode` is a **side-channel capability enhancement module** that lets a role pack declare how one LLM generation is presented as one or more assistant message segments. v1 ships `single` (default) and `burst` (N segments split on a configurable line-only separator, with optional per-segment display delays).

It is **not** a six-slot backend and **not** a numbered facility submodule. It hooks into the Stable turn chain at `post_llm`, after `reply_post_process`, and only transforms reply presentation. The separator protocol is injected into the generated prompt by the host, so role persona text does not hard-code protocol details.

---

## 1. 定位与分类

| 大类 | 占六槽？ | 设施子模块号？ | 接入 |
|------|----------|----------------|------|
| 第 1–6 模块 | 是 | — | `PluginHost` → `process_message` |
| 设施 ①–④ | 否 | 是 | `turn_pipeline` 编排行内 |
| **`reply_mode`** | **否** | **否** | `turn_pipeline/post.rs` · post_llm 之后 · 自有 resolver |

归类依据与 `reply_post_process` 一致：两者都是“LLM 输出后、用户看到前”的回复加工，但职责不同。`reply_post_process` 负责文本润色，`reply_mode` 负责分段与展示节奏，因此注册为新的独立通道 id，而不是复用或吞并后处理通道。

**默认行为**：未配置或 `mode = "single"` 时完全等于现状，单条助手消息，单气泡。

---

## 2. 配置 schema（角色包 `config.json`）

```json
{
  "reply_mode": {
    "mode": "burst",
    "segments": 2,
    "separator": "+++",
    "delays_ms": [0, 300],
    "streaming": "live"
  }
}
```

| 字段 | 类型 | 默认 | 说明 |
|------|------|------|------|
| `mode` | `"single"` \| `"burst"` | `single` | v1 支持两种模式 |
| `segments` | uint | `2` | 期望段数，1 等价于 single；`burst` 上限 8 |
| `separator` | string | `"+++"` | 段间协议标记；按“单独占一行且整行等于该值（或该值后仅跟结尾标点）”匹配 |
| `delays_ms` | uint[] | `[0, 0]` | 每段显示前的视觉延迟；首段必须为 0；长度不足按 0 补齐，超出截断 |
| `streaming` | `"live"` \| `"batch"` | `live` | 前端流式拆分或生成完成后一次展示 |
| `fallback_leads` | string[] | `[]` | 弱模型降级切分用的自然双发引子词（如 `——`、`而且`）；仅当模型未输出分隔符协议时使用，不注入提示词、不暴露前端 |

**分隔符校验**：

- 去除首尾空白后非空
- 不包含 `\r` 或 `\n`
- 长度不超过 16 个 Unicode 字符
- 不能是纯空白

校验失败时该角色包回退为 `single`，并在角色加载诊断中提示，不静默猜测分隔符。

---

## 3. 输出协议与提示词注入

分隔符协议由**宿主注入**，不写在角色人设里。角色包 `reply_mode.enabled` 时，宿主在组装提示词时追加：

```text
【输出格式要求】
你的回复必须分成 N 段。每一段写完后必须换行，单独输出一行分隔符（这一行只有分隔符本身，不允许添加任何文字或标点）：
<separator>
然后再换行写下一段。绝对不允许把各段连成一段，绝对不允许省略分隔符这一行。
```

这样任何角色包开启该模式都会自动获得协议，`separator` 修改后无需改人设文件。角色包人设可以描述“两次发射”的语气节奏，并提及“按系统给出的分隔符分开”，但不承载分隔符的具体值。

---

## 4. 流水线顺序

```text
pre / build_prompt
  → 宿主按 reply_mode 追加【输出格式要求】
  → 六槽 LLM 生成
  → emo/adult 解析
  → 普通共景回合先切分并剥离协议标记
  → 情绪策略 / 人格演化 / short_term_memory 使用无标记正文
  → reply_post_process 润色
  → reply_mode 对最终展示文本再次权威切分
  → 聊天落库（一条助手消息 + 段元数据）
  → SendMessageResponse
  → 前端按段渲染 / 语音按顺序朗读
```

最终展示切分仍放在 `reply_post_process` 之后，保证润色器看到完整文本、用户只看到最终切分结果。与此同时，内核会在情绪策略、人格演化和短期记忆消费 LLM 正文前先生成一份无协议标记的语义正文；因此 `+++` 不会因持久化顺序较早而进入状态或下一轮上下文。

---

## 5. 切分语义（纯函数）

输入 `raw`、`separator`、`segments`、`fallback_leads`：

1. 统一 `\r\n` 为 `\n`。
2. 某行去除首尾空白后与 `separator` 完全相等、等于 `separator` 加仅含结尾标点（`。！？.!?` 等）的后缀、或以“句末标点后紧跟 `separator`”结尾时，视为边界并剥离标记；其余内容（含 `C+++`、`a +++ b`、`+++abc`）不是边界。
3. 每段去除首尾空白；空段丢弃。
4. 段数超过 `segments` 时，超出部分合并进最后一段。
5. 未出现分隔符边界时按降级链切分（弱本地模型常省略协议）：先按空行分段；仍只有一段时，若角色包声明了 `fallback_leads`，在“句末标点或行首后紧跟引子词”的位置切出第二发；全部无效则保持单段。
6. 无边界且降级链未命中时返回一段，即整条回复；这是“第二发迟到”等例外情况的天然降级。
7. `raw` 全空时返回空列表。

前端实时流只镜像上述主协议边界（独立分隔符行、分隔符后仅结尾标点、句末标点后紧跟分隔符）；空行与 `fallback_leads` 降级由后端在完整回复上权威执行，避免流尚未结束时误切普通段落。

---

## 6. 落库与 DTO

**存储**：仍是一条助手消息。

- `chat_messages.content`：去掉分隔符后的完整回复，段与段之间以一个换行分隔，供搜索、导出、记忆和下一轮上下文使用。
- `chat_messages.metadata.reply_segments`：`["第一发", "第二发"]`；`reply_segment_delays_ms`：`[0, 300]`。前者供历史加载还原气泡，后者保留本轮展示节奏快照。
- `short_term_memory.bot_reply`：普通共景回合在任何原子写入前使用去掉分隔符的语义正文；成人与远程分支不启用 `reply_mode`。
- `message_count`、撤回、重新生成、删除仍按一轮整体处理，不新增表。

**DTO**：

- `SendMessageResponse.reply`：保持兼容，存去掉分隔符后的完整回复。
- 新增可选 `reply_presentation`：

```json
{
  "segments": ["第一发", "第二发"],
  "delays_ms": [0, 300]
}
```

- `RoleInfo` 暴露只读 `pack_reply_mode`（`mode` / `segments` / `separator` / `delays_ms` / `streaming`），供前端流式拆分在收到首个 token 前获知分隔符。

---

## 7. 前端

- `streaming = "live"`：第一个气泡正常流式；累计文本出现有效分隔符边界后，剥离标记并依次启动后续气泡（最多 8 段），按 `delays_ms` 顺序延迟展示。
- `streaming = "batch"`：仍使用 SSE 传输，但完整响应到达前不创建助手气泡；随后按后端 `reply_presentation` 展示并执行段间延迟。
- 历史加载：`chat_messages.metadata.reply_segments` 展开为多个气泡，同一基础消息 id 加段序号。
- 旁白：先分段，再对每段执行现有“对话/旁白”拆分；对话留在对应气泡，旁白汇总到本轮叙事条。
- 语音：普通单条回复保留低延迟流式朗读；启用 `reply_mode` 时不朗读原始 SSE 片段，等后端返回权威最终回复后只朗读去掉分隔符的完整正文，从源头避免读出 `+++` 或因前缀不匹配整段重播。
- 流式失败回退阻塞 `/chat` 时，使用响应 `reply_presentation` 渲染，行为一致。

---

## 8. 降级与非目标

- Agent 快捷回复、降级短句、远程生活轨迹、成人 staged beat 保持单段，不经过 `reply_mode`。
- 模型未输出分隔符或只输出一段时，优雅回退为单条回复。
- 分隔符校验失败回退 `single`。
- v1 不新增六槽类型，不占设施编号，不新增数据库表。
- v1 不改动 `short_term_memory` 结构；记忆写去掉分隔符后的完整回复。

---

## 9. 扩展：用户自定义回复模式

`separator`、`segments`、`delays_ms`、`streaming` 均可由角色包配置修改，`+++` 只是默认值。后续需要新语义（如分支回复、多角色合声）时：

1. 扩展 `mode` 枚举与对应的宿主策略；
2. 若需要第三方实现，按 [RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md](RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md) 增加 `provides: ["reply_mode"]` 的目录插件后端，内置 `burst` 保持为默认实现。

---

## 10. 验收

- [x] 未配置 `reply_mode` 的普通回复保留原流式气泡与低延迟流式语音
- [x] `burst` + `+++` 生成输出切分正确，分隔符不出现在响应、聊天记录或短期记忆中
- [x] 分隔符缺失、超段、空段、CRLF、全角分隔符均有纯函数单测
- [x] 一条助手消息落库，`metadata.reply_segments` 可还原任意 2–8 个兄弟气泡
- [x] `live`、`batch` 与阻塞回退均以最终 `reply_presentation` 收口；三段实时顺序和历史重载已有测试
- [ ] 撤回 / 重新生成 / 删除的一轮整体交互待真机回归确认（存储仍为一条助手消息）
- [x] 回复模式的流式语音被抑制，`message:sent.reply` 只携带后端清洗正文；普通回复流式语音不回退
- [ ] 多段同时含旁白时的最终叙事条汇总待真机回归确认
- [x] `RoleInfo.pack_reply_mode` 只读透传，前端不硬编码角色 id
- [x] MODULE_MAP §11 与 RFC_SIDE_CHANNEL 注册表登记 `reply_mode`

---

## 11. 参考锚点

| 主题 | 路径 |
|------|------|
| post 锚点 | `kernel/crates/oclive_kernel_host/src/domain/chat_engine/turn_pipeline/post/post_llm.rs` |
| 回复后处理独立通道 | `kernel/crates/oclive_kernel_host/src/domain/reply_post_processor.rs` |
| 角色包配置模型 | `kernel/crates/oclive_kernel_types/src/models/role_pack_config.rs` |
| 聊天落库 | `kernel/crates/oclive_kernel_host/src/infrastructure/chat_storage/chat_messages.rs` |
| 前端发送 | `distros/shared/src/stores/chatStoreSend.ts` |
| 前端历史加载 | `distros/shared/src/stores/chatStoreLoad.ts` |
| 旁白拆分 | `distros/shared/src/utils/roleplayReplySplit.ts` |
