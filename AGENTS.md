# Agent / AI 协作说明（A.I.Live · oclivenewnew）

本仓库为 **A.I.Live / OCLive** —— 开源、可组装、隐私优先的 **AI 角色运行时与开发者平台**（**Tauri + Vue 3 + Rust**）。默认角色包（如 `distros/chat-pro/roles/mumu`）为**官方示例**，非产品上限。

**人类开发者（不用 Cursor）**：请先 **[human-docs/README.md](human-docs/README.md)**（L0–L2 约 1 小时 · 排版面向人类认知）；**不要**从本文起步。

**AI 深读分类目录**（架构 · 契约 · 代码锚点 · 场景路径）：**[handoff/AI_READING_INDEX.md](handoff/AI_READING_INDEX.md)**  
**文档五层分工**：**[handoff/README.md](handoff/README.md) §文档分层**  
**GitHub 首页 [`README.md`](README.md) 仅面向人类**；细节 **链 SSOT，禁止复制长表**（G14）。

---

## 改代码前必读（AI · 精简索引）

| 优先级 | 文档 | 用途 |
|--------|------|------|
| 1 | [handoff/AI_CHANGE_BOUNDARIES.md](handoff/AI_CHANGE_BOUNDARIES.md) | **G1–G17** · 代码编写纪律 · 关联改动闭环 · **文档编写纪律** |
| 2 | [handoff/MODULE_MAP_AND_HANDOFF.md](handoff/MODULE_MAP_AND_HANDOFF.md) | 模块定义 · 六槽/设施/独立通道 · **逐槽关系** |
| 3 | [creator-docs/NAMING_CONVENTIONS.md](creator-docs/NAMING_CONVENTIONS.md) §4.2 | canonical import |
| 4 | [handoff/BUS_FACTOR_NOTES.md](handoff/BUS_FACTOR_NOTES.md) | `process_message` · DB · 错误码 **文件锚点** |
| 5 | [handoff/AI_VERIFICATION_PROTOCOL.md](handoff/AI_VERIFICATION_PROTOCOL.md) | 带数字的审查 / 汇报 **须核实** |
| 6 | [handoff/README.md](handoff/README.md) §文档分责 | 动文档前查 SSOT · **禁止冗余新建** |
| 7 | [`.cursor/rules/oclivenewnew.mdc`](.cursor/rules/oclivenewnew.mdc) | 10 条硬约束镜像 |
| — | [`.cursor/skills/oclive-dev-pipeline/SKILL.md`](.cursor/skills/oclive-dev-pipeline/SKILL.md) | **七阶段开发流水线（OCLive 定制层）**；通用框架 `~/.cursor/skills/dev-pipeline/` |
| — | [`.cursor/skills/oclive-debt-marathon/SKILL.md`](.cursor/skills/oclive-debt-marathon/SKILL.md) · [`handoff/debt-marathon/`](handoff/debt-marathon/README.md) | **债偿还马拉松**：长流程计划书 · 分阶段子 Agent · 波次日志 |

**文档纪律摘要（G10–G16）**：模块关系 **只**改 MODULE_MAP；无 RFC/关键决策 **不新建**顶层 `.md`；**先读**关联 SSOT 再写（可以慢）；**链接代替复制**；人类长文在 `human-docs/` / `creator-docs/`，本文 **不**堆架构长节。

**关联改动摘要（G17）**：按能力核对生产者 → 契约 → 适配/权限 → 消费者 → 状态/回退 → 测试；最小改动面不等于只改单端。涉及 Chat Pro / 目录插件 / 插槽时跑 `npm run check:module-compat`。

---

## 发版版本（`main`，2026-07-10）

| 产物 | 版本 | 位置 |
|------|------|------|
| **桌面宿主** | **0.5.0** | 根 `package.json`、`distros/desktop-tauri/Cargo.toml` |
| **角色包编写器** | **0.5.0** | 姊妹仓 `oclive-pack-editor` |
| **VS Code 扩展** | **0.4.1** | 姊妹仓 `oclive-vscode` |
| **`oclive-cli`** | **0.1.0** | `kernel/crates/oclive-cli/Cargo.toml` |
| **`oclive_kernel_runtime`** | **0.2.0** | `kernel/crates/oclive_kernel_runtime/Cargo.toml` |

