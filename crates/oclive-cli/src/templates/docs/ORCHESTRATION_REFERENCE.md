# 编排参考（纯内核开发者）

本文说明 **Monolith / 无头内核** 场景下与主应用 `process_message` 对齐的**六段主流程**概念顺序，以及 Monolith 焊接可跳过哪些槽位。

> **桌面宿主**：oclivenewnew Tauri / `--api` HTTP 路径**不走**本文件的可变顺序；以仓库内 `src-tauri/src/domain/chat_engine/mod.rs` 的 `process_message` 为准。

English: **`ORCHESTRATION_REFERENCE.en.md`**

## 六段主流程（逻辑顺序）

1. **加载上下文** — 角色包、会话、近期消息（`load_context`）
2. **情绪与事件** — 用户情绪分析、事件估计（`analyze_emotion` / `detect_event`）
3. **记忆** — 检索与排序（`retrieve_memory`）
4. **Prompt** — 组装系统/用户消息（`build_prompt`）
5. **LLM** — 主生成（`call_llm`）
6. **后处理** — 持久化、复杂情感叙事缓存等（`post_process`）

生成项目中的 **`src/process_message_monolith.rs`** 由 `monolith.toml` 驱动，演示七焊接键（第 1–6 模块：memory / emotion / event / prompt / llm / agent；设施焊接键 `complex_emotion`）的静态或 trait 占位调用。

## 可安全调整顺序的步骤

在**自研** `process_message` 实现中，下列子步骤在数据依赖允许时可互换（需自行保证 Prompt 输入完整）：

| 可互换组 | 说明 |
|----------|------|
| `analyze_emotion` ↔ `detect_event` | 二者均主要消费用户本轮输入，互不依赖对方输出时可对调 |
| `retrieve_memory` 与情绪/事件链 | 若记忆检索不依赖情绪标签，可提前到情绪分析之前 |

## 不可破坏的硬约束

| 约束 | 原因 |
|------|------|
| **`build_prompt` 必须在 `call_llm` 之前** | LLM 需要完整 messages |
| **`load_context` 应早于依赖角色/会话状态的步骤** | 否则 Prompt 缺角色设定 |
| **`post_process` 应在 `call_llm` 之后** | 需基于模型输出写回 |

## 用 `monolith.toml` 跳过某槽

在 `[monolith]` 中：

- **`weld_modules`**：列出要**静态焊接**的槽（编译期直连 builtin）。
- **`exclude`**：与 `weld_modules` 互斥；列出要**排除焊接**、保留动态 trait 的槽。

`oclive init --monolith --monolith-preset embedded` 等档位会预填 `weld_modules`；可手改后执行 **`oclive build`** 再生 `process_message_monolith.rs`。

将某槽**既不焊入 `weld_modules` 也不列入 `exclude`** 时，生成代码对该槽使用 trait 占位路径（体积更小、延迟略高）。

## 修改入口

| 产物 | 用途 |
|------|------|
| `monolith.toml` | 焊接计划源 |
| `src/process_message_monolith.rs` | 生成物，勿手改焊接块 |
| `vendor/oclive_monolith_builtin/` | 焊接桩；可替换为真实 `oclive_*_builtin` |

完整 RFC：`creator-docs/rfc/RFC_OCLIVE_MONOLITH_MODE.md`（相对路径随克隆位置调整）。
