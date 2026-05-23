# 测试体系归属（主仓 vs 编写器）

本文档固定「测什么、在哪个仓库跑」，避免与主仓 `README` / `AGENTS` 叙述冲突。

## 协议与内核（主仓 `oclivenewnew`）

| 层级 | 内容 | 位置 / 命令 |
|------|------|----------------|
| Rust 单元与集成测试 | 编排、`--api` HTTP 路由、`process_message`、**`invoke` 热路径（11 条 `*_impl` 烟测）**（[`invoke_hotpath_matrix.rs`](../../src-tauri/tests/invoke_hotpath_matrix.rs)，对照 [`handoff/INVOKE_HOTPATH_MATRIX.md`](../../handoff/INVOKE_HOTPATH_MATRIX.md)）、蓝图写盘 [`save_role_slot_registry.rs`](../../src-tauri/tests/save_role_slot_registry.rs) 等 | `src-tauri/` 下 `cargo test`；集成测在 `src-tauri/tests/` |
| OOCP 对齐 HTTP 黑盒 | **13 场景（S0–S12）**；可选 **S13** 双核降级（见 [`OOCP_TEST_SUITE.md`](./OOCP_TEST_SUITE.md)） | `examples/oocp-test-suite/run.mjs`；CI job **`oocp-test-suite`**；另跑 **`scripts/e2e-core-api-restart.mjs`**（进程重启烟测，**A1.1a**） |
| 前端烟测 | Vitest 守门 + **`vite preview` + Playwright** 首屏（**A1.1b**；**CI 仅 Ubuntu `frontend`**） | `npm run test:unit`；`npm run build && npm run test:e2e:preview`（[`e2e/preview-shell.spec.ts`](../../e2e/preview-shell.spec.ts)；见 CONTRIBUTING **Windows** 说明） |

## 组件与插件壳（编写器 `oclive-pack-editor`）

| 范围 | 说明 |
|------|------|
| **T05–T13**（Vue 组件测试等） | 权威来源在编写器仓库；主仓不复制 42 条用例树。 |

### T05–T13 覆盖状态（2026-05-20 · 第四批分批）

规划约 **42** 条组件/交互用例；当前以 **关键路径 Vitest** 分批落地（编写器 `npm run test`）。

| ID | 范围 | 编写器覆盖（代表文件） | 状态 |
|----|------|------------------------|------|
| **T05** | 试聊 / runtime API 与 Tauri invoke 映射 | `runtimeApiHelpers.test.ts`、`runtimeApiChatParse.test.ts`、`rolePackEditorApi.test.ts` | **已覆盖（核心）** |
| **T06** | 专家模型 / 高级创作 | `oclexpertPack.test.ts`、`AnchorPresetManager.spec.ts` | **已覆盖（核心）** |
| **T07** | 简单 / 高级视图分级与路由 | `useEditorViewState.test.ts`、`simpleCreation.test.ts` | **已覆盖（核心）** |
| **T08** | 角色运行时面板（试聊 / 反馈） | `ChatPanel.spec.ts`、`FeedbackWorkspace.spec.ts` | **已覆盖（核心）** |
| **T09** | 检查与导出工作流 | `exportPrepare.test.ts`、`exportPack.test.ts`、`packChecks.test.ts` | **部分** |
| **T10** | 知识库 / 世界观 | `knowledgeFrontMatter.test.ts`、`knowledgeHitPreview.test.ts` | **部分** |
| **T11** | 市场 compose 导入 | `marketComposeImport.test.ts` | **部分** |
| **T12** | 角色包编辑器面板 | `RolePackEditorPanel.spec.ts` | **部分** |
| **T13** | 前端测试运行器 UI | Playwright `e2e/smoke.spec.ts` + 主仓 `official-vue-test-runner` 插件 | **部分** |

**合计（编写器 Vitest `it` 数）**：**87**（2026-05-20）；组件 spec **20+** 条。余下 T09–T13 深交互随 studio UX 稳定继续补。

| **T14–T20**（`official-vue-test-runner` 等） | 编写器内置能力，以**目录插件**范式对接工作区；**T14 Vue runner 已入库** `plugins/official-vue-test-runner/`（见插件 README）。 |

主应用通过包格式与 HTTP/`invoke` 契约对接；组件级与插件壳级测试在编写器侧执行即可覆盖创作者工具链。

---

[English](../../creator-docs-en/testing/OVERVIEW.md)
