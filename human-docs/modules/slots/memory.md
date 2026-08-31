# 六槽开工包 · `memory`

> **读者**：改记忆检索、STM/LTM 策略或 memory 后端的工程师。  
> **读完能做什么**：分清聊天日志/STM/LTM，并理解 memory 如何提出回想事件、被采纳后影响本轮回复。
> **耗时**：约 **50 min**  
> **SSOT 范围**：人类 checklist；定义见 [MODULE_MAP §4](../../../handoff/MODULE_MAP_AND_HANDOFF.md)
> **最后更新**：2026-08-31
> **下一篇**：[chat-storage](../side-channels/chat-storage.md) · [side-channels 索引](../side-channels/)

---

## 1. 你插在哪

- **MODULE_MAP**：[§4 第 1 模块 · `memory`](../../../handoff/MODULE_MAP_AND_HANDOFF.md#4-第-1-模块--memory)
- **`plugin_backends` 键**：`memory`  
- **Trait**：`MemoryRetrieval`（`oclive_kernel_contracts`）  
- **主链 hook**：`turn_pipeline/pre.rs` 检索 · `post_llm` 写入 STM/LTM
- **Event Ring hook**：`co_present/run_middle.rs` 从本轮相关记忆选候选 → `memory.recall.candidate` → `memory.recollection.activated`（可能无）

---

## 2. 边界

| 能改 | 禁止 |
|------|------|
| 检索算法、decay、archive 阈值、`remote` / `directory` / `local` 协议 | 用 `chat_messages` 当真源；删聊天清记忆表 |
| `BuiltinMemoryRetrieval` + `MemoryEngine` 参数 | 角色任务改 `slot_registry`（G1） |
| 多 memory 实例 **去重合并** 检索 | 共景路径通常 **禁止** `none`（见 MODULE_NONE_SEMANTICS） |

**三套存储**：聊天日志 ≠ `short_term_memory` ≠ `long_term_memory` — 深读 [CHAT_STORAGE_ARCHITECTURE](../../../handoff/CHAT_STORAGE_ARCHITECTURE.md)，勿与本包重复整表。

memory 只有候选提案权：事件信封只携带记忆 ID 与分数，决策模块采纳后，正文才在 Ring 外按 ID 回取并进入单轮回想 Prompt。memory 不能自行宣布“角色已经想起”。完整边界见 [EVENT_RING](../../../creator-docs/plugin-and-architecture/EVENT_RING.md)。

---

## 3. 阅读清单

1. [MODULE_MAP §4](../../../handoff/MODULE_MAP_AND_HANDOFF.md#4-第-1-模块--memory)
2. [CHAT_STORAGE_ARCHITECTURE](../../../handoff/CHAT_STORAGE_ARCHITECTURE.md)
3. [01 简架构 §记忆三套](../../01_ARCHITECTURE_SIMPLE.md#记忆三套存储最易混--必背)  
4. [CROSS_HOST_MEMORY](../../../creator-docs/role-pack/CROSS_HOST_MEMORY.md)（跨宿主时）
5. 迁移 SSOT：`kernel/crates/oclive_kernel_host/migrations/001_init.sql` — `short_term_memory` · `long_term_memory`
6. [EVENT_RING](../../../creator-docs/plugin-and-architecture/EVENT_RING.md) — 回想提案、采纳与隐私边界

---

## 4. 开发流程

- [ ] 说清本改动影响 ② STM / ③ LTM 哪一侧（不进 ① 聊天日志）  
- [ ] 改检索 → `pre.rs` / `MemoryEngine`；改写入 → `post_llm` / archive 策略  
- [ ] 换 backend → 蓝图 + [SLOT_BACKEND_REALITY_MATRIX](../../../handoff/SLOT_BACKEND_REALITY_MATRIX.md)
- [ ] 「记忆回放」走 `replay_memory_extraction`，不覆盖 LTM 全文  
- [ ] domain 单测优先于端到端  
- [ ] `npm run check:rust`

---

## 5. 验收

- [ ] UI 删聊天记录后 STM/LTM 仍按设计保留或衰减  
- [ ] Prompt 中记忆段落来自 memory 槽检索，非直接拼 chat 行  
- [ ] 多 memory 实例检索去重合并  
- [ ] Fast Turn Thinking `strong_only` 时持久化行为符合 RFC（链 [orchestration/turn-thinking](../orchestration/turn-thinking.md)）
- [ ] 未采纳回想时仍沿用普通 memory Prompt；事件中不复制记忆正文

---

## 6. 联调依赖

| 相关模块 | 数据关系 |
|----------|----------|
| `prompt` | 检索结果注入 `PromptInput` |
| `emotion` / `event` | 并行 pre 阶段，不共享表 |
| `chat-storage` | ① 聊天日志独立通道；回放合并进 LTM |
| Turn Thinking | Fast 档可能跳过部分 LTM 写入（HostProfile） |
