# 🔍 工作台项目优化分析报告

> 生成时间：2026-05-23  
> 覆盖范围：5 个工作台项目（oclivenewnew、oclive-pack-editor、oclive-launcher、oclive-plugin-market、oclive doll core）

---

## 📎 与 Cursor / Agent 优化文档对接

本报告为 **五仓工程卫生与依赖** 专项扫描（24 项）；与主仓既有 **Cursor 优化轮 / 技术债 / 产品缺口** 文档互补，执行时请先查下表避免重复立项。

| 本报告角色 | 路径 |
|---|---|
| **本文（五仓 24 项 + 分阶段计划）** | [`docs/WORKSPACE_OPTIMIZATION_REPORT.md`](./WORKSPACE_OPTIMIZATION_REPORT.md) |
| **Agent 总入口** | [`AGENTS.md`](../AGENTS.md)（性能、Breaking、测试、蓝图边界） |
| **文档索引 · 工程纪律** | [`creator-docs/getting-started/DOCUMENTATION_INDEX.md`](../creator-docs/getting-started/DOCUMENTATION_INDEX.md) §工程纪律 |
| **Cursor 规则（主仓编排约束）** | [`.cursor/rules/oclivenewnew.mdc`](../.cursor/rules/oclivenewnew.mdc) |

| 既有 handoff 文档 | 与本报告关系 |
|---|---|
| [`handoff/TECHNICAL_DEBT_INVENTORY.md`](../handoff/TECHNICAL_DEBT_INVENTORY.md) | 中长期技术债（Monolith/library、多模态、OTA、市场 UGC 等）；**不重复**本文 gitignore / 二进制 / .env 类条目 |
| [`handoff/ARCHITECTURE_LAYERING.md`](../handoff/ARCHITECTURE_LAYERING.md) §「Cursor 优化轮（2026-05-20）」 | 主仓 **已落地** 的 Cursor 轮次（sqlx、deny、bench 拆分、lint 等）；本文 §1 中 Cargo release / ESLint 等为 **新一轮** 候选 |
| [`handoff/20_SESSION_OPTIMIZATION_REPORT.md`](../handoff/20_SESSION_OPTIMIZATION_REPORT.md) | 主仓 **运行时/契约** 可维护性（`chat_turn`、`bot_emotion` 等）；与本文前端 TS 迁移无重叠 |
| [`handoff/PRODUCT_AND_KERNEL_GAP_CHECKLIST.md`](../handoff/PRODUCT_AND_KERNEL_GAP_CHECKLIST.md) | 产品 P0/P1（测试闸门、首装、Sentry）；本文 launcher/market **缺测试** 可挂 A1 子项 |
| [`handoff/PERF_PHASES.md`](../handoff/PERF_PHASES.md) · [`PERFORMANCE_BASELINE_ACCEPTANCE.md`](../handoff/PERFORMANCE_BASELINE_ACCEPTANCE.md) | 主仓 **性能/包体** 已收尾阶段；本文 §1 Cargo `codegen-units`/`strip` 属 release 微调 |
| [`handoff/FRONTEND_CHUNK_OPTIMIZATION.md`](../handoff/FRONTEND_CHUNK_OPTIMIZATION.md) | 主仓前端分包；本文 §1「主前端仍 JS」为 **类型安全** 迁移，非 chunk 专项 |
| [`handoff/I18N_FOUR_REPO_BASELINE.md`](../handoff/I18N_FOUR_REPO_BASELINE.md) | 四仓 i18n 基线；姊妹仓 `.env.example` 缺口见本文 §跨项目 |
| [`handoff/DUAL_CORE_CURSOR_HANDOFF.md`](../handoff/DUAL_CORE_CURSOR_HANDOFF.md) | 双核蓝图 Cursor 对齐；**不替代** v2 角色包/编写器卫生项 |

### 24 项 → 建议跟踪文档（执行时勾选）

| ID | 本报告条目 | 建议写入/跟踪 |
|---|---|---|
| 1.1–1.3 | oclivenewnew 练习文件 / gitignore / logs | 本文阶段 2；可选 PR 模板「工程卫生」 |
| 1.4–1.7 | TS 迁移 / 依赖 / ESLint / Cargo release | TS → 新 mini-RFC 或 `PRODUCT_LINE_TASK_BUCKETS`；Cargo → [`RUST_RELEASE_AND_DEPENDENCIES.md`](../handoff/RUST_RELEASE_AND_DEPENDENCIES.md) |
| 2.1–2.5 | pack-editor check / tsconfig / start.bat / .env | 编写器 [`AGENTS.md`](../../oclive-pack-editor/AGENTS.md)；2.1 **已过时**（`npm run test` 存在，见下） |
| 3.1–3.4 | launcher OllamaSetup / 无测试 / check 拆分 | [`PRODUCT_AND_KERNEL_GAP_CHECKLIST`](../handoff/PRODUCT_AND_KERNEL_GAP_CHECKLIST.md) A1；启动器 CONTRIBUTING |
| 4.1–4.5 | market .env 泄露 / gitignore / 依赖 / 测试 | **立即** 安全；[`KNOWN_VULNERABILITIES`](../creator-docs/security/KNOWN_VULNERABILITIES.md) 若升级 deps |
| 5.1–5.3 | doll core 二进制 / gitignore / scripts | doll core [`README.md`](../../oclive%20doll%20core/README.md) + Releases 分发策略 |

