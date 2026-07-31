# Phase 4+ 生态 — 文档笔记（剧场验证之后）

**状态：** 笔记 only · **不在 Phase 1–3 之前写代码**

## VS Code

- 维持 attach `:8420`；强化「开发者漏斗」文档即可。
- 入口：[`../oclive-vscode/AGENTS.md`](../../oclive-vscode/AGENTS.md) · [`creator-docs/role-pack/CROSS_HOST_MEMORY.md`](../creator-docs/role-pack/CROSS_HOST_MEMORY.md)
- Theater 角色包拷到 `OCLIVE_ROLES_DIR` 后 VS Code 可加载（标准 v2 蓝图）。

## 基础聊天（Chat Pro）

- **Release hero** = `desktop`（**OCLive Chat Pro**）· bundled profile [`distros/desktop-tauri/resources/distro-profiles/desktop.oclive.toml`](../distros/desktop-tauri/resources/distro-profiles/desktop.oclive.toml)
- **`desktop-chat`** = **dev lab only**（`examples/` + monorepo fallback）；**不当 hero**。
- Profile：[`examples/distro-profiles/desktop-chat.oclive.toml`](../examples/distro-profiles/desktop-chat.oclive.toml)
- 三发行版内核结项 smoke：[`THREE_DISTRO_KERNEL_CLOSURE.md`](./THREE_DISTRO_KERNEL_CLOSURE.md)

## OC 社交

- RFC / 笔记阶段 only。不在本阶段实现联机或社交图谱。

## AI 赌场

- **不投入**。无 backlog 任务。

## 第二剧场 / 多场景

- 进入 Phase 4+ backlog，需 **Chat Pro + VS Code Flash smoke 通过** 且 Theater v0 陌生人测试通过后再排期。
- 导演 RPC / 模式 2–3：[`theater/DEVELOPMENT_ROADMAP.md`](./theater/DEVELOPMENT_ROADMAP.md) §5（**Deferred**，不在 Pro/Flash 计划内实现）。
