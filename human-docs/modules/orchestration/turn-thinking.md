# 编排行开工包 · Turn Thinking

> **读者**：改 Fast/Deep/Auto、持久化分流或包级 `turn_thinking` 路由的工程师。  
> **读完能做什么**：把 Turn Thinking 当 **编排行策略** 改动，**不**登记为第七槽。  
> **耗时**：约 **50 min**  
> **SSOT 范围**：人类 checklist；RFC 见 [RFC_TURN_THINKING_PERSISTENCE](../../creator-docs/rfc/RFC_TURN_THINKING_PERSISTENCE.md)  
> **最后更新**：2026-06-26  
> **下一篇**：[packs/role-pack-config](../packs/role-pack-config.md) · [MODULE_MAP §12](../../handoff/MODULE_MAP_AND_HANDOFF.md#12-编排行策略非模块号--易与六槽混淆)

---

## 1. 你插在哪

- **MODULE_MAP**：[§12 Turn Thinking](../../handoff/MODULE_MAP_AND_HANDOFF.md#12-编排行策略非模块号--易与六槽混淆)  
- **非六槽** · **非** `plugin_backends` 键  
- **代码**：`turn_thinking.rs` · `co_present` / `TurnThinkingRouter`  
- **配置**：发行版 `[turn_thinking]` · 角色包 `config.json` → `turn_thinking`（RFC §8–12）

---

## 2. 边界

| 能改 | 禁止 |
|------|------|
| `fast_persistence` · `strong_only` 等 HostProfile 字段 | 新增 `turn_thinking` 六槽键 |
| 包级 OR/AND · latch · `ephemeral_archive` | Fast 轮压缩用户原句 |
| `035_turn_thinking_runtime.sql` 相关字段 | 玩家侧 Fast/Deep 开关（产品纪律） |

**纪律**：聊天 turns **每轮仍写** UI 日志；Fast **不压缩**用户原句。

---

## 3. 阅读清单

1. [RFC_TURN_THINKING_PERSISTENCE](../../creator-docs/rfc/RFC_TURN_THINKING_PERSISTENCE.md)（含 §8–12）  
2. [MODULE_MAP §12](../../handoff/MODULE_MAP_AND_HANDOFF.md#12-编排行策略非模块号--易与六槽混淆)  
3. [01 简架构 §Turn Thinking](../../01_ARCHITECTURE_SIMPLE.md#turn-thinkingfast--deep--编排行)  
4. [DISTRO_CAPABILITY_PROFILE §3.2.1](../../creator-docs/kernel/DISTRO_CAPABILITY_PROFILE.md)  
5. [role-pack-config](../packs/role-pack-config.md)

---

## 4. 开发流程

- [ ] 区分发行版 HostProfile vs 角色包 config  
- [ ] 改路由 → `turn_thinking.rs`；改持久化 → `co_present` / `post`  
- [ ] 动 SQL → 新迁移 + repository  
- [ ] 单测覆盖 Fast/Deep/Auto 与 strong 事件例外  
- [ ] `npm run check:rust`

---

## 5. 验收

- [ ] 蓝图 `steps[]` 仍不参与首轮调度  
- [ ] Quarrel 等强事件在 `strong_only` 下仍写 LTM/好感  
- [ ] RFC §8–12 行为与实现一致

---

## 6. 联调依赖

| 相关模块 | 数据关系 |
|----------|----------|
| [memory](../slots/memory.md) | Fast 档 LTM 写入策略 |
| [event](../slots/event.md) | Fast 不调 LLM event 路径 |
| [model-tier](model-tier.md) | Small/Large 与 Deep capsule |