变更见 [CHANGELOG.md](CHANGELOG.md) **`[0.5.0]`** · SemVer [RELEASE_VERSIONING.md](creator-docs/development/RELEASE_VERSIONING.md)。

---

## 架构与契约（详情链出 · 勿在本文双写）

| 主题 | SSOT |
|------|------|
| 模块注册表 · 记忆三套存储 · 六槽解耦 | [handoff/MODULE_MAP_AND_HANDOFF.md](handoff/MODULE_MAP_AND_HANDOFF.md) |
| 对外架构叙述 | [OCLIVE_ARCHITECTURE_OVERVIEW.md](creator-docs/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md) |
| 六槽 DTO · 编排顺序 | [PLUGIN_V1.md](creator-docs/plugin-and-architecture/PLUGIN_V1.md) |
| 角色包 vs 蓝图 | [handoff/ROLE_PACK_BOUNDARY.md](handoff/ROLE_PACK_BOUNDARY.md) |
| 聊天 vs 记忆 | [handoff/CHAT_STORAGE_ARCHITECTURE.md](handoff/CHAT_STORAGE_ARCHITECTURE.md) |
| 发行版 HostProfile | [DISTRO_CAPABILITY_PROFILE.md](creator-docs/kernel/DISTRO_CAPABILITY_PROFILE.md) |
| 活跃债 / 冻结 | [handoff/TECHNICAL_DEBT_INVENTORY.md](handoff/TECHNICAL_DEBT_INVENTORY.md) |
| 文档总索引 | [DOCUMENTATION_INDEX.md](creator-docs/getting-started/DOCUMENTATION_INDEX.md) |
| 人类接手阶梯 L0–L8 | [human-docs/README.md](human-docs/README.md) |

**主编排**：`kernel/crates/oclive_kernel_host/.../process_message.rs` → `co_present` · 蓝图 **`steps[]` 不调度** · Tauri 命令只在 `distros/desktop-tauri/src/api/*.rs` · 回复字段 **`reply`**。

---

## 测试（AI 须用 SSOT 条数）

- 审查汇报前：[AI_VERIFICATION_PROTOCOL.md](handoff/AI_VERIFICATION_PROTOCOL.md)
- OOCP S0–S12（+ 可选 S13/S14）：[OOCP_TEST_SUITE.md](creator-docs/testing/OOCP_TEST_SUITE.md)
- invoke 热路径 **13** 条：[INVOKE_HOTPATH_MATRIX.md](handoff/INVOKE_HOTPATH_MATRIX.md)
- Dimension 5 门禁：`node scripts/dimension5-acceptance.mjs --ci`；检查项总数以脚本结尾 `PASS (N checks)` 为准（`--ci` 跳过 sample lib tests，但仍计入结果）。
- 日常 `npm run check:rust` **不含 doctest**；发版 `npm run check:release` **含**

---

## 仓库布局（速记）

- **内核**：`kernel/crates/`（`oclive_kernel_host` = 编排 + DB）
- **桌面**：`distros/desktop-tauri` · **前端**：`distros/chat-pro` + `distros/shared`
- **角色包 SSOT**：`distros/chat-pro/roles/`
- **Cargo target**：仓库外 `../oclive-dev-artifacts/oclivenewnew-cargo-target/`（见 `.cargo/config.toml`）

---

## 仅本文保留的易漏点

- **中文写入红线**：禁管道/`-Encoding Ascii` 传中文；写中文文件只用 apply_patch 或 .NET UTF-8 无 BOM；写后自查汉字数>0、无 `\?{3,}`、无 BOM（SSOT：[handoff/debt-marathon/AI_AND_PIPELINE_GATES.md](handoff/debt-marathon/AI_AND_PIPELINE_GATES.md) §7）
- **Tauri invoke**：Rust `snake_case` → 前端 **camelCase**（`distros/shared/src/api/`）
- **权限**：directory 插件 / MCP / remote env 须用户授权（`network:*` · `process:spawn`）
- **Remotion 演示**：独立仓 `oclive-remotion-demo`，勿在主仓根跑 `npm run preview`
- **姊妹仓**：`oclive-pack-editor` · `oclive-launcher` · `oclive-plugin-market` · `oclive-vscode`（各仓 `AGENTS.md` 指回本索引）

**禁止当 truth**：`handoff/archive/*`（含已归档的 `04_4.6_PROJECT_TRUTH_CHECKLIST.md`，G3）
