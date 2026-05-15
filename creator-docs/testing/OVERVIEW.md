# 测试体系归属（主仓 vs 编写器）

本文档固定「测什么、在哪个仓库跑」，避免与主仓 `README` / `AGENTS` 叙述冲突。

## 协议与内核（主仓 `oclivenewnew`）

| 层级 | 内容 | 位置 / 命令 |
|------|------|----------------|
| Rust 单元与集成测试 | 编排、`--api` HTTP 路由、`process_message`、**`invoke` 热路径（9 条 `*_impl`）**（[`invoke_hotpath_matrix.rs`](../../src-tauri/tests/invoke_hotpath_matrix.rs)，对照 [`handoff/INVOKE_HOTPATH_MATRIX.md`](../../handoff/INVOKE_HOTPATH_MATRIX.md)）等 | `src-tauri/` 下 `cargo test`；集成测在 `src-tauri/tests/` |
| OOCP 对齐 HTTP 黑盒 | 场景 **S0–S11**（见 [`OOCP_TEST_SUITE.md`](./OOCP_TEST_SUITE.md)） | `examples/oocp-test-suite/run.mjs`；CI job **`oocp-test-suite`**；另跑 **`scripts/e2e-core-api-restart.mjs`**（进程重启烟测，**A1.1a**） |
| 前端烟测 | Vitest 守门 + **`vite preview` + Playwright** 首屏（**A1.1b**；**CI 仅 Ubuntu `frontend`**） | `npm run test:unit`；`npm run build && npm run test:e2e:preview`（[`e2e/preview-shell.spec.ts`](../../e2e/preview-shell.spec.ts)；见 CONTRIBUTING **Windows** 说明） |

## 组件与插件壳（编写器 `oclive-pack-editor`）

| 范围 | 说明 |
|------|------|
| **T05–T13**（Vue 组件测试等） | 权威来源在编写器仓库；主仓不复制 42 条用例树。 |
| **T14–T20**（`official-vue-test-runner` 等） | 编写器内置能力，以**目录插件**范式对接工作区；详见编写器文档与插件 README。 |

主应用通过包格式与 HTTP/`invoke` 契约对接；组件级与插件壳级测试在编写器侧执行即可覆盖创作者工具链。

---

[English](../../creator-docs-en/testing/OVERVIEW.md)
