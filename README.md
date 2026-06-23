# A.I.Live — 可插拔的角色动脉织机

> 工程仓库：**oclivenewnew**（工程代号 **oclive**）

[English](README.en.md)

[![CI](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/workflows/ci.yml/badge.svg)](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/workflows/ci.yml)

> **最近进展**：Chat Pro **0.4.0** · VS Code Flash **0.4.1** — 见 [CHANGELOG.md](CHANGELOG.md)。**AI 剧场**已从 0 规划，见 [`handoff/theater/`](handoff/theater/)。

本地优先的 **AI 角色组装平台**（开源、可组装、隐私优先）：**Tauri + Vue 3 + Rust** 运行时 + **六槽可替换模块** + **角色包独立分发** + **发行版 profile** + **插件市场**。默认角色包（如 `distros/chat-pro/roles/mumu`）为**官方示例**，展示平台能力；社区创作与分发角色包是核心价值。工程代号 **oclive**。

## 仓库布局（内核 + 发行版）

| 目录 | 内容 |
|------|------|
| **[`kernel/`](kernel/)** | Rust 内核（`kernel/crates/`、`kernel/fuzz/`、OOCP 示例等） |
| **[`distros/chat-pro/`](distros/chat-pro/)** | **OCLive Chat Pro** 前端（ToolShell / FluentShell） |
| **[`distros/theater/`](distros/theater/)** | **AI Theater** 第三发行版前端 |
| **[`distros/shared/`](distros/shared/)** | 桌面共享 UI（`@oclive/desktop-shared`） |
| **[`distros/desktop-tauri/`](distros/desktop-tauri/)** | 共享 Tauri 宿主（原 `src-tauri`） |

RFC：[handoff/distros/ARCHITECTURE_DECOUPLING_RFC.md](handoff/distros/ARCHITECTURE_DECOUPLING_RFC.md)

**人类开发者：从这里开始 → [human-docs/](human-docs/)**（30 分钟跑通：`npm install` → `npm run tauri:dev` → `npm run check`）。使用 Cursor / Agent 见 [AGENTS.md](AGENTS.md)。

