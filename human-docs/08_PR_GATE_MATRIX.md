# PR 门禁矩阵（本地自检）

> 完整流程见 [CONTRIBUTING.md](../CONTRIBUTING.md) · [English summary](../CONTRIBUTING.en.md#tests-before-merge)

在打开 PR 前，按改动类型选择命令。**CI 红 = 合并阻塞**（除标注 `continue-on-error` 的 job）。

| 改动类型 | 必跑（本地） | 可选 / 条件 |
|----------|--------------|-------------|
| **仅文档** (`*.md`, `human-docs/`, `creator-docs/`) | 链接抽查；若动 `CHANGELOG.md` → `node scripts/check-changelog-parity.mjs` | — |
| **公共 Base / 最小角色开发反馈** | `npm run check:kernel-base`（源码仓库根目录） | 合批验收仍按实际影响选下列门禁；改公开 Rust API另跑 `cargo test --workspace --doc` |
| **仅前端** (`distros/shared/`, `distros/chat-pro/`, `distros/chat-pro/e2e/`, `distros/chat-pro/vite.config.ts`) | `npm run test:unit` · `npm run build` | Linux/macOS：`npm run test:e2e:preview` |
| **内核 / 编排** (`kernel/crates/oclive_kernel_host/`, `process_message`) | `npm run check` · 触及 HTTP/持久化 → `npm run check:release` | `node scripts/dimension5-acceptance.mjs --ci` |
| **Tauri API** (`distros/desktop-tauri/src/api/`) | `cargo test -p oclivenewnew-tauri` · `npm run check` | `npm run test:e2e:core-api-restart` |
| **Cargo.lock / 依赖** | `node scripts/dimension5-acceptance.mjs --ci` | `cargo audit` |
| **发版 / 契约** | `npm run check:release` | OOCP：`examples/oocp-test-suite/run.mjs` |

## 快速组合

在 OCLive 源码根目录运行 `npm run check:kernel-base`，会依次运行现有 Base fixture、最小角色合同测试、两项公共 crate 的 Clippy 和显式 doctest。需要 Node（按 package.json）、Cargo 和 Clippy；不要求先构建前端或准备桌面资源。命令失败即停止并返回非零；首次编译较慢，每步先显示命令，Cargo 输出在该步结束后显示，每步上限20分钟。详细[检查范围及未覆盖边界](../creator-docs/roadmap/SOMEDAY_TOOLCHAIN_CI.md#kernel-host-ci-first-slice)由CI设计维护。

例如修改 Memory 的 Base 调用时，先用这个入口检查 OCLive 的公共契约基线，再运行自己模块的测试。脚手架生成工程仍在其自身目录执行 `cargo check` / `cargo test`；本入口不会自动检查树外模块。消费者接线、Host行为及里程碑验收按实际改动补相应检查；窄测通过不等于完整CI通过。

```bash
# 公共基础契约的日常窄测（OCLive源码仓库）
npm run check:kernel-base

# 日常（与 CI frontend + rust 子集对齐）
npm run check

# 发版或改引擎
npm run check:release

# 本地 CI 子集（dimension5 `--ci`，检查项总数以脚本输出为准，需已 build）
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
