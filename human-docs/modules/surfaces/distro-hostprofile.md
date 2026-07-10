# 宿主面开工包 · 发行版 HostProfile

> **读者**：改 `distro.oclive.toml`、发行版能力、无头 HTTP 或三发行版内核生命周期的工程师。  
> **读完能做什么**：在 HostProfile 四层配置最上层改动，不越界改角色人设。  
> **耗时**：约 **50 min**  
> **SSOT 范围**：人类 checklist；字段 SSOT 见 [DISTRO_CAPABILITY_PROFILE](../../creator-docs/kernel/DISTRO_CAPABILITY_PROFILE.md)  
> **最后更新**：2026-06-26  
> **下一篇**：[paths/integrator](../../paths/integrator.md) · [orchestration/turn-thinking](../orchestration/turn-thinking.md)

---

## 1. 你插在哪

- **MODULE_MAP**：[§14 发行版层](../../handoff/MODULE_MAP_AND_HANDOFF.md#14-配置四层谁可改什么)  
- **文件**：`distros/*/distro.oclive.toml` → `HostProfile`  
- **能力门控**：`host_flags` · `[turn_thinking]` · `[plugin_backends]` 整表替换等  
- **无头**：`oclivenewnew-tauri --api` · `oclive-kernel-server`

---

## 2. 边界

| 能改 | 禁止 |
|------|------|
| 发行版 toml · HostProfile 字段 | 发行版任务改角色 `core_personality.txt`（G1 反向） |
| `skip_agent` · `event_impact_llm` 等 | 静默新增 handoff 顶层文档（G11） |
| 三发行版 spawn/attach 策略 | 把渗透写进 Chat Pro 内核默认链 |

---

## 3. 阅读清单

1. [DISTRO_CAPABILITY_PROFILE](../../creator-docs/kernel/DISTRO_CAPABILITY_PROFILE.md)  
2. [KERNEL_INTEGRATOR_LEARNING_PATH](../../creator-docs/getting-started/KERNEL_INTEGRATOR_LEARNING_PATH.md)  
3. [DISTRO_KERNEL_LIFECYCLE](../../creator-docs/kernel/DISTRO_KERNEL_LIFECYCLE.md)  
4. [THREE_DISTRO_KERNEL_CLOSURE](../../handoff/THREE_DISTRO_KERNEL_CLOSURE.md)  
5. [MODULE_MAP §3.2 有效 backends 解析链](../../handoff/MODULE_MAP_AND_HANDOFF.md#32-有效-backends-解析链)

---

## 4. 开发流程

- [ ] 改 toml → 对照 DISTRO_CAPABILITY_PROFILE 字段表  
- [ ] 若整表替换 `plugin_backends` → 理解合并链  
- [ ] 无头集成 → `OCLIVE_APP_DATA` · health check  
- [ ] `npm run test:distro-profile-mirror`（若动 profile）  
- [ ] `npm run check`

---

## 5. 验收

- [ ] 角色包人设未被发行版 PR 顺带修改  
- [ ] Turn Thinking / agent skip 行为可复现  
- [ ] 三发行版 ensure 报告仍绿（若涉 kernel ensure）

---

## 6. 联调依赖

| 相关模块 | 数据关系 |
|----------|----------|
| [turn-thinking](../orchestration/turn-thinking.md) | `[turn_thinking]` |
| [slots/agent](../slots/agent.md) | `skip_agent` |
| [slots/event](../slots/event.md) | `event_impact_llm` |
| [tauri-invoke](tauri-invoke.md) | 桌面宿主 IPC |