| 贡献入口 | 说明 |
|----------|------|
| [human-docs/02_THIRTY_MINUTE_START.md](human-docs/02_THIRTY_MINUTE_START.md) | 30 分钟跑通与验证分级 |
| [CONTRIBUTING.md](CONTRIBUTING.md) | PR 流程、测试矩阵、模块负责人 |
| [Good first issues](https://github.com/linkaiheng2233-cyber/oclivenewnew/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22) | 新人友好任务（策展见 [handoff/GOOD_FIRST_ISSUES.md](handoff/GOOD_FIRST_ISSUES.md)） |

行为准则：[CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) · 安全报告：[SECURITY.md](SECURITY.md)

**架构（摘要）**：A.I.Live 采用 **契约型薄核** + **单核双态构建架构**——**六宿主后端模块**（memory / emotion / event / prompt / llm / agent）经 `PluginHost` 接入；**第 N 设施子模块**（如复杂情感、专家模型）等为编排行内设施模块；**后端模块插件模块** 挂第 K 模块外挂、**不占第 N 模块号**。交付：**OOCP**、角色包、**`oclive-cli` 内核工厂**；构建可选 **Monolith 宏核态**。详见 **[架构总览](creator-docs/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md)**（[English](creator-docs-en/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md)）。

## 当前工程状态（摘要）

| 领域 | 状态 |
|------|------|
| **内核编排** | 主编排在 **`kernel/crates/oclive_kernel_host/src/domain/chat_engine/mod.rs`** 的 **`process_message`**；无独立入口蓝图 DSL 主路径；子系统经 **`PluginHost`** 解析（含 **`agent`**）。 |
| **测试（三层）** | **协议层（本仓）**：`distros/desktop-tauri` 的 **`cargo test`** + `tests/` 集成测；**OOCP HTTP 黑盒 S0–S12（13 场景；可选 S13/S14）** 已入库 [`examples/oocp-test-suite/`](examples/oocp-test-suite/)，**CI 已集成** job **`oocp-test-suite`**（Ubuntu，构建 `--features dual_core` 并运行 `run.mjs --include-dual-core`）。**A1.1b**：**`vite preview` + Playwright** 首屏烟测（[`distros/chat-pro/e2e/preview-shell.spec.ts`](distros/chat-pro/e2e/preview-shell.spec.ts)），**CI 仅 Ubuntu `frontend`**（Windows `frontend` 跑 Vitest + build）。**组件层（编写器）**：**oclive-pack-editor** 仓库 Vitest / Playwright 等（与本仓 CI 分工）。**插件层（编写器）**：目录插件 / `official-vue-test-runner` 等范式与用例在 **oclive-pack-editor**。**前端最小烟测**：CI **`npm ci` + `npm run test:unit`（Vitest）+ `npm run build`**；Playwright 见上。总览见 [creator-docs/testing/OVERVIEW.md](creator-docs/testing/OVERVIEW.md)、[creator-docs/testing/OOCP_TEST_SUITE.md](creator-docs/testing/OOCP_TEST_SUITE.md)。 |
| **oclive-cli** | Workspace crate **`oclive-cli`**：**`oclive dev`**（监听 `distros/chat-pro/roles/`）；**`bench`**（`--save` / `--compare` / **`--cold-start`**）；**`test --coverage`** / **`--miri`**；**`explain`** / **`completions`**；**`init --dry-run`** / **`--check`**；**`lint --audit-ci`**；**`doctor --sbom`**；**`pack`** 与 **Monolith** 流程见 [creator-docs/cli/OCLIVE_CLI_GUIDE.md](creator-docs/cli/OCLIVE_CLI_GUIDE.md)。 |
| **启动健康检查** | 首轮 **`process_message`** 前一次性自检（槽位、角色包文件、SQLite **`health_ping`**、可选 LLM 探测）；可用 **`OCLIVE_SKIP_STARTUP_HEALTH`** / **`OCLIVE_SKIP_LLM_STARTUP_PROBE`** 跳过。实现见 `kernel/crates/oclive_kernel_host/src/domain/startup_health.rs`。 |
| **Monolith（高耦合编译）** | 无头脚手架在编译期按 **七焊接键**（第 1–6 模块 + `complex_emotion`）焊接静态路径；RFC 与 CLI 四阶段（`init` → `build` → 双二进制 `bench`）见 [creator-docs/rfc/RFC_OCLIVE_MONOLITH_MODE.md](creator-docs/rfc/RFC_OCLIVE_MONOLITH_MODE.md) 与上文 CLI 指南。 |
| **安全** | 已跑 **`cargo audit`（0.22.1）**；**漏洞级已清零**（警告级仍跟踪；数字以 [KNOWN_VULNERABILITIES.md](creator-docs/security/KNOWN_VULNERABILITIES.md) 为准），见该文「维护约定」；审查边界见 [creator-docs/security/SECURITY_AUDIT_SCOPE.md](creator-docs/security/SECURITY_AUDIT_SCOPE.md)。 |
| **CI 守门** | **`rustfmt` + workspace `clippy`（`-D warnings`）+ `cargo test --workspace`** + **`npm ci` / `npm run test:unit` / `npm run build`**；另含 **`oocp-test-suite`**、**`layering-ratchet`**、**`dimension5-acceptance`**、**`cross-host-e2e`**（含 profile 调度）、**`cargo-audit`**（失败即红；**`Cargo.lock` PR** 另走严格 job）、**`npm-audit`**（可见性）与 **remote-plugin-demo**。 |
| **轻量化基线** | [creator-docs/development/LIGHTWEIGHT_PROFILE.md](creator-docs/development/LIGHTWEIGHT_PROFILE.md)（Release、`cargo-bloat` 采样）。 |

协作说明见根目录 **[AGENTS.md](AGENTS.md)**。

## 贡献者三十分钟路径（主仓）

```bash
npm install
npm run tauri:dev          # 桌面客户端 + 热重载
npm run check              # 日常门禁（build + fmt + clippy + test --lib）
```

Windows 需 **Visual Studio Build Tools**（MSVC）；Cargo 产物在仓库外 `../oclive-dev-artifacts/oclivenewnew-cargo-target/`。分步说明与验收：[human-docs/02_THIRTY_MINUTE_START.md](human-docs/02_THIRTY_MINUTE_START.md) · 完整贡献流程：[CONTRIBUTING.md](CONTRIBUTING.md)。

## 开发者入口

| 目标 | 文档 |
|------|------|
| **人类接手包**（窄入口 · 学习阶梯 L0–L7） | **[human-docs/README.md](human-docs/README.md)** |
| **脚手架**（`oclive-cli init` / `dev` / `bench`） | [OCLIVE_CLI_GUIDE.md](creator-docs/cli/OCLIVE_CLI_GUIDE.md) |
| **角色包规范**（manifest / settings / 身份 / 后处理） | [ROLE_PACK_SPEC.md](creator-docs/role-pack/ROLE_PACK_SPEC.md) |
| **发行版 profile**（`distro.oclive.toml` / 多宿主 attach） | [DISTRO_KERNEL_LIFECYCLE.md](creator-docs/kernel/DISTRO_KERNEL_LIFECYCLE.md) · [DISTRO_CAPABILITY_PROFILE.md](creator-docs/kernel/DISTRO_CAPABILITY_PROFILE.md) |
| **架构与六槽契约** | [OCLIVE_ARCHITECTURE_OVERVIEW.md](creator-docs/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md) |
| **定位与差异化**（对外叙事） | [handoff/OCLIVE_POSITIONING_DIFFERENTIATION.md](handoff/OCLIVE_POSITIONING_DIFFERENTIATION.md) |

## 快速开始（内核工厂 / oclive-cli）

约 5 分钟从零到可编译内核工程（纯对话、极速模板）见 **[内核工厂愿景 · 5 分钟从零到对话](creator-docs/getting-started/KERNEL_FACTORY_VISION.md#5-分钟从零到对话纯内核脚手架)**（[English](creator-docs-en/getting-started/KERNEL_FACTORY_VISION.md)）。入口命令：`oclive doctor` → `oclive init --quick`。

### 平台能力（oclive-cli）

除 **`init` / `build` / `bench` / `dev` / `pack` / `plugin` / `doctor`** 外，还支持：**`registry`**（本地工程清单）、**`compose`**（多实例 YAML 编排）、**`publish`** 与 **`init --template-url`**（模板分享）、**`init --tui`**（模板可视化）、**`bench --watch`**（变更触发基准）、**`debug`**（`process_message` 逐步追踪）。见 [OCLIVE_CLI_GUIDE.md](creator-docs/cli/OCLIVE_CLI_GUIDE.md) 与 [KERNEL_FACTORY_VISION.md](creator-docs/getting-started/KERNEL_FACTORY_VISION.md)。

## 性能

对外披露的 **Release 二进制 `cargo-bloat` 采样、Monolith 与 `oclive bench` 方法、已知产品向限制** 见 **[creator-docs/getting-started/PERFORMANCE.md](creator-docs/getting-started/PERFORMANCE.md)**（[English](creator-docs-en/getting-started/PERFORMANCE.md)）。数值以 [LIGHTWEIGHT_PROFILE.md](creator-docs/development/LIGHTWEIGHT_PROFILE.md) §6.7 最新采样为准。

## 支持

- **唯一反馈入口**：[**GitHub Issues**](https://github.com/linkaiheng2233-cyber/oclivenewnew/issues)（本仓库）。  
- **Issue 标题建议**：`[bug]: …` · `[feat]: …` · `[support]: …`（与模板 `title` 前缀一致，便于筛选）。  
- **首次响应**：维护者通常在 **3–5 个工作日** 内完成首轮分类（志愿维护窗口，**非合同 SLA**；节假日顺延）。  
- **请附带环境信息**：**操作系统**；**应用版本**（例如 `package.json` / `distros/desktop-tauri/Cargo.toml` 的 `version`）；**`oclive-cli` 版本**（`kernel/crates/oclive-cli/Cargo.toml` 的 `version` 或 `cargo run -p oclive-cli -- --help` 输出）；并粘贴应用内 **设置 → 常规 → 环境自检** 的结果摘要。**勿**在公开 issue 中粘贴 API 密钥、Token 或可识别隐私的完整本机路径。

**自助排查**：[用户手册](creator-docs/getting-started/USER_MANUAL.md)（[English](creator-docs-en/getting-started/USER_MANUAL.md)）· [FAQ](creator-docs/FAQ.md) · [文档索引](creator-docs/getting-started/DOCUMENTATION_INDEX.md) · [ERROR_CODES](creator-docs/getting-started/ERROR_CODES.md)。报告缺陷请尽量包含 **错误码** 与 **最少复现步骤**。

## 早期采用者与已知限制

- 当前以 **0.2.x** 桌面宿主为主；**在线更新器未配置**，分发以 **离线安装包** 为准（见下文「可观测性与发布」）。  
- **Ollama** 为本地对话默认路径；未安装或模型未拉取时对话会失败——请见 [CREATOR_WORKFLOW.md](creator-docs/getting-started/CREATOR_WORKFLOW.md) 与 [ERROR_CODES.md](creator-docs/getting-started/ERROR_CODES.md)（§1.5 首装常见）。  
- **Remote / 目录插件 / MCP** 涉及出站网络或子进程时，须按 manifest 与宿主授权流程使用（见 [DIRECTORY_PLUGINS.md](creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md)）。  
- **产品级首发 P0**：**A1（可 CI 子集）**已收口（HTTP 重启、`vite preview`+Playwright、九条 `invoke` 热路径；见 [handoff/PRODUCT_RELEASE_CHECKLIST.md](handoff/PRODUCT_RELEASE_CHECKLIST.md)）；**A2 首装文案 / 离线弱网** 等仍在推进，维护者用 [handoff/PRODUCT_AND_KERNEL_GAP_CHECKLIST.md](handoff/PRODUCT_AND_KERNEL_GAP_CHECKLIST.md) 跟踪。

## 模型、插件与数据（速览三问）

1. **第三方模型与 API**：默认 **本地 Ollama**；若使用云端或侧车，密钥与出站网络由 **用户与侧车配置** 负责 — 见 [creator-docs/getting-started/SIDECAR_LLM_USER_GUIDE.md](creator-docs/getting-started/SIDECAR_LLM_USER_GUIDE.md) 与 [LICENSE_POLICY.md](creator-docs/LICENSE_POLICY.md)。  
2. **插件**：须遵守 **manifest 权限** 与宿主授权；主程序以 **Apache-2.0** 发布，见 [LICENSE](LICENSE) 与 [LICENSE_POLICY.md](creator-docs/LICENSE_POLICY.md)。  
3. **用户数据落盘**：SQLite 与 `{app_data}` 路径见 [creator-docs/guides/CONFIGURATION_FILES.md](creator-docs/guides/CONFIGURATION_FILES.md)；勿在公开渠道粘贴含隐私的路径全文。

## 平台愿景（开放实验场）

在 **本地优先、可替换子系统、角色包为唯一对接面** 的前提下，A.I.Live 希望成为创作者与玩家都能 **安全实验** 的桌面底座：契约与 CI 守住兼容边界，侧车与目录式插件降低扩展成本。**愿景摘要**见 [creator-docs/roadmap/VISION_OPEN_LAB.md](creator-docs/roadmap/VISION_OPEN_LAB.md)；分阶段路线见 [creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md](creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md)。

## 文档（创作者与扩展）

**入口**：[creator-docs/README.md](creator-docs/README.md)（含目录说明与阅读顺序）

| 说明 | 路径 |
|------|------|
| **用户手册**（安装 → 导入角色包 → 日常对话） | [creator-docs/getting-started/USER_MANUAL.md](creator-docs/getting-started/USER_MANUAL.md)（[English](creator-docs-en/getting-started/USER_MANUAL.md)） |
| 文档总索引 | [creator-docs/getting-started/DOCUMENTATION_INDEX.md](creator-docs/getting-started/DOCUMENTATION_INDEX.md) |
| **项目全貌（三件套、事项、命令）** | [creator-docs/getting-started/PROJECT_OVERVIEW.md](creator-docs/getting-started/PROJECT_OVERVIEW.md) |
| **GitHub：CI / Dependabot / 网页设置清单** | [creator-docs/getting-started/GITHUB_REPO_CHECKLIST.md](creator-docs/getting-started/GITHUB_REPO_CHECKLIST.md) |
| 愿景与按月路线 | [creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md](creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md) |
| **开放实验场（愿景摘要）** | [creator-docs/roadmap/VISION_OPEN_LAB.md](creator-docs/roadmap/VISION_OPEN_LAB.md) |
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
| 侧车范例：OpenAI 兼容 API | [examples/remote_plugin_openai_compat/README.md](examples/remote_plugin_openai_compat/README.md) |
| 侧车示例共用模块（JSON-RPC / 非 LLM 占位） | [examples/common/README.md](examples/common/README.md) |
| 角色 manifest 说明 | [distros/chat-pro/roles/README_MANIFEST.md](distros/chat-pro/roles/README_MANIFEST.md)（含应用内 **导入 `.ocpak` / `.zip` / 文件夹**） |
| **性格档案（核心 / 可变 / 七维视图）** | [docs/personality-archive-notes.md](docs/personality-archive-notes.md)（与 `evolution.personality_source` 对齐） |
| **设计思路演进记录** | [docs/design-axis-evolution.md](docs/design-axis-evolution.md) |
| 角色包导入 — 手工测试清单 | [distros/chat-pro/roles/TESTING_ROLE_PACK_IMPORT.md](distros/chat-pro/roles/TESTING_ROLE_PACK_IMPORT.md) |

**说明**：旧路径 `docs/*.md` 已迁移至 `creator-docs/`，见 [docs/README.md](docs/README.md)。**开发史料归档**（交接日志索引）：[ARCHIVE_PROJECT_HISTORY.md](handoff/archive/ARCHIVE_PROJECT_HISTORY.md)。

## 仓库结构（心智模型）

| 部分 | 说明 |
|------|------|
| **运行时（本仓库）** | 玩家使用的桌面客户端 + 对话引擎 |
| **角色包** | `distros/chat-pro/roles/` 下每角色一目录；**唯一对接面**为磁盘上的包目录（或 zip 解压后同等结构） |
| **角色包编写器** | **独立仓库** [oclive-pack-editor](https://github.com/linkaiheng2233-cyber/oclive-pack-editor)（与本仓库**同级**目录常见）：产出 v2 角色包；人设 / 六槽 / 知识 / 导出 |
| **启动器（已退役）** | [oclive-launcher](https://github.com/linkaiheng2233-cyber/oclive-launcher) 仅归档；新用户用 **编写器 + 本运行时**，无需第三应用 |
| **扩展** | 见 [creator-docs/plugin-and-architecture/EXTENSION_POINTS.md](creator-docs/plugin-and-architecture/EXTENSION_POINTS.md)；HTTP 侧车见 [creator-docs/plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md](creator-docs/plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md)；**目录式插件**（整壳 / 嵌入插槽、manifest）见 [creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md](creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md) |

**契约与版本（摘要）**：`manifest.min_runtime_version`、根对象顶层键白名单、`validate_disk_manifest` 等以 [PACK_VERSIONING.md](creator-docs/role-pack/PACK_VERSIONING.md) 与 `RoleStorage::load_role` 为准。编写器侧 **`HOST_RUNTIME_VERSION`**（`oclive-pack-editor`）应与 **`distros/desktop-tauri/Cargo.toml` 的 `version`** 一致。

## 快速开始：编写器 + 运行时

1. **安装依赖**：Node.js、Ollama（本地对话默认路径）。详见 [CREATOR_WORKFLOW.md](creator-docs/getting-started/CREATOR_WORKFLOW.md)。
2. **克隆两仓**（同级目录最省事）：**本仓库**（A.I.Live 运行时）与 **[oclive-pack-editor](https://github.com/linkaiheng2233-cyber/oclive-pack-editor)**（角色包编写器）。
3. **制作角色包**：在编写器中编辑并 **导出 zip / 写入文件夹**，或复制 `distros/chat-pro/roles/mumu/` 等示例；使 **蓝图文件 `distros/chat-pro/roles/{角色id}/pipeline.ocblueprint`** 位于 **roles 根**（**不以** `steps[]` 作主路径调度；本项目内 `distros/chat-pro/roles/`，或设置 **`OCLIVE_ROLES_DIR`**）。
4. **运行本应用**：`npm run tauri:dev`（或 Release 安装包）；加载角色并开始对话。
5. **（可选）高级能力**：在本应用 **插件与后端管理 → 架构图** 配置 **专家路由**（`expert_routing.json`）、`groups` 等；之后在编写器保存人设时，编写器会保留这些蓝图扩展字段。

旧版 **oclive-launcher** 已退役，见 [oclive-launcher README](https://github.com/linkaiheng2233-cyber/oclive-launcher/blob/main/README.md)。数据流：**编写器 → 磁盘角色包 → 本应用 `load_role`**。

## 环境要求

- **Node.js**（建议 18+）、**npm**
- **Rust** stable、**Ollama**（本地 LLM，默认 `OLLAMA_MODEL` 可配）
- Windows 开发需已安装 **Visual Studio Build Tools**（链接器）

## 开发

本机调试外部角色目录时，可设置环境变量 **`OCLIVE_ROLES_DIR`** 指向 **roles 根**（其下为各 `角色id/` 子目录，内含 `manifest.json`）。详见 [distros/chat-pro/roles/README_MANIFEST.md](distros/chat-pro/roles/README_MANIFEST.md) 与 [creator-docs/getting-started/CREATOR_WORKFLOW.md](creator-docs/getting-started/CREATOR_WORKFLOW.md)。

**插件开发（目录式插件）**：支持以 **Vue 单文件组件** 作为嵌入插槽 UI，契约与整壳、iframe 回退、事件订阅与安全选项见 **[DIRECTORY_PLUGINS.md](creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md)**。

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
- **`POST /chat`**：JSON 体 `{ "role_path": "D:/.../distros/chat-pro/roles/某角色id", "message": "你好", "session_id": null }`，成功时扁平字段含 **`reply`**、`personality_source` 等（与 Tauri `send_message` 契约一致）。`role_path` 为含 `manifest.json` 的角色目录的**绝对或规范化路径**。

与 Tauri IPC 相同，内部走 `chat_engine::process_message`；需本机 **Ollama** 等环境可用。自动化/CI 可对进程设置 **`OCLIVE_HTTP_API_MOCK_LLM=1`** 以使用内存库 + Mock LLM（不访问网络），详见 [`examples/oocp-test-suite/README.md`](examples/oocp-test-suite/README.md)。

仅前端静态资源：

```bash
npm run dev
npm run build
```

## 测试与检查

**主路径快捷键（应用内）**：**Ctrl+Shift+F** 打开插件管理、**Ctrl+Shift+S** 打开设置、**Ctrl+Shift+D** 开关调试面板；完整说明见应用内 **设置** 相关文案与 `distros/shared/src/i18n` 中 **`shortcutHelp`**（与 [FAQ](creator-docs/FAQ.md) 一致）。

**CI（`.github/workflows/ci.yml`）**：在 **Ubuntu** 与 **Windows** 上均执行 Rust **`rustfmt` + workspace `clippy`（`-D warnings`）+ `cargo test --workspace`**，以及 **`npm ci` + `npm run test:unit` + `npm run build`**。**Ubuntu** 的 **`frontend`** job 另跑 **Playwright + `vite preview` 首屏烟测（A1.1b）**；**Windows** 的 **`frontend`** job 不跑 Playwright（避免子进程拉起 `vite preview` 不稳定），以 **Vitest + build** 为主。在 **Ubuntu** 上另跑 **`oocp-test-suite`**（`--api` + `examples/oocp-test-suite/run.mjs --include-dual-core`，默认 S0–S12 + 可选双核 S13/S14；随后 **`scripts/e2e-core-api-restart.mjs`** 进程重启烟测）、**`layering-ratchet`**、**`dimension5-acceptance`**、**`cross-host-e2e`**（profile 调度）、**`cargo-audit`（0.22.1，失败即红；`Cargo.lock` PR 另走严格 job）**、**`npm-audit`（可见性）** 与 **`remote-plugin-demo`**。**组件 / 插件层**自动化在 **oclive-pack-editor** 各自 workflow 中维护（见上文「测试（三层）」）。

| 命令 | 用途 |
|------|------|
| `npm run test:unit` | **Vitest**：主仓最小烟测（`distros/chat-pro/src/smoke.test.ts`） |
| `npm run test:e2e:core-api-restart` | **A1.1a**：`--api` 进程 **重启后** 仍能 `/health` + `POST /chat`（需先 `cargo build -p oclivenewnew-tauri`；默认 Mock LLM） |
| `npm run test:e2e:preview` | **A1.1b**：**`vite preview`** 下 **Playwright** 首屏烟测（需先 **`npm run build`**；首次本地需 `npx playwright install chromium`） |
| `npm run build:analyze` | 生成前端打包体积报告（`dist/stats.html`，用于定位大 chunk） |
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

- **Sentry**：仅当构建时设置环境变量 **`VITE_SENTRY_DSN`** 时，前端**可能**初始化 `@sentry/vue`，上报 **Vue 侧未捕获异常**（`sendDefaultPii: false`，请求 URL 去掉 query）；**Rust 后端错误默认不上报 Sentry**（以本地/系统日志为准）。未配置 DSN 时无任何上报。若构建已带 DSN，用户可在 **设置 → 常规** 勾选 **「禁用崩溃上报」**，将偏好写入本机 **`localStorage`**（键 **`oclive.telemetry.sentryOptOut`**，`1` 表示退出）；取消勾选后需**重启应用**才会重新初始化上报。
- **在线更新**：当前 **未配置** Tauri 内置更新端点；对外分发以 **离线安装包**（`tauri build` 产物）为准。若日后启用更新器，需另行配置签名与更新源并在发行说明中写明。
- **版本与协作**：发版前请统一 **`package.json` / `distros/desktop-tauri/Cargo.toml` / `tauri.conf.json` 版本号**，并更新 **`CHANGELOG.md`** 与 **`CHANGELOG.en.md`**（用户可见变更保持中英同步）；使用 Git 便于回滚与对照 CI。

## 打包

```bash
npm run build
cd src-tauri && npx tauri build
```

发版前建议先 **`npm run check:release`**。详见历史说明：`handoff/18_DEVELOPMENT_REPORT_USER_ACTIONS.md`（若仍存在）。

## 聊天记录导出

主界面支持导出 **JSON / TXT**（可选全部角色），经 `export_chat_logs` 与浏览器下载。说明见 `handoff/17_TIME_SCENE_EXPORT_HANDOFF.md`。

## 路线图状态

「完全愿景」分阶段推进，详见 [creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md](creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md)。**本仓库已落实**：开源与 CI、契约文档、扩展点索引、**HTTP JSON-RPC Remote 宿主客户端**、创作者架构说明、`PluginHost` 五类后端枚举；记忆 **与** 情绪 / 事件 / Prompt 均具备 **`builtin` + `remote`（LLM 为 `ollama`/`remote`）** 可切换路径及回归测试、`get_role_info`/`load_role` 暴露 `plugin_backends` 等（`builtin_v2` 仅为已废弃 wire alias，读入等同 `builtin`）。**独立角色包编写器**为**另仓**（见上表），经同一包格式对接。**仍属路线图**：包内知识库深化、启动器等（侧车逻辑由创作者自部署，见 [creator-docs/plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md](creator-docs/plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md)）。

## 免责声明

关于 **模型权重与许可证**、**第三方插件责任**、**本地数据与遥测** 的完整说明见 **[creator-docs/legal/DISCLAIMER.md](creator-docs/legal/DISCLAIMER.md)**（[English](creator-docs-en/legal/DISCLAIMER.md)）。安全审查边界见 [SECURITY_AUDIT_SCOPE.md](creator-docs/security/SECURITY_AUDIT_SCOPE.md) 中的「第三方风险」。

## 许可证

**Apache License 2.0**（SPDX: `Apache-2.0`）— 见 [LICENSE](LICENSE) 与 [NOTICE](NOTICE)。策略说明见 [LICENSE_POLICY.md](creator-docs/LICENSE_POLICY.md)。

## 贡献与安全

- [CONTRIBUTING.md](CONTRIBUTING.md)
- [SECURITY.md](SECURITY.md)

## IDE

- [VS Code](https://code.visualstudio.com/) + [Vue - Official](https://marketplace.visualstudio.com/items?itemName=Vue.volar) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
