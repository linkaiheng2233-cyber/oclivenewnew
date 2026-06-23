# 项目现状（事实快照）

**用途**：给协作者与发布流程一个**短、可核对**的当前状态摘要（版本、交付面、内核与产品门槛、变更日志入口）。**不替代** [KERNEL_IMPLEMENTATION_PLAN.md](KERNEL_IMPLEMENTATION_PLAN.md) 的验收细节或 [PROJECT_STATUS_AND_ALIGNMENT.md](PROJECT_STATUS_AND_ALIGNMENT.md) 的文档地图。

[English](../../creator-docs-en/getting-started/PROJECT_CURRENT_STATUS.md)

**快照日期**：2026-05-15（随大里程碑或版本号 bump 时应更新本页首段与日期）

---

## 应用与仓库版本

| 项 | 值 |
|----|-----|
| 桌面应用语义化版本 | **0.2.0**（以根目录 `package.json`、`distros/desktop-tauri/tauri.conf.json`、`distros/desktop-tauri/Cargo.toml` 对齐为准） |
| 默认 HTTP API（`--api`） | `http://127.0.0.1:8420`（健康检查 `GET /health`） |
| 用户可见变更流水 | **[CHANGELOG.md](../../CHANGELOG.md)**（中文）· **[CHANGELOG.en.md](../../CHANGELOG.en.md)**（英文，与中文同步维护条目） |

---

## 交付面（本仓库 `oclivenewnew`）

- **运行时**：Tauri 桌面端；角色包导入（`.ocpak` / `.zip` / 目录）、对话主路径 **`process_message`**、**第 1–6 模块** `plugin_backends`、**第 1–2 设施子模块**（复杂情感、专家模型/专家路由）、后端模块插件模块（目录/Remote）、本地 HTTP `--api`、启动健康检查等（架构见 [OCLIVE_ARCHITECTURE_OVERVIEW.md](OCLIVE_ARCHITECTURE_OVERVIEW.md)；细节见 [PROJECT_OVERVIEW.md](PROJECT_OVERVIEW.md)）。
- **内核工程**：里程碑 **K0–K5** 在计划中除 **P2（OTA / 远程日志等）** 外已收口；验收留痕与 CI 见 [KERNEL_IMPLEMENTATION_PLAN.md](KERNEL_IMPLEMENTATION_PLAN.md) 与根目录 [AGENTS.md](../../AGENTS.md)。
- **产品级「首发」硬门槛**：仍以 [handoff/PRODUCT_AND_KERNEL_GAP_CHECKLIST.md](../../handoff/PRODUCT_AND_KERNEL_GAP_CHECKLIST.md) **§A** 为准，与内核里程碑解耦排期（见该文 **§D**）。

---

## 姊妹仓与国际化

- **编写器** `oclive-pack-editor`、**启动器** `oclive-launcher`、**插件市场站** `oclive-plugin-market`：与本仓通过角色包与文档索引协作；详见 [PROJECT_OVERVIEW.md](PROJECT_OVERVIEW.md)。
- **四仓 UI 文案双语基线**：见 [handoff/I18N_FOUR_REPO_BASELINE.md](../../handoff/I18N_FOUR_REPO_BASELINE.md)。
- **创作者英文文档（`creator-docs-en/`）**：收尾范围与更新约定见 [creator-docs-en/README.md](../../creator-docs-en/README.md#documentation-bilingual-closure-baseline)（与中文 `creator-docs/` 对拍；路线图等长尾仍为中文）。

## 路线图与对齐习惯

| 需求 | 文档 |
|------|------|
| 按月愿景与阶段 | [../roadmap/VISION_ROADMAP_MONTHLY.md](../roadmap/VISION_ROADMAP_MONTHLY.md) |
| 体验向 backlog（试聊、市场等） | [../roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md](../roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md) |
| 一页：进度 + 目标 + 按用途分类的文档地图 | [PROJECT_STATUS_AND_ALIGNMENT.md](PROJECT_STATUS_AND_ALIGNMENT.md) |

发版或改契约前：根 [README.md](../../README.md) 中的 `npm run check` / `check:release` 与 CHANGELOG 双语更新。
