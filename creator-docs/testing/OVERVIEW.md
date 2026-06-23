# 测试体系归属（主仓 vs 编写器）

本文档固定「测什么、在哪个仓库跑」，避免与主仓 `README` / `AGENTS` 叙述冲突。

## 协议与内核（主仓 `oclivenewnew`）

| 层级 | 内容 | 位置 / 命令 |
|------|------|----------------|
| Rust 单元与集成测试 | 编排、`--api` HTTP 路由、`process_message`、**`invoke` 热路径（11 条 `*_impl` 烟测）**（[`invoke_hotpath_matrix.rs`](../../distros/desktop-tauri/tests/invoke_hotpath_matrix.rs)，对照 [`handoff/INVOKE_HOTPATH_MATRIX.md`](../../handoff/INVOKE_HOTPATH_MATRIX.md)）、蓝图写盘 [`save_role_slot_registry.rs`](../../distros/desktop-tauri/tests/save_role_slot_registry.rs) 等 | `distros/desktop-tauri/` 下 `cargo test`；集成测在 `distros/desktop-tauri/tests/` |
| OOCP 对齐 HTTP 黑盒 | **13 场景（S0–S12）**；可选 **S13/S14** 双核场景（降级与成功路径，见 [`OOCP_TEST_SUITE.md`](./OOCP_TEST_SUITE.md)） | `examples/oocp-test-suite/run.mjs`；CI job **`oocp-test-suite`**；另跑 **`scripts/e2e-core-api-restart.mjs`**（进程重启烟测，**A1.1a**） |
| 前端烟测 | Vitest 守门 + **`vite preview` + Playwright** 首屏（**A1.1b**；**CI 仅 Ubuntu `frontend`**） | `npm run test:unit`；`npm run build && npm run test:e2e:preview`（[`distros/chat-pro/e2e/preview-shell.spec.ts`](../../distros/chat-pro/e2e/preview-shell.spec.ts)；见 CONTRIBUTING **Windows** 说明） |

### Remote LLM 测试覆盖

| 层级 | 状态 | 位置 |
|------|------|------|
| JSON-RPC 客户端（`RemoteLlmHttp`） | **已覆盖** | [`remote_llm_jsonrpc_roundtrip.rs`](../../distros/desktop-tauri/tests/remote_llm_jsonrpc_roundtrip.rs) |
| 完整 `process_message`（`plugin_backends.llm = remote`） | **已覆盖** | [`remote_llm_process_message_roundtrip.rs`](../../distros/desktop-tauri/tests/remote_llm_process_message_roundtrip.rs) |
| OpenAI-compatible 路径（`OCLIVE_LLM_CLOUD_API_STYLE`） | **未覆盖** | 见 [REMOTE_PLUGIN_PROTOCOL.md](../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md) §测试覆盖 |

## 组件与插件壳（编写器 `oclive-pack-editor`）

| 范围 | 说明 |
|------|------|
| **T05–T13**（Vue 组件测试等） | 权威来源在编写器仓库；主仓不复制 42 条用例树。 |

### T05–T13 覆盖状态（2026-05-20 · T09–T13 补全）

规划约 **42** 条组件/交互用例；**T05–T13 关键路径 Vitest 已全部落地**（编写器 `npm run test`）。

| ID | 范围 | 编写器覆盖（代表文件） | 状态 |
|----|------|------------------------|------|
| **T05** | 试聊 / runtime API 与 Tauri invoke 映射 | `runtimeApiHelpers.test.ts`、`runtimeApiChatParse.test.ts`、`rolePackEditorApi.test.ts` | **已覆盖** |
| **T06** | 专家模型 / 高级创作 | `oclexpertPack.test.ts`、`AnchorPresetManager.spec.ts` | **已覆盖** |
| **T07** | 简单 / 高级视图分级与路由 | `useEditorViewState.test.ts`、`simpleCreation.test.ts` | **已覆盖** |
| **T08** | 角色运行时面板（试聊 / 反馈） | `ChatPanel.spec.ts`、`FeedbackWorkspace.spec.ts` | **已覆盖** |
| **T09** | 模型选择器 / 检查导出 | `HostModelPickRow.spec.ts`、`exportPrepare.test.ts`、`exportPack.test.ts`、`packChecks.test.ts` | **已覆盖** |
| **T10** | 插件后端 / 角色包编辑器 | `PluginManagerPanel.spec.ts`（`RolePackEditorPanel` 插件槽）、`RolePackEditorPanel.spec.ts` | **已覆盖** |
| **T11** | 校验调试面板 | `DebugPanel.spec.ts`（`PackChecksSection` wasm/TS 状态） | **已覆盖** |
| **T12** | 快捷键 / 视图分级 | `HotkeySettingsSection.spec.ts`、`useEditorViewState.test.ts` | **已覆盖** |
| **T13** | 前端测试运行器 + 工具函数 | `FrontendTestRunnerPanel.spec.ts`、`mergeManifest.test.ts`、`uiConfig.test.ts`、`authorPack.test.ts`；Playwright `distros/chat-pro/e2e/smoke.spec.ts` | **已覆盖** |

**合计（编写器 Vitest `it` 数）**：**119**（2026-05-20）；组件 spec **32** 条。

| **T14–T20**（`official-vue-test-runner` 等） | 编写器内置能力，以**目录插件**范式对接工作区；**T14 Vue runner 已入库** `distros/chat-pro/plugins/official-vue-test-runner/`（见插件 README）。 |

主应用通过包格式与 HTTP/`invoke` 契约对接；组件级与插件壳级测试在编写器侧执行即可覆盖创作者工具链。

---

[English](../../creator-docs-en/testing/OVERVIEW.md)