> **勘误（2026-05-23）**：§2 pack-editor「check 引用 test:unit 不存在」——当前 `package.json` 同时有 `"test"` 与 `"test:unit"`（均指向 `vitest run`），`npm run check` 可正常执行。该项可降为低优先级或关闭。

---

## 1. oclivenewnew（主项目 · Tauri + Vue 3 + Rust）

### 🔴 高优先级

| 问题 | 详情 |
|---|---|
| 练习文件残留 | 根目录 `练习` 文件（118 行 Rust 代码）是练习/测试文件，不应出现在主项目中。建议移入 `crates/` 下的示例或直接删除。 |
| .gitignore 不完整 | 缺少 `dist/`、`test-results/`、`playwright-report/` 等目录。`dist/` 目录实际存在（含构建产物），但未被忽略，容易误提交。 |
| 日志文件被 git 追踪 | `logs/dev-start.log` 已在版本控制中。日志文件不应提交到仓库。需执行 `git rm --cached logs/dev-start.log` 并确认 `.gitignore` 中 `logs` 规则生效。 |

### 🟡 中优先级

| 问题 | 详情 |
|---|---|
| 前端未使用 TypeScript | `vite.config.js` 使用 `.js`，`src/main.js` 使用 `.js`。其他 4 个仓库均已迁移至 `.ts`，主仓应跟进以获得类型安全和 IDE 提示。 |
| 依赖版本策略不一致 | `@tauri-apps/api: "1.5.6"`（精确版本）vs `oclive-launcher` 使用 `"^1.5.6"`。建议主仓也改用 `^` 前缀以自动接收补丁更新。 |
| 缺少 ESLint/Prettier | 前端无代码检查/格式化配置。与 `oclive-pack-editor` 使用 `vue-tsc` 的做法形成对比。建议加入 `eslint` + `@antfu/eslint-config` 或 `prettier`。 |
| Cargo release profile 可优化 | `[profile.release]` 已有 `opt-level = "z"` + `lto = true`，可再加 `codegen-units = 1` 和 `strip = true` 进一步减小体积。 |

### 🟢 低优先级

| 问题 | 详情 |
|---|---|
| .env.example 过于精简 | 仅文档化了 `VITE_SENTRY_DSN`。其他常用变量如 `TAURI_DEV_HOST`、`OCLIVE_HTTP_API_MOCK_LLM`、`OCLIVE_SKIP_STARTUP_HEALTH` 等均未列出。 |
| 脚本文件类型混杂 | `scripts/` 下有 `.mjs`、`.cjs`、`.py`、`.sh` 混合。可按类型分子目录或统一为单一运行时。 |
| oclive-pack-editor.code-workspace | 根目录存在跨仓库的工作区文件，需确认是否应保留或加入 `.gitignore`。 |

---

## 2. oclive-pack-editor（角色包编写器 · Tauri + Vue 3 + TS）

### 🔴 高优先级

| 问题 | 详情 |
|---|---|
| check 脚本引用不存在的命令 | ~~`package.json` 的 `"check"` 中 `"npm run test"` 不存在~~ **已勘误**：`test` 与 `test:unit` 均已存在且指向 `vitest run`；`npm run check` 可执行。可保留为「check 是否应合并/去重 test 与 test:unit」低优先级项。 |
| tsconfig 碎片化 | 存在 `tsconfig.json`、`tsconfig.app.json`、`tsconfig.node.json` 三个配置文件。可评估是否可通过 `references` + `extends` 合并简化为 2 个以内。 |

### 🟡 中优先级

| 问题 | 详情 |
|---|---|
| start.bat 在根目录 | Windows 批处理文件直接放根目录，应统一移入 `scripts/` 目录，或在 AGENTS.md 中提供跨平台替代方案。 |
| 缺少 .env.example | 无环境变量文档模板，开发者不清楚有哪些可配置项（如端口、后端地址等）。建议新增。 |
| Vite 未配置 clearScreen: false | 与 `oclivenewnew` 和 `oclive-launcher` 不一致。Vite 启动时会清屏，丢失之前的终端输出。 |

