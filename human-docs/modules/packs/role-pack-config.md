# 角色包开工包 · `config.json` 与校验

> **读者**：为角色包增加或修改 `config.json` 字段、turn_thinking 路由的工程师。  
> **读完能做什么**：走通 loader → validation → runtime 使用链，不破坏 G1。  
> **耗时**：约 **50 min**  
> **SSOT 范围**：人类 checklist；字段 SSOT 见 [ROLE_PACK_SPEC](../../creator-docs/role-pack/ROLE_PACK_SPEC.md)  
> **最后更新**：2026-06-26  
> **下一篇**：[orchestration/turn-thinking](../orchestration/turn-thinking.md) · [07 §3](../../07_COMMON_TASKS.md#3-加-configjson-字段)

---

## 1. 你插在哪

- **文件**：`distros/chat-pro/roles/{id}/config.json`  
- **解析**：`RoleStorage::load_role` 及相关 loader  
- **校验**：`kernel/crates/oclive_validation`  
- **运行时**：`turn_thinking.rs` · 各 `*_engine`（**非** API 层）  
- **MODULE_MAP**：[§12 Turn Thinking 包级路由](../../handoff/MODULE_MAP_AND_HANDOFF.md#12-编排行策略非模块号--易与六槽混淆)

---

## 2. 边界

| 能改 | 禁止 |
|------|------|
| 角色包 `config.json` 已文档化字段 | 角色任务改蓝图 `slot_registry`（G1） |
| `oclive_validation` 中角色包 schema | 在 Tauri `api/*.rs` 堆解析逻辑 |
| `turn_thinking` OR/AND · latch · ephemeral（RFC §8–12） | 把 Turn Thinking 登记为第七槽 |

---

## 3. 阅读清单

1. [ROLE_PACK_SPEC §9](../../creator-docs/role-pack/ROLE_PACK_SPEC.md) — `config.json`  
2. [RFC_TURN_THINKING_PERSISTENCE §8–12](../../creator-docs/rfc/RFC_TURN_THINKING_PERSISTENCE.md)  
3. [07 常见任务 §3](../../07_COMMON_TASKS.md#3-加-configjson-字段)  
4. [MODULE_MAP §12](../../handoff/MODULE_MAP_AND_HANDOFF.md#12-编排行策略非模块号--易与六槽混淆)  
5. 迁移：`035_turn_thinking_runtime.sql`（若涉持久化字段）

---

## 4. 开发流程

- [ ] 在 ROLE_PACK_SPEC 确认字段是否已定义；新字段须先扩 spec + validation  
- [ ] 改 loader / validation crate  
- [ ] 在使用点（`turn_thinking.rs` 或对应 engine）读取  
- [ ] 补 domain 或 validation 单测  
- [ ] `npm run check:rust`

---

## 5. 验收

- [ ] 非法 `config.json` 在校验层失败，错误码符合 [KERNEL_ERROR_CODE_CONVENTION](../../creator-docs/getting-started/KERNEL_ERROR_CODE_CONVENTION.md)  
- [ ] HostProfile 与包级 `turn_thinking` 合并行为符合 RFC  
- [ ] 未改 `slot_registry` 结构

---

## 6. 联调依赖

| 相关模块 | 数据关系 |
|----------|----------|
| [turn-thinking](../orchestration/turn-thinking.md) | Fast/Deep/Auto 与持久化分流 |
| `event` | 强事件仍写 LTM / 好感（与 Fast 档策略交叉） |
| [role-pack-content](role-pack-content.md) | 人设文件与 config 分工 |
