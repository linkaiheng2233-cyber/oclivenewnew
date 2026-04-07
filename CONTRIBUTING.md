# 贡献指南

感谢考虑为 oclive 做贡献。项目目标见 [creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md](creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md)。

## GitHub 仓库（CI、Dependabot、分支保护）

合并默认分支后，**Dependabot** 会按 [`.github/dependabot.yml`](.github/dependabot.yml) 开依赖更新 PR；**CI** 见 Actions。若你维护组织/仓库设置（分支保护、Secrets 等），见 **[creator-docs/getting-started/GITHUB_REPO_CHECKLIST.md](creator-docs/getting-started/GITHUB_REPO_CHECKLIST.md)**。

## 开发流程

1. Fork / 分支开发，尽量**小步 PR**，一条 PR 解决一类问题。
2. 修改契约（manifest、`PLUGIN_V1`、DTO）时**同步文档**与校验逻辑。
3. 提交前在本地运行（与 CI 尽量对齐）：
   - **日常 PR**：`npm run check`（`vite build` + **`cargo fmt` / `clippy` / `cargo test --lib`**）
   - **发版或改引擎契约前**：`npm run check:release`（同上，且 **`cargo test` 含 `tests/` 集成与 doc-tests**），与 README「发版门槛」一致。
   - 分步：`npm run check:rust:fmt`、`check:rust:clippy`、`check:rust:test`，或 `cd src-tauri` 后手写同等命令。
   - **CI**：GitHub Actions 在 **Ubuntu 与 Windows** 上均跑 `cargo fmt` / `clippy` / `cargo test` 与 `npm run build`（见 `.github/workflows/ci.yml`）。
   - Windows 若链接报 **LNK1104**，多为 `target\\debug\\*.exe` 被占用或并行链接冲突；关闭占用进程后重试，或单线程：`cd src-tauri && cargo test -j 1`。

## 代码风格

- **Rust**：与现有模块一致；公共 API 变更需考虑角色包与前端类型。
- **Vue / TS**：与现有 composables、stores 风格一致；`SendMessageResponse` 字段名与 [src-tauri/src/models/dto.rs](src-tauri/src/models/dto.rs) 对齐。

## 不要提交

- 密钥、Token、个人路径；勿将 `.env` 提交入库（见 `.gitignore`）。
- `src-tauri/target`（构建产物）。

## 讨论与路线图

大改动建议先开 issue 或对照路线图中的月份目标，避免与「双软件（运行时 / 编写器）」分工冲突。
