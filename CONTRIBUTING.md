# 贡献指南

感谢考虑为 oclive 做贡献。项目目标见 [creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md](creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md)。

## GitHub 仓库（CI、Dependabot、分支保护）

合并默认分支后，**Dependabot** 会按 [`.github/dependabot.yml`](.github/dependabot.yml) 开依赖更新 PR；**CI** 见 Actions。若你维护组织/仓库设置（分支保护、Secrets 等），见 **[creator-docs/getting-started/GITHUB_REPO_CHECKLIST.md](creator-docs/getting-started/GITHUB_REPO_CHECKLIST.md)**。

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
- **Vue / TypeScript**：与现有 composables、stores 风格一致；与 Tauri 契约字段对齐（如 **`reply`**，见 [`src-tauri/src/models/dto.rs`](src-tauri/src/models/dto.rs)）。

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

**CI 对齐**：**`.github/workflows/ci.yml`** 在 Ubuntu / Windows 上跑 Rust 与 **`npm run build`**、**`npm run test:unit`** 等；Ubuntu 另跑 **OOCP** 与 **`oclive-cli`** 相关 job。详见根目录 [README.md](README.md)「测试与检查」。

## PR 流程

1. **Fork / 功能分支**，一条 PR 聚焦一类变更；契约（manifest、DTO、`PLUGIN_V1`）变更需 **同步文档** 与 **`crates/oclive_validation`**（若适用）。
2. **描述**：说明动机、行为变化、风险与手动验证步骤；关联 issue（若有）。
3. **自检**：至少 **`npm run check`**；触及持久化 / HTTP / 编排时建议 **`npm run check:release`**。
4. **Review**：关注 CI 红绿、安全与本地化文案；大功能建议先开 issue 对照路线图。

## 文档约定

- **用户可见文案**：避免多处硬编码漂移（参见 [AGENTS.md](AGENTS.md) 中插件管理入口说明）。
- **契约与表名**：以 `roles/README_MANIFEST.md`、`RoleStorage::load_role` 及 **`crates/oclive_validation`** 为准；**禁止**虚构数据库表名。
- **创作者文档索引**：[creator-docs/getting-started/DOCUMENTATION_INDEX.md](creator-docs/getting-started/DOCUMENTATION_INDEX.md)。

## 不要提交

- 密钥、Token、个人路径；勿将 `.env` 提交入库（见 `.gitignore`）。
- 若本地仍有历史目录 **`src-tauri/target/`**，可删除；发行 bundle 以外置 **`target-dir`** 下的 **`release/bundle/`** 为准。

## 讨论与路线图

大改动建议先开 issue 或对照路线图中的月份目标，避免与「运行时 / 编写器」分工冲突。