---

## 3. oclive-launcher（启动器 · Tauri + Vue 3 + TS）

### 🔴 高优先级

| 问题 | 详情 |
|---|---|
| OllamaSetup.exe 在根目录 | 大型安装程序二进制直接放根目录。`.gitignore` 虽忽略了 `src-tauri/bundled/ollama/OllamaSetup.exe`，但根目录的 `/OllamaSetup.exe` 行未被匹配。应确认文件是否仍在 git 追踪中，并从仓库移除。 |
| 完全缺少测试 | `package.json` 无 `vitest`、`playwright` 等任何测试依赖和脚本。作为用户入口的启动器，缺乏质量保障会有风险。 |

### 🟡 中优先级

| 问题 | 详情 |
|---|---|
| start.bat 在根目录 | 同上，建议移入 `scripts/`。 |
| 缺少 .env.example | 无环境变量文档模板。建议新增。 |
| check 脚本无单独粒度 | `"check"` 一行执行 build + fmt + clippy + test 全部，建议拆分为独立脚本（如 `check:fmt`、`check:clippy`、`check:test`），便于 CI 并行和本地调试。 |

---

## 4. oclive-plugin-market（插件市场站 · Vue 3 + TS + Vite）

### 🔴 高优先级（安全问题）

| 问题 | 详情 |
|---|---|
| .env 文件被提交到 git | ⚠️ `.env` 文件含 `VITE_PLUGIN_INDEX_URL`，虽非敏感信息，但按惯例 `.env` 不应进仓库，且可能误操作导致未来泄露敏感值。应加入 `.gitignore` 并执行 `git rm --cached`。 |
| .env.local 被提交到 git | ⚠️ `.env.local` 通常用于本地敏感配置（Supabase Key 等），已被提交。应立即审查内容、从 git 移除，并加入 `.gitignore`。 |
| .gitignore 严重不完整 | 仅有 4 行（`node_modules`、`dist`、`.DS_Store`、`*.local`）。缺 `dist-ssr`、`.env`、`.env.local`、`*.log`、编辑器目录等标准规则。 |

### 🟡 中优先级

| 问题 | 详情 |
|---|---|
| 依赖版本严重滞后 | `typescript: ~5.7.2` vs launcher 的 `~6.0.2`，`vite: ^6.0.3` vs launcher 的 `^8.0.4`。落后一个大版本，建议升级。 |
| 缺少测试配置 | 无 `vitest`、`playwright` 等测试依赖和脚本。最低限度应加入 `vitest` 做组件测试。 |

---

## 5. oclive doll core（Doll 内核 · 独立发行）

### 🔴 高优先级

| 问题 | 详情 |
|---|---|
| 二进制文件直接提交 | ⚠️ `oclive_kernel_server.exe`（Windows 可执行文件）和 `doll-kernel-v1.0.0.zip`（压缩包）直接存入 git。这会显著增加仓库体积和历史不可逆。应使用 Git LFS 或仅通过 GitHub Releases 分发。 |
| .gitignore 规则不足 | 应忽略 `*.exe`、`*.zip`、`dist/` 等构建产物，避免未来误提交。 |
| 脚本无目录组织 | `pack.ps1` 和 `pack.sh` 直接放根目录。建议建立 `scripts/` 统一管理构建脚本。 |

---

## 📊 跨项目共性统计

| 共性 | 涉及项目 |
|---|---|
| 版本策略不一致（精确 vs `^`） | oclivenewnew, oclive-launcher, oclive-pack-editor |
| .env.example 覆盖率低（仅 2/5） | oclive-pack-editor, oclive-launcher, oclive doll core |
| 测试覆盖不均（2 项目完全无测试） | oclive-launcher, oclive-plugin-market |
| TypeScript 迁移不完整 | oclivenewnew（主前端仍为 JS） |
| start.bat 散落根目录 | oclive-pack-editor, oclive-launcher |
| .gitignore 不规范 | oclive-plugin-market（4 行）, oclive doll core（无关键规则） |

---

## ✅ 建议处理顺序

1. **立即处理**：`oclive-plugin-market` 的 `.env`/`.env.local` 清理、`oclive doll core` 的二进制移除
2. **本周内**：`oclivenewnew` 的 `练习` 文件 + `logs/` 清理、`.gitignore` 补全
3. **本月内**：`oclivenewnew` TypeScript 迁移、`oclive-launcher` 和 `oclive-plugin-market` 测试体系搭建
4. **持续改善**：依赖版本统一、环境变量文档化、脚本目录规范化