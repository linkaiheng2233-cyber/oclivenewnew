# 独立通道开工包 · 聊天存储

> **读者**：改聊天日志、会话列表、导出，或澄清「聊天 vs 记忆」的工程师。  
> **读完能做什么**：在不动 memory 槽真源的前提下改 HybridConversationStore 边界。  
> **耗时**：约 **45 min**  
> **SSOT 范围**：人类 checklist；架构见 [CHAT_STORAGE_ARCHITECTURE](../../handoff/CHAT_STORAGE_ARCHITECTURE.md)  
> **最后更新**：2026-06-26  
> **下一篇**：[slots/memory](../slots/memory.md) · [MODULE_MAP §11](../../handoff/MODULE_MAP_AND_HANDOFF.md#11-独立通道能力增强注册表--非六槽)

---

## 1. 你插在哪

- **归类**：独立通道（**非**六槽 `memory`）  
- **组件**：`HybridConversationStore` · `chat_sessions` / `chat_messages`  
- **MODULE_MAP**：[§4 与聊天存储无关](../../handoff/MODULE_MAP_AND_HANDOFF.md#4-第-1-模块--memory)（memory 模块显式声明）  
- **深读 SSOT**：[CHAT_STORAGE_ARCHITECTURE](../../handoff/CHAT_STORAGE_ARCHITECTURE.md)

---

## 2. 边界

| 能改 | 禁止 |
|------|------|
| 聊天 UI 回放、导出、会话管理 API | 把 `chat_messages` 当 memory 检索真源 |
| `HybridConversationStore` 实现与迁移 | 删聊天时顺带清空 `short_term_memory` / `long_term_memory` |
| 记忆回放 `replay_memory_extraction`（①→③ 合并） | 在 memory 槽直接读 `{app_data}/chats/` 目录 |

---

## 3. 阅读清单

1. [CHAT_STORAGE_ARCHITECTURE](../../handoff/CHAT_STORAGE_ARCHITECTURE.md)  
2. [01 简架构 §记忆三套](../../01_ARCHITECTURE_SIMPLE.md#记忆三套存储最易混--必背)  
3. [MODULE_MAP §4](../../handoff/MODULE_MAP_AND_HANDOFF.md#4-第-1-模块--memory)  
4. [BUS_FACTOR](../../handoff/BUS_FACTOR_NOTES.md) — 持久化锚点  
5. 迁移：`001_init.sql` — `chat_*` 表

---

## 4. 开发流程

- [ ] 画清改动影响 ① 聊天日志还是 ②③ 记忆表  
- [ ] 若只动聊天 → repository / `HybridConversationStore`  
- [ ] 若动回放 → 确认不覆盖 LTM 全文  
- [ ] 同步 DTO（`reply` 等）若涉前端  
- [ ] `npm run check:rust`

---

## 5. 验收

- [ ] 删 UI 聊天记录后 STM/LTM 行为符合产品预期  
- [ ] memory 槽检索不直接拼 chat 行  
- [ ] 文档链 CHAT_STORAGE_ARCHITECTURE，未复制三套存储大表进 PR

---

## 6. 联调依赖

| 相关模块 | 数据关系 |
|----------|----------|
| [memory](../slots/memory.md) | ②③ 独立表；pre/post 写入 |
| `reply_post_process` | post 阶段润色 **reply**，不写 chat 结构 |
| Turn Thinking | Fast 档仍写聊天 turns；可能跳过部分 LTM |
