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

**依赖审计**：`cargo udeps` 需 **nightly** toolchain。最近一次全 workspace 扫描（**2026-05-22**，`rustup run nightly cargo udeps --workspace --all-targets`）：**无未使用依赖**（`All deps seem to have been used`）。复现：`rustup toolchain install nightly` 后执行上述命令。

**CI 对齐**：**`.github/workflows/ci.yml`** 在 **Ubuntu 22.04** / Windows 上跑 **`rust`**（fmt / clippy / test）；**`rust` job 在 clippy 前先 `npm ci && npm run build`**（Tauri 1.5 `generate_context!` 需要仓库根 `dist/`）。Linux 构建 `oclivenewnew-tauri` 需 **`libwebkit2gtk-4.0-dev`** 等（Tauri 1.x，非 4.1）。**`frontend`** 跑 **`npm run test:unit`** 与 **`npm run build`**；**Ubuntu `frontend`** 另跑 **`npm run test:e2e:preview`**。**`oocp-test-suite`** 与 **`cli` / `cli-bench`** 在 Ubuntu 22.04。详见根目录 [README.md](README.md)「测试与检查」。

## 模块负责人（当前维护者）

| Crate / 区域 | 路径 | 负责人 | 说明 |
|--------------|------|--------|------|
| 桌面宿主 | `src-tauri/` | @linkaiheng2233-cyber | Tauri IPC、HTTP `--api`、`AppState` |
| 内核编排 | `src-tauri/src/domain/chat_engine/` | 同上 | `process_message` / `co_present` |
| 内核 crate | `crates/oclive_kernel_types` | 同上 | DTO、`AppError` |
| 内核 crate | `crates/oclive_kernel_contracts` | 同上 | 端口 trait |
| 内核 crate | `crates/oclive_kernel_runtime` | 同上 | 编排与 re-export |
| 校验 | `crates/oclive_validation` | 同上 | manifest / v2 蓝图 |
| CLI | `crates/oclive-cli` | 同上 | `init` / `bench` / `test` / `doctor` |
| 前端 | `src/`（Vue） | 同上 | Pinia、插件管理、i18n |
| 文档 | `creator-docs/`、`handoff/` | 同上 | 契约与发版清单 |

更细入口见 **[`handoff/BUS_FACTOR_NOTES.md`](handoff/BUS_FACTOR_NOTES.md)**（含 crate 拆分后路径）。

## 代码导航（按问题域）

| 你想… | 从这里开始 |
|--------|------------|
| 理解一条消息如何走完 | `src-tauri/src/domain/chat_engine/process_message.rs` → `co_present.rs` |
| 改多实例槽合并规则 | `src-tauri/src/domain/slot_runner.rs`（读函数头「为何」注释） |
| 改插件后端解析 | `src-tauri/src/domain/plugin_host.rs` + `slot_resolver.rs` |
| 改蓝图加载 / 写盘 | `src-tauri/src/infrastructure/storage.rs`；校验在 `crates/oclive_validation` |
| 实现目录 / Remote 插件 | `creator-docs/plugin-and-architecture/PLUGIN_V1.md` + `crates/oclive_kernel_contracts` 对应 trait |
| 改 HTTP / Tauri 契约 | `src-tauri/src/models/`、`src-tauri/src/api/`、`creator-docs/getting-started/ERROR_CODES.md` |
| 理解架构取舍 | [`creator-docs/architecture/DESIGN_DECISIONS.md`](creator-docs/architecture/DESIGN_DECISIONS.md) |

## 常见修改场景

| 场景 | 建议改动位置 | 还需同步 |
|------|----------------|----------|
| 新增槽位类型或合并策略 | `slot_runner.rs`、`slot_resolver.rs`、`oclive_validation`（schema + 校验） | `ROLE_PACK_SPEC.md`、前端 `slotRegistry` / 架构图 |
| 新增插件后端种类 | `plugin_host.rs`（`BackendRegistry`）、`models` 枚举、`PLUGIN_V1.md` | `settings.json` / 蓝图 `slot_registry` 文档 |
| 调整共景阶段顺序 | `co_present.rs`（**慎重**；属主编排） | `DESIGN_DECISIONS.md`、OOCP / 集成测 |
| 新持久化字段 | `src-tauri/migrations/`、`infrastructure/repositories.rs` | 禁止虚构表名；更新 handoff 清单 |
| 新 Tauri 命令 | `src-tauri/src/api/*.rs` + `lib.rs` `generate_handler!` | 前端 `tauri-api.ts` camelCase 键、DTO `reply` 字段 |

## PR 流程

1. **Fork / 功能分支**，一条 PR 聚焦一类变更；契约（manifest、DTO、`PLUGIN_V1`）变更需 **同步文档** 与 **`crates/oclive_validation`**（若适用）。
2. **描述**：说明动机、行为变化、风险与手动验证步骤；关联 issue（若有）。
3. **自检**：至少 **`npm run check`**；触及持久化 / HTTP / 编排时建议 **`npm run check:release`**；内核工程可加 **`cargo run -p oclive-cli -- test -o . --json`**。
4. **审阅**：由 **模块负责人**（上表）或受邀维护者 Review；关注 CI、安全、i18n 与契约文档是否同步。
5. **合并条件**：CI 相关 job 绿（或已知 `continue-on-error` 项已登记）；Breaking 变更走 [`BREAKING_CHANGE_PROCESS.md`](handoff/BREAKING_CHANGE_PROCESS.md)；无未解决的 **P0** 发版阻塞项。

### CI 失败时怎么处理

| Job / 症状 | 建议步骤 |
|------------|----------|
| `cargo fmt` | 本地 `cargo fmt --all` 后重提 |
| `cargo clippy` | `cargo clippy --workspace --all-targets --all-features -- -D warnings` |
| `cargo test`（Windows 集成） | 以 **Ubuntu CI** 为准；本机可 `cargo test --workspace --lib` |
| `frontend` / Vitest | `npm run test:unit` |
| `oocp-test-suite` | 确认 `OCLIVE_HTTP_API_MOCK_LLM=1`、端口空闲；见 [OOCP_TEST_SUITE.md](creator-docs/testing/OOCP_TEST_SUITE.md) |
| `cargo-audit` | 跟踪 [KNOWN_VULNERABILITIES.md](creator-docs/security/KNOWN_VULNERABILITIES.md)；不挡合并 |
| 契约 / 角色包 | `cargo run -p oclive-cli -- pack validate <role>` |

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
