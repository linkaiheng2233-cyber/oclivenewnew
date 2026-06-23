# 目录重组迁移路径清单

> **生成**：`node scripts/check-stale-paths.mjs`（Phase 0 基线）+ 手工补充 `kernel/` / `distros/` 迁移项。  
> **用途**：PR 合并前对照；`check-stale-paths.mjs` 扩展后 CI ratchet 禁止旧路径回潮。

## 1. `check-stale-paths` 基线命中（2026-06-24）

| 文件 | 问题 |
|------|------|
| `CONTRIBUTING.md` | 禁止别名 `memory_backend` |
| `creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md` | 禁止别名 `memory_backend` |
| `creator-docs/role-pack/PACK_VERSIONING.md` | 禁止别名 `memory_backend` |
| `handoff/COMMENT_ENGLISH_MIGRATION_PLAN.md` | 遗留路径 `src-tauri/src/domain` |

## 2. Rust / Cargo 路径（Phase 1）

| 旧路径 | 新路径 |
|--------|--------|
| `crates/*` | `kernel/crates/*` |
| `fuzz/` | `kernel/fuzz/` |
| `path = "../crates/..."`（desktop-tauri） | `path = "../../kernel/crates/..."` |
| 根 `Cargo.toml` members `crates/...` | `kernel/crates/...` |
| 根 `Cargo.toml` members `fuzz` | `kernel/fuzz` |
| `sqlx = { path = "crates/oclive_sqlx" }` | `kernel/crates/oclive_sqlx` |

## 3. Tauri / 打包（Phase 3）

| 旧路径 | 新路径 |
|--------|--------|
| `src-tauri/` | `distros/desktop-tauri/` |
| `../dist`（tauri distDir） | `../chat-pro/dist` 或 `../theater/dist`（`OCLIVE_TAURI_SHELL`） |
| `../roles`（bundle resources） | `../chat-pro/roles`；剧场经 `filter-theater-roles.mjs` → `resources/roles` |
| `scripts/bundle-kernel-for-tauri.mjs` dest | `distros/desktop-tauri/resources/` |

## 4. 前端 / npm workspaces（Phase 2）

| 旧路径 | 新路径 |
|--------|--------|
| 根 `src/`（共享） | `distros/shared/src/` |
| 根 `src/shells/tool|fluent` | `distros/chat-pro/src/shells/` |
| 根 `src/shells/theater` | `distros/theater/src/shells/theater/` |
| 根 `src/composables/theater` | `distros/theater/src/composables/theater/` |
| 根 `public/theater` | `distros/theater/public/theater/` |
| 根 `vite.config.ts` | `distros/chat-pro/vite.config.ts` + `distros/theater/vite.config.ts` |
| 根 `roles/` | `distros/chat-pro/roles/` |
| 根 `plugins/` | `distros/chat-pro/plugins/` |
| 根 `e2e/` | `distros/chat-pro/e2e/` |

## 5. CI / 文档（Phase 4）— **Done（2026-06-24）**

| 旧引用 | 新引用 |
|--------|--------|
| `crates/README.md` | `kernel/crates/README.md`（根留 redirect） |
| CI `working-directory: src-tauri` | `distros/desktop-tauri` |
| `AGENTS.md` / `.cursor/rules` 编排路径 | `kernel/crates/oclive_kernel_host/...` |
| normative docs 机械迁移 | `node scripts/migrate-doc-paths.mjs`（194 文件） |
| `check-stale-paths.mjs` | 硬门禁 + dimension5 第 11 检 |

## 6. CI ratchet（迁移完成后禁止）

迁移合入后，以下路径**不得**再出现在 normative docs / 新脚本中（`handoff/archive/**`、本清单、CHANGELOG 历史行除外）：

- 根 `crates/`、`fuzz/`、`src-tauri/`、根 `src/`（前端）
- `src-tauri/src/domain`（编排已迁至 `kernel/crates/oclive_kernel_host`）

**验收（2026-06-24）**：`node scripts/check-stale-paths.mjs` OK · `node scripts/dimension5-acceptance.mjs --ci` PASS。
