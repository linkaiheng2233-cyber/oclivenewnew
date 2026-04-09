# oclive（oclivenewnew）

[![CI](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/workflows/ci.yml/badge.svg)](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/workflows/ci.yml)

本地优先的桌面角色对话应用：**Tauri + Vue 3 + Rust**。引擎支持场景、虚拟时间、异地/共景、好感与记忆、可替换子系统（记忆检索 / 情绪 / 事件估计 / Prompt 组装），角色内容以 **`roles/{角色id}/`** 角色包分发。

## 文档（创作者与扩展）

**入口**：[creator-docs/README.md](creator-docs/README.md)（含目录说明与阅读顺序）

| 说明 | 路径 |
|------|------|
| 文档总索引 | [creator-docs/getting-started/DOCUMENTATION_INDEX.md](creator-docs/getting-started/DOCUMENTATION_INDEX.md) |
| **项目全貌（三件套、事项、命令）** | [creator-docs/getting-started/PROJECT_OVERVIEW.md](creator-docs/getting-started/PROJECT_OVERVIEW.md) |
| **GitHub：CI / Dependabot / 网页设置清单** | [creator-docs/getting-started/GITHUB_REPO_CHECKLIST.md](creator-docs/getting-started/GITHUB_REPO_CHECKLIST.md) |
| 愿景与按月路线 | [creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md](creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md) |
| 体验差异化 backlog（与愿景对照） | [creator-docs/roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md](creator-docs/roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md) |
| 后日待办（工具链 / CI · 性价比备忘） | [creator-docs/roadmap/SOMEDAY_TOOLCHAIN_CI.md](creator-docs/roadmap/SOMEDAY_TOOLCHAIN_CI.md) |
| 市场 · 启动器联动（发版同发、分阶段） | [creator-docs/roadmap/MARKET_LAUNCHER_INTEGRATION.md](creator-docs/roadmap/MARKET_LAUNCHER_INTEGRATION.md) |
| 社区站愿景（网页 · 论坛 / 角色包 / 插件） | [creator-docs/roadmap/COMMUNITY_WEB_VISION.md](creator-docs/roadmap/COMMUNITY_WEB_VISION.md) |
| 插件区（网站）信息架构 | [creator-docs/roadmap/PLUGIN_WEB_SECTION.md](creator-docs/roadmap/PLUGIN_WEB_SECTION.md) |
| 插件契约 v1 | [creator-docs/plugin-and-architecture/PLUGIN_V1.md](creator-docs/plugin-and-architecture/PLUGIN_V1.md) |
| 包版本与兼容性 | [creator-docs/role-pack/PACK_VERSIONING.md](creator-docs/role-pack/PACK_VERSIONING.md) |
| 创作者工作流 | [creator-docs/getting-started/CREATOR_WORKFLOW.md](creator-docs/getting-started/CREATOR_WORKFLOW.md)（含 **`OCLIVE_ROLES_DIR`**、编写器分工） |
| 扩展点索引 | [creator-docs/plugin-and-architecture/EXTENSION_POINTS.md](creator-docs/plugin-and-architecture/EXTENSION_POINTS.md) |
| 如何替换模块 | [creator-docs/plugin-and-architecture/HOW_TO_REPLACE_MODULES.md](creator-docs/plugin-and-architecture/HOW_TO_REPLACE_MODULES.md) |
| 创作者架构（本地 / HTTP 侧车 / 更新策略） | [creator-docs/plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md](creator-docs/plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md) |
| HTTP JSON-RPC 协议 | [creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md](creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md) |
| **本机侧车 + 闭源 API（BYOK，用户向）** | [creator-docs/getting-started/SIDECAR_LLM_USER_GUIDE.md](creator-docs/getting-started/SIDECAR_LLM_USER_GUIDE.md) |
| 最小侧车示例（Python） | [examples/remote_plugin_minimal/README.md](examples/remote_plugin_minimal/README.md) |
| 角色 manifest 说明 | [roles/README_MANIFEST.md](roles/README_MANIFEST.md)（含应用内 **导入 `.ocpak` / `.zip` / 文件夹**） |
| 角色包导入 — 手工测试清单 | [roles/TESTING_ROLE_PACK_IMPORT.md](roles/TESTING_ROLE_PACK_IMPORT.md) |

