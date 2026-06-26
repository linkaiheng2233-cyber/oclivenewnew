# 独立通道开工包 · 用户身份

> **读者**：改 `user_identities/`、pre 段落用户身份注入的工程师。  
> **读完能做什么**：在独立通道 `user_identity` 边界内改动，不进六槽。  
> **耗时**：约 **35 min**  
> **SSOT 范围**：人类 checklist；见 [MODULE_MAP §11](../../handoff/MODULE_MAP_AND_HANDOFF.md#11-独立通道能力增强注册表--非六槽)  
> **最后更新**：2026-06-26  
> **下一篇**：[reply-post-process](reply-post-process.md) · [RFC_SIDE_CHANNEL](../../creator-docs/rfc/RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md)

---

## 1. 你插在哪

- **MODULE_MAP**：[§11 `user_identity`](../../handoff/MODULE_MAP_AND_HANDOFF.md#11-独立通道能力增强注册表--非六槽)  
- **注册表 `id`**：`user_identity`（**非**六槽键）  
- **锚点**：`user_identities/` · `turn_pipeline/pre.rs`  
- **进 `process_message`？**：**是**（pre 段落）

---

## 2. 边界

| 能改 | 禁止 |
|------|------|
| 身份文件格式 · pre 注入逻辑 | 写入 `plugin_backends` 冒充六槽 |
| [RFC_SIDE_CHANNEL](../../creator-docs/rfc/RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md) 范围 | 与 MCP user 混淆（属 agent 授权域） |

Phase2 已交付 — 见 [USER_IDENTITY_REPLY_POST_PROCESSOR_PHASE2](../../handoff/USER_IDENTITY_REPLY_POST_PROCESSOR_PHASE2.md)（史料，勿当待办）。

---

## 3. 阅读清单

1. [MODULE_MAP §11](../../handoff/MODULE_MAP_AND_HANDOFF.md#11-独立通道能力增强注册表--非六槽)  
2. [RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS](../../creator-docs/rfc/RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md)  
3. [ROLE_PACK_SPEC](../../creator-docs/role-pack/ROLE_PACK_SPEC.md) — 身份相关字段  
4. `turn_pipeline/pre.rs`  
5. [CROSS_HOST_MEMORY](../../creator-docs/role-pack/CROSS_HOST_MEMORY.md)（跨宿主）

---

## 4. 开发流程

- [ ] 改存储 → `user_identities/` 约定 + loader  
- [ ] 改注入 → pre 段落 · `PromptInput`  
- [ ] 单测 pre roundtrip  
- [ ] `npm run check:rust`

---

## 5. 验收

- [ ] 身份进 pre，不进 post_llm 润色链混淆  
- [ ] 注册表 id 仍为 `user_identity`  
- [ ] 未扩成六槽

---

## 6. 联调依赖

| 相关模块 | 数据关系 |
|----------|----------|
| [prompt](../slots/prompt.md) | 身份段落进组装 |
| [reply-post-process](reply-post-process.md) | 独立 post 通道 |
| [chat-storage](chat-storage.md) | 聊天日志不含身份真源 |
