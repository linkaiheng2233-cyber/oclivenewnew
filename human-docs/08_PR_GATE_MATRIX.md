# PR 门禁矩阵（本地自检）

> 完整流程见 [CONTRIBUTING.md](../CONTRIBUTING.md) · [English summary](../CONTRIBUTING.en.md#tests-before-merge)

在打开 PR 前，按改动类型选择命令。**CI 红 = 合并阻塞**（除标注 `continue-on-error` 的 job）。

| 改动类型 | 必跑（本地） | 可选 / 条件 |
|----------|--------------|-------------|
| **仅文档** (`*.md`, `human-docs/`, `creator-docs/`) | 链接抽查；若动 `CHANGELOG.md` → `node scripts/check-changelog-parity.mjs` | — |
| **仅前端** (`src/`, `e2e/`, `vite.config.ts`) | `npm run test:unit` · `npm run build` | Linux/macOS：`npm run test:e2e:preview` |
| **内核 / 编排** (`crates/oclive_kernel_host/`, `process_message`) | `npm run check` · 触及 HTTP/持久化 → `npm run check:release` | `node scripts/dimension5-acceptance.mjs --ci` |
| **Tauri API** (`src-tauri/src/api/`) | `cargo test -p oclivenewnew-tauri` · `npm run check` | `npm run test:e2e:core-api-restart` |
| **Cargo.lock / 依赖** | `node scripts/dimension5-acceptance.mjs --ci` | `cargo audit` |
| **发版 / 契约** | `npm run check:release` | OOCP：`examples/oocp-test-suite/run.mjs` |

## 快速组合

```bash
# 日常（与 CI frontend + rust 子集对齐）
npm run check

# 发版或改引擎
npm run check:release

# 本地 CI 子集（dimension5 九检，需已 build）
npm run check:ci-local
```

## CI job 对照

| GitHub Actions job | 主要命令 |
|--------------------|----------|
| `rust` | `npm run build` · fmt · clippy · `cargo test --workspace` |
| `frontend` | `npm run test:unit` · `npm run build` ·（Ubuntu）Playwright |
| `oocp-test-suite` | `oclivenewnew-tauri --api` · `node run.mjs` |
| `dimension5-acceptance` | `node scripts/dimension5-acceptance.mjs --ci` |
| `layering-ratchet` | layering 脚本 |

Windows 开发者：Playwright 见 [10_SETUP_WINDOWS.md](10_SETUP_WINDOWS.md) 与 CONTRIBUTING「Web 预览壳 E2E」。