**说明**：旧路径 `docs/*.md` 已迁移至 `creator-docs/`，见 [docs/README.md](docs/README.md)。**开发史料归档**（交接日志索引）：[ARCHIVE_PROJECT_HISTORY.md](ARCHIVE_PROJECT_HISTORY.md)。

## 仓库结构（心智模型）

| 部分 | 说明 |
|------|------|
| **运行时（本仓库）** | 玩家使用的桌面客户端 + 对话引擎 |
| **角色包** | `roles/` 下每角色一目录；**唯一对接面**为磁盘上的包目录（或 zip 解压后同等结构） |
| **角色包编写器** | **独立仓库**（例如与本仓库**同级**目录 `oclive-pack-editor`），只负责产出包，不引用本仓库源码；多根工作区可用根目录 `oclive-pack-editor.code-workspace` 同时打开两项目 |
| **启动器** | **独立仓库** [oclive-launcher](https://github.com/linkaiheng2233-cyber/oclive-launcher)：统一配置编写器与运行时路径、为 oclive 注入 **`OCLIVE_ROLES_DIR`**，并支持 **从 zip 安装角色包**（选择 `settings.json` 的 Ollama `model`、可选覆盖、**`ollama pull`**） |
| **扩展** | 见 [creator-docs/plugin-and-architecture/EXTENSION_POINTS.md](creator-docs/plugin-and-architecture/EXTENSION_POINTS.md)；HTTP 侧车见 [creator-docs/plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md](creator-docs/plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md) |

**契约与版本（摘要）**：`manifest.min_runtime_version`、根对象顶层键白名单、`validate_disk_manifest` 等以 [PACK_VERSIONING.md](creator-docs/role-pack/PACK_VERSIONING.md) 与 `RoleStorage::load_role` 为准。编写器侧 **`HOST_RUNTIME_VERSION`**（`oclive-pack-editor`）应与 **`src-tauri/Cargo.toml` 的 `version`** 一致。

## 新用户：从下载到第一次对话

1. **安装依赖**：Node.js、Ollama（对话默认走本地模型）。详见 [creator-docs/getting-started/CREATOR_WORKFLOW.md](creator-docs/getting-started/CREATOR_WORKFLOW.md)。
2. **三仓库**（可选）：[oclivenewnew](https://github.com/linkaiheng2233-cyber/oclivenewnew)（本仓库）、[oclive-pack-editor](https://github.com/linkaiheng2233-cyber/oclive-pack-editor)、[oclive-launcher](https://github.com/linkaiheng2233-cyber/oclive-launcher)。同级克隆最省事。
3. **角色包目录**：将环境变量 **`OCLIVE_ROLES_DIR`** 指向 **roles 根**（其下为各 `角色id/`）。可用启动器 **一键填入** 本仓库内 `roles/`；或使用启动器 **「从 zip 安装角色包」** 解压编写器导出包并写入模型名；亦可手动把 zip 解压到该根下。
4. **运行本应用**：`npm run tauri:dev` 启动桌面客户端；在应用内加载 `roles/` 下的角色并开始对话。

详细步骤与「编写器 → 磁盘 → 运行时」数据流见 **启动器 README** 与 [CREATOR_WORKFLOW.md](creator-docs/getting-started/CREATOR_WORKFLOW.md)。

## 环境要求

- **Node.js**（建议 18+）、**npm**
- **Rust** stable、**Ollama**（本地 LLM，默认 `OLLAMA_MODEL` 可配）
- Windows 开发需已安装 **Visual Studio Build Tools**（链接器）

## 开发

本机调试外部角色目录时，可设置环境变量 **`OCLIVE_ROLES_DIR`** 指向 **roles 根**（其下为各 `角色id/` 子目录，内含 `manifest.json`）。详见 [roles/README_MANIFEST.md](roles/README_MANIFEST.md) 与 [creator-docs/getting-started/CREATOR_WORKFLOW.md](creator-docs/getting-started/CREATOR_WORKFLOW.md)。

```bash
npm install
npm run tauri:dev
```

### 本地 HTTP API（编写器试聊 / 调试）

使用与 GUI **同一可执行文件**，加 `--api` 在 **`127.0.0.1`** 上启动仅本地访问的 HTTP 服务（默认端口 **8420**，可用 `--port` 或环境变量 **`OCLIVE_API_PORT`** 覆盖）：

```bash
# 构建后，在可执行文件所在目录：
./oclivenewnew --api
./oclivenewnew --api --port 8420
```

- **`GET /health`**：返回纯文本 `ok`。
- **`POST /chat`**：JSON 体 `{ "role_path": "D:/.../roles/某角色id", "message": "你好", "session_id": null }`，返回 `{ "reply": "..." }`。`role_path` 为含 `manifest.json` 的角色目录的**绝对或规范化路径**。

与 Tauri IPC 相同，内部走 `chat_engine::process_message`；需本机 **Ollama** 等环境可用。

仅前端静态资源：

```bash
npm run dev
npm run build
```

## 测试与检查

**CI（`.github/workflows/ci.yml`）**：在 **Ubuntu** 与 **Windows** 上均执行 Rust **`rustfmt` + `clippy`（`-D warnings`）+ 完整 `cargo test`**（含 `tests/` 集成测试），以及 **`npm ci` + `npm run build`**。用于尽早发现路径、换行符与 Windows 专用链接问题。

| 命令 | 用途 |
|------|------|
| `npm run check` | 日常开发：`vite build` + `cargo fmt` / `clippy` / **`cargo test --lib`** |
| `npm run check:release` | **发版门槛**：`vite build` + fmt / clippy + **完整 `cargo test`**（与 CI 中 Rust  job 一致） |
| `npm run check:rust:test:all` | 仅跑全量测试（已包含在 `check:release` 中） |

```bash
npm run check
```

```bash
npm run check:release
```

仅快速编译时可：

```bash
cd src-tauri
cargo check --lib
```

> Windows 上若遇 **LNK1104**（无法写入 `target\debug\*.exe`），多为文件被占用；关闭相关进程后重试。

**相关仓库 CI**：若与本项目同级检出 **oclive-pack-editor**、**oclive-launcher**，二者根目录均有 `.github/workflows/ci.yml`（Ubuntu + Windows：`npm` 构建 + `src-tauri` 的 `cargo build`；编写器在 **Linux** 上另跑 Vitest 与 Playwright E2E）。推送到远端后请在各仓库 **Actions** 中确认通过。

## 可观测性与发布

- **Sentry**：仅当构建时设置环境变量 **`VITE_SENTRY_DSN`** 时，前端会初始化 `@sentry/vue`，上报 **Vue 侧未捕获异常**；**Rust 后端错误默认不上报 Sentry**（以本地/系统日志为准）。未配置 DSN 时无任何上报。
- **在线更新**：当前 **未配置** Tauri 内置更新端点；对外分发以 **离线安装包**（`tauri build` 产物）为准。若日后启用更新器，需另行配置签名与更新源并在发行说明中写明。
- **版本与协作**：发版前请统一 **`package.json` / `src-tauri/Cargo.toml` / `tauri.conf.json` 版本号**，并更新 **`CHANGELOG.md`**；使用 Git 便于回滚与对照 CI。

## 打包

```bash
npm run build
cd src-tauri && npx tauri build
```

发版前建议先 **`npm run check:release`**。详见历史说明：`handoff/18_DEVELOPMENT_REPORT_USER_ACTIONS.md`（若仍存在）。

## 聊天记录导出

主界面支持导出 **JSON / TXT**（可选全部角色），经 `export_chat_logs` 与浏览器下载。说明见 `handoff/17_TIME_SCENE_EXPORT_HANDOFF.md`。

## 路线图状态

「完全愿景」分阶段推进，详见 [creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md](creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md)。**本仓库已落实**：开源与 CI、契约文档、扩展点索引、**HTTP JSON-RPC Remote 宿主客户端**、创作者架构说明、`PluginHost` 五类后端枚举；记忆 **与** 情绪 / 事件 / Prompt 均具备 **`builtin` + `builtin_v2` + `remote`（LLM 为 `ollama`/`remote`）** 可切换路径及回归测试、`get_role_info`/`load_role` 暴露 `plugin_backends` 等。**独立角色包编写器**为**另仓**（见上表），经同一包格式对接。**仍属路线图**：包内知识库深化、启动器等（侧车逻辑由创作者自部署，见 [creator-docs/plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md](creator-docs/plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md)）。

## 许可证

MIT，见 [LICENSE](LICENSE)。

## 贡献与安全

- [CONTRIBUTING.md](CONTRIBUTING.md)
- [SECURITY.md](SECURITY.md)

## IDE

- [VS Code](https://code.visualstudio.com/) + [Vue - Official](https://marketplace.visualstudio.com/items?itemName=Vue.volar) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
