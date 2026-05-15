# A2 结论汇总（首装、环境、可恢复性 / 离线弱网）

**日期**：2026-05-15（工程收口）

## 已落实

### A2.2 轻量环境自检

- **后端**：`src-tauri/src/api/diagnostics.rs` — `run_environment_diagnostics`（`OLLAMA_BASE_URL` 默认 `http://localhost:11434`，`GET …/api/tags` ~8s；`roles_dir`；`app_data` 写探针）。
- **注册**：`src-tauri/src/lib.rs` `generate_handler!`。
- **前端**：`src/utils/tauri-api.ts`；`src/views/SettingsView.vue`（常规 → 环境自检）；`settings.envCheck*` i18n。

### A2.1 子集（ERROR_CODES §1.5）

- **文档**：`creator-docs/getting-started/ERROR_CODES.md` / `creator-docs-en/.../ERROR_CODES.md` §1.5。
- **invoke 映射**：`src/i18n/locales/fragments/apiErrors.{zh,en}.ts` — `LLM_ERROR` / `IO_ERROR` / `ROLE_NOT_FOUND`。

### A2.3 离线 / 弱网（产品可见 + 文档）

- **既有后端**：`sync_plugin_index_command` 在线失败 → `load_cached_index` + `offlineMode` + `warning`（`src-tauri/src/api/plugin_index.rs`）。
- **本批次前端**：`PluginManagerPanel.vue` — 索引失败时 **横幅与 Toast 走 i18n**（`pluginWorkbench.market.syncFailedTitle` / `syncFailedDetail` / `toastOfflineCache`）；仅缓存无 `warning` 时仍显示 `market.offline`。
- **文档**：§**1.6**（中/英 `ERROR_CODES.md`）；提 issue 环境变量补充 **`OCLIVE_PLUGIN_INDEX_URL`**。

## 未宣称完成（后续里程碑）

- **A2.1（全集）**：全部首装/权限失败分支的统一 code + 映射。
- **A2.3（全集）**：Remote 插件市场站、全产品面统一「网络状态」组件 — 仍以各模块分治 + 文档为准。

## 验收

- `cargo check -p oclivenewnew-tauri`
- `npm run test:unit`
- 手动：断网 → 插件工作台 → 同步在线索引 → 应见中文/英文说明横幅 + 短 Toast，列表来自缓存（若有）。

## 发版清单

`handoff/PRODUCT_RELEASE_CHECKLIST.md`：**A2.1（子集）**、**A2.2**、**A2.3** 已勾选。
