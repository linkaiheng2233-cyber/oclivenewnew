# 贡献指南

[English](CONTRIBUTING.en.md)

感谢考虑为 oclive 做贡献。项目目标见 [creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md](creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md)。

## GitHub 仓库（CI、Dependabot、分支保护）

合并默认分支后，**Dependabot** 会按 [`.github/dependabot.yml`](.github/dependabot.yml) 开依赖更新 PR；**CI** 见 Actions。若你维护组织/仓库设置（分支保护、Secrets 等），见 **[creator-docs/getting-started/GITHUB_REPO_CHECKLIST.md](creator-docs/getting-started/GITHUB_REPO_CHECKLIST.md)**。

## 获取帮助

- **一般问题、安装与配置**：请使用仓库 [**GitHub Issues**](https://github.com/linkaiheng2233-cyber/oclivenewnew/issues)，并选用 **Bug / Feature / Support** 模板；标题建议 `[bug]:` / `[feat]:` / `[support]:` 前缀（见根目录 [README.md](README.md)「支持」小节）。维护者通常在 **3–5 个工作日** 内做首轮分类（非 SLA）。  
- **自助材料**：[FAQ](creator-docs/FAQ.md) · [文档索引](creator-docs/getting-started/DOCUMENTATION_INDEX.md) · [ERROR_CODES](creator-docs/getting-started/ERROR_CODES.md)。  
- **安全漏洞**：**勿**在公开 issue 披露细节 — 见 [SECURITY.md](SECURITY.md)。

## 开发环境

- **本仓库**：**Node.js**（建议 18+）、**npm**、**Rust** stable、**Ollama**（本地对话默认路径，可选）。
- **Windows**：需 **Visual Studio Build Tools**（MSVC 链接器）。
- **克隆后**：在仓库根目录执行 **`npm install`**；首次 **`npm run tauri:dev`** 会拉取前端依赖并由 Tauri 驱动 `src-tauri` 构建。
- **仅验证 Rust workspace**（含 `oclive_validation`、`oclive-cli`、`oclivenewnew-tauri`）：在根目录执行 **`cargo test --workspace`**，或 **`cargo test --manifest-path src-tauri/Cargo.toml`** 仅桌面宿主。
- **Cargo 产物目录**：根目录 [`.cargo/config.toml`](.cargo/config.toml) 将 **`target-dir`** 指到仓库外 **`../oclive-dev-artifacts/oclivenewnew-cargo-target/`**；与源码分离，便于清理。

## 构建与本地运行

```bash
npm install
npm run tauri:dev          # 桌面客户端 + 热重载
# 或仅前端静态站
npm run dev
npm run build
```

**本地 HTTP API**（与 GUI 同一二进制）：`./oclivenewnew-tauri` / 安装包可执行文件加 **`--api`**，见根目录 [README.md](README.md)「本地 HTTP API」节。

## 代码规范（Rust / Vue）

- **Rust**
  - **格式化**：`cargo fmt`；CI 与 **`npm run check:rust:fmt`** 使用 **`cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check`**。
  - **Clippy**：工作区根 **[`Cargo.toml`](Cargo.toml)** 定义 **`[workspace.lints.rust]`**（如 **`unsafe_code = "forbid"`**）与 **`[workspace.lints.clippy]`**（如 **`missing_errors_doc`**、**`missing_panics_doc`**、**`must_use_candidate`** 等 **`warn`**）。本地与 CI 使用 **`cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`**（见 **`npm run check:rust:clippy`**），即 **所有 Clippy 告警在 CI 中视为错误**。
  - **`unwrap` / `expect`**：业务代码优先 **`Result` / `Option` + `context`**；集成测试等可在 crate 顶部 **`#![allow(clippy::unwrap_used, clippy::expect_used)]`**（与现有 `tests/*.rs` 一致）。**勿**在无关路径放宽 lint。
- **Vue / TypeScript**：与现有 composables、stores 风格一致；与 Tauri 契约字段对齐（如 **`reply`**，见 `oclive_kernel_runtime` 中 DTO 定义，经 `src-tauri/src/models/mod.rs` 再导出）。

## 提交规范

- 采用 **[约定式提交](https://www.conventionalcommits.org/zh-hans/v1.0.0/)** 风格：`类型(可选范围): 简短描述`。
- **常用类型**：**`feat`**、**`fix`**、**`docs`**、**`chore`**、**`refactor`**、**`test`**、**`perf`**、**`ci`**。
- **示例**：**`docs: update README feature matrix`**；**`fix(chat): handle empty session id`**。

## 测试要求（合并前建议全绿）

| 场景 | 命令 |
|------|------|
| 日常开发（与 `npm run check` 对齐） | **`npm run check`**（`vite build` + **`cargo fmt` / `clippy` / `cargo test --lib`**，manifest 指向 `src-tauri`） |
| 发版或改引擎 / 契约前 | **`npm run check:release`**（含 **`cargo test`** 全量，即 **`tests/`** 集成与单元） |
| 仅 Rust workspace | **`cargo test --workspace`**（根目录；含 `crates/*` 与 `src-tauri`） |
| 仅前端单元 | **`npm run test:unit`**（Vitest） |
| **核心 HTTP 重启烟测（A1.1a）** | **`npm run test:e2e:core-api-restart`**（需已 `cargo build -p oclivenewnew-tauri`；默认 `OCLIVE_HTTP_API_MOCK_LLM=1`） |
| **Web 预览壳 E2E（A1.1b）** | **`npm run build && npm run test:e2e:preview`**（Playwright + `vite preview`；**CI 仅 Ubuntu `frontend`**）。**Windows 本地**：若内置 `webServer` 超时，请先 **`npm run preview -- --host 127.0.0.1 --port 4180 --strictPort`**，再在另一终端 **`$env:PW_TEST_USE_EXTERNAL='1'`**（PowerShell）后执行 **`npm run test:e2e:preview`** |

**CI 对齐（重要）**：**`npm run check:release` 不包含 `npm run test:unit`**；CI 在 **`frontend`** job 中对 **Ubuntu / Windows** 执行 **`npm run test:unit`** 与 **`npm run build`**；**Playwright（`npm run test:e2e:preview`）仅在 Ubuntu `frontend`** 执行（见 `.github/workflows/ci.yml`）。发版前建议本地补跑 **`npm run test:unit`**；有前端改动时，在 **Linux/macOS** 可 **`npm run build && npm run test:e2e:preview`**，或确认 **Actions → frontend（ubuntu）** 已绿。完整发版勾选见 [handoff/PRODUCT_RELEASE_CHECKLIST.md](handoff/PRODUCT_RELEASE_CHECKLIST.md)。

**依赖审计**：`cargo udeps` 需 **nightly** toolchain（`rustup run nightly cargo udeps --workspace`）；stable 下以 `cargo clippy` / `cargo test` 与手工审阅 `Cargo.toml` 为准。

**CI 对齐**：**`.github/workflows/ci.yml`** 在 **Ubuntu 22.04** / Windows 上跑 **`rust`**（fmt / clippy / test）；**`rust` job 在 clippy 前先 `npm ci && npm run build`**（Tauri 1.5 `generate_context!` 需要仓库根 `dist/`）。Linux 构建 `oclivenewnew-tauri` 需 **`libwebkit2gtk-4.0-dev`** 等（Tauri 1.x，非 4.1）。**`frontend`** 跑 **`npm run test:unit`** 与 **`npm run build`**；**Ubuntu `frontend`** 另跑 **`npm run test:e2e:preview`**。**`oocp-test-suite`** 与 **`cli` / `cli-bench`** 在 Ubuntu 22.04。详见根目录 [README.md](README.md)「测试与检查」。

## PR 流程

1. **Fork / 功能分支**，一条 PR 聚焦一类变更；契约（manifest、DTO、`PLUGIN_V1`）变更需 **同步文档** 与 **`crates/oclive_validation`**（若适用）。
2. **描述**：说明动机、行为变化、风险与手动验证步骤；关联 issue（若有）。
3. **自检**：至少 **`npm run check`**；触及持久化 / HTTP / 编排时建议 **`npm run check:release`**。
4. **Review**：关注 CI 红绿、安全与本地化文案；大功能建议先开 issue 对照路线图。

## 破坏性变更（Breaking changes）

**完整流程、兼容层要求、PR/迁移模板**：必读 **[`handoff/BREAKING_CHANGE_PROCESS.md`](handoff/BREAKING_CHANGE_PROCESS.md)**（§C2 工程纪律；与 [`PRODUCT_AND_KERNEL_GAP_CHECKLIST.md`](handoff/PRODUCT_AND_KERNEL_GAP_CHECKLIST.md) 对齐）。

摘要：

1. **先开 issue**（或对大面变更开 RFC），说明对角色包、`plugin_backends`、HTTP OOCP / `invoke` DTO 的迁移影响；PR 描述中显式标注 **BREAKING**。  
2. **PR 须带**：`crates/oclive_validation` 更新（若 manifest / `settings` 键变更）、**`PLUGIN_V1.md` / `ERROR_CODES.md` / `COMPATIBILITY.md`** 等触及项、**`creator-docs/`** / **`creator-docs-en/`** 镜像，以及 **`CHANGELOG.md` / `CHANGELOG.en.md`** 双语条目。  
3. **审阅**：至少一名维护者确认 **兼容层与迁移路径**、CI 与 [PRODUCT_RELEASE_CHECKLIST.md](handoff/PRODUCT_RELEASE_CHECKLIST.md) P0 行。

## 文档约定

- **用户可见文案**：避免多处硬编码漂移（参见 [AGENTS.md](AGENTS.md) 中插件管理入口说明）。
- **契约与表名**：以 `roles/README_MANIFEST.md`、`RoleStorage::load_role` 及 **`crates/oclive_validation`** 为准；**禁止**虚构数据库表名。
- **创作者文档索引**：[creator-docs/getting-started/DOCUMENTATION_INDEX.md](creator-docs/getting-started/DOCUMENTATION_INDEX.md)。
- **发版与兼容**：semver bump 或契约变更时，核对 [`creator-docs/COMPATIBILITY.md`](creator-docs/COMPATIBILITY.md) 快照与一页表，并过 [handoff/PRODUCT_RELEASE_CHECKLIST.md](handoff/PRODUCT_RELEASE_CHECKLIST.md)「对外说明」；角色包版本规则见 [PACK_VERSIONING.md](creator-docs/role-pack/PACK_VERSIONING.md)。

## 不要提交

- 密钥、Token、个人路径；勿将 `.env` 提交入库（见 `.gitignore`）。
- 若本地仍有历史目录 **`src-tauri/target/`**，可删除；发行 bundle 以外置 **`target-dir`** 下的 **`release/bundle/`** 为准。

## 讨论与路线图

大改动建议先开 issue 或对照路线图中的月份目标，避免与「运行时 / 编写器」分工冲突。
