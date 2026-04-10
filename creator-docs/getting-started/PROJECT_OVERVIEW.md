# 项目全貌与事项总览（理清条理）

本文把 **三件套分工、已落实内容、文档入口、命令、人机分工、待排期** 收拢在一处；细节仍以各专题文档为准。

---

## 1. 三个仓库分别是什么

| 仓库 | 角色 | 技术栈（概要） |
|------|------|----------------|
| **oclivenewnew**（本仓库） | **运行时**：玩家对话、角色包加载、引擎、Tauri 桌面端 | Rust + Vue + Tauri |
| **oclive-pack-editor**（另仓，同级目录常见） | **编写器**：编辑/导出 `roles/{id}/` 或 zip | Vue + Tauri（与运行时**不同** `package.json`） |
| **oclive-launcher**（另仓） | **启动器**：配置路径、拉起运行时与编写器；**环境与排障**（依赖检测、配置重置） | Vue + Tauri |

**唯一纽带**：磁盘上的 **角色包**（结构与 `roles/{角色id}/` 一致）。运行时与编写器通过 **导入/导出包** 或 **OCLIVE_ROLES_DIR** 对接，不依赖进程间复杂 IPC。

---

## 2. 本仓库（oclivenewnew）已落实的能力（摘要）

- **角色包导入**：`.ocpak`、`.zip`、已解压目录；预览与冲突处理；导入进度；ZIP 内 `manifest.json` 优先级（根目录优先）。
- **工程**：`npm run check`（日常）、`npm run check:release`（发版前全量测试）；Rust fmt / clippy / `cargo test`。
- **CI**：GitHub Actions 在 **Ubuntu + Windows** 上跑 Rust 与 `npm run build`（见 `.github/workflows/ci.yml`）。
- **文档**：`creator-docs/`、`roles/README_MANIFEST.md`、导入测试清单 `roles/TESTING_ROLE_PACK_IMPORT.md`、愿景与体验 backlog 等。

---

## 3. 文档地图（从哪里读起）

| 需求 | 文档 |
|------|------|
| **总索引** | [DOCUMENTATION_INDEX.md](DOCUMENTATION_INDEX.md) |
| **创作者：从包到 oclive** | [CREATOR_WORKFLOW.md](CREATOR_WORKFLOW.md) |
| **manifest / 导入** | [roles/README_MANIFEST.md](../../roles/README_MANIFEST.md) |
| **性格档案设计轴心** | [docs/personality-archive-notes.md](../../docs/personality-archive-notes.md) |
| **思路变化（以前 vs 现在）** | [docs/design-axis-evolution.md](../../docs/design-axis-evolution.md) |
| **导入手工测试清单** | [roles/TESTING_ROLE_PACK_IMPORT.md](../../roles/TESTING_ROLE_PACK_IMPORT.md) |
| **按月愿景** | [../roadmap/VISION_ROADMAP_MONTHLY.md](../roadmap/VISION_ROADMAP_MONTHLY.md) |
| **体验向 backlog（试聊 / 启动器 / 市场）** | [../roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md](../roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md) |
| **构建、CI、发版门槛、Sentry** | 根目录 [README.md](../../README.md) |
| **贡献与本地检查** | [CONTRIBUTING.md](../../CONTRIBUTING.md) |

---

## 4. 常用命令（本仓库根目录）

| 命令 | 用途 |
|------|------|
| `npm run dev` / `npm run tauri:dev` | 本地开发 |
| `npm run check` | 日常提交前：`vite build` + `cargo fmt` / `clippy` / **`cargo test --lib`** |
| `npm run check:release` | **发版或改引擎前**：含 **全量 `cargo test`**（与 CI 中 Rust 任务对齐） |
| `npm run check:rust:test:all` | 只跑全量 Rust 测试 |

---

## 5. 人机分工（谁做什么）

### 需要你本地完成的

- **Git**：`clone` / `pull`、**push** 到 GitHub（或你的远端），在 **Actions** 里确认 workflow **绿灯**。
- **LNK1104**（Windows 链接失败）：关占用进程、`cargo test -j 1` 等；属环境/占用问题，**无法靠 CI 在每台开发机上强制避免**。
- **发版决策**：版本号、`CHANGELOG`、是否配置 `VITE_SENTRY_DSN`、安装包签名、对外说明。
- **冒烟与联调**：装包 → 启动 → 对话 → 导入包；编写器导出 → oclive 导入（按测试清单）。

### 适合在协作里交给开发/AI 的

- 功能与重构、文档与 CI 脚本修改、按 backlog 拆任务、根据报错改代码与补测试。

---

## 6. 与「愿景」相比：未排期或进行中的事

不表示「没做好」，而是 **产品阶段与排期由你决定**：

- 编写器内 **快速试聊**、启动器 **一键 Ollama/模型**、**角色/插件市场** 等 → [BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md](../roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md)。
- 契约深化、`min_runtime`、编写器与 **`load_role` 校验** 完全对齐等 → [VISION_ROADMAP_MONTHLY.md](../roadmap/VISION_ROADMAP_MONTHLY.md) 与各 `EDITOR_*`、`PACK_VERSIONING` 文档。

---

## 7. 另两个仓库的 CI（别忘了）

若已检出 **oclive-pack-editor**、**oclive-launcher**，各自根目录有 `.github/workflows/ci.yml`（双平台 + 编写器在 Linux 上另跑 Vitest/E2E）。**推送到远端后**在对应仓库 **Actions** 查看结果。

---

## 8. 发版前可勾选的极简清单

1. 本机：`npm run check:release`（或至少 `check` + 你接受的测试范围）。  
2. 三仓 push 后 **CI 通过**。  
3. 版本号与 `CHANGELOG` 已更新。  
4. 按 `TESTING_ROLE_PACK_IMPORT.md` 或等价冒烟过一遍。  
5. 分发方式明确：当前 **未配置 Tauri 在线更新** 时，以 **离线安装包** 为准（见根 `README`）。

---

*若本文与专题文档冲突，以专题文档与仓库代码为准；重大变更请同步更新本节或 `CHANGELOG.md`。*
