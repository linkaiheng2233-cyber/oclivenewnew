# 独立通道开工包 · 回复后处理

> **读者**：改 `reply_post_process`、回复润色/改写链路的工程师。  
> **读完能做什么**：在 post 独立通道内改润色，保持 DTO 字段 **`reply`**。  
> **耗时**：约 **40 min**  
> **SSOT 范围**：人类 checklist；见 [MODULE_MAP §11](../../handoff/MODULE_MAP_AND_HANDOFF.md#11-独立通道能力增强注册表--非六槽)  
> **最后更新**：2026-06-26  
> **下一篇**：[user-identity](user-identity.md) · [ROLE_PACK_SPEC](../../creator-docs/role-pack/ROLE_PACK_SPEC.md)

---

## 1. 你插在哪

- **MODULE_MAP**：[§11 `reply_post_process`](../../handoff/MODULE_MAP_AND_HANDOFF.md#11-独立通道能力增强注册表--非六槽)  
- **配置**：角色包 `config.json`  
- **锚点**：`turn_pipeline/post.rs` · post_llm 之后  
- **进 `process_message`？**：**是**（post）

---

## 2. 边界

| 能改 | 禁止 |
|------|------|
| 润色规则 · remote 后处理器 | 把响应用 `response` 字段（须 **`reply`**） |
| `config.json` 已文档化开关 | 在 Vue 层二次改写绕过内核 |
| Phase2 已交付行为扩展 | 复制 [REPLY_POST_PROCESSOR_DESIGN_REPORT](../../handoff/REPLY_POST_PROCESSOR_DESIGN_REPORT.md) 全文进 PR |

设计史料：[USER_IDENTITY_REPLY_POST_PROCESSOR_PHASE2](../../handoff/USER_IDENTITY_REPLY_POST_PROCESSOR_PHASE2.md) · [REPLY_POST_PROCESS_POLISH_SCOPE](../../handoff/REPLY_POST_PROCESS_POLISH_SCOPE.md)。

---

## 3. 阅读清单

1. [MODULE_MAP §11](../../handoff/MODULE_MAP_AND_HANDOFF.md#11-独立通道能力增强注册表--非六槽)  
2. [RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS](../../creator-docs/rfc/RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md)  
3. [04 工程约束 §4](../../04_ENGINEERING_RULES.md) — `reply` 字段  
4. [ROLE_PACK_SPEC](../../creator-docs/role-pack/ROLE_PACK_SPEC.md)  
5. `turn_pipeline/post.rs`

---

## 4. 开发流程

- [ ] 改处理器 → post 链 · 保持 `reply` 契约  
- [ ] 角色包开关 → `config.json` + validation  
- [ ] 单测 post roundtrip  
- [ ] `npm run check:rust`

---

## 5. 验收

- [ ] 前端仍读 **`reply`**  
- [ ] agent 短路输出也可被后处理（若配置启用）  
- [ ] 未登记为六槽

---

## 6. 联调依赖

| 相关模块 | 数据关系 |
|----------|----------|
| [slots/llm](../slots/llm.md) | 上游原始生成文本 |
| [slots/agent](../slots/agent.md) | 短路文本也可进 post |
| [role-pack-config](../packs/role-pack-config.md) | config 开关 |
