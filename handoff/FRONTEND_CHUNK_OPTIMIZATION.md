# 前端 chunk 优化

- **`main.js`**：`app.mount` 后 `requestIdleCallback` / `setTimeout` 再动态 `import("@sentry/vue")`；默认 `tracesSampleRate: 0`；可选 `VITE_SENTRY_TRACES_SAMPLE_RATE`（0～1）。
- **`App.vue`**：插件管理 V1/V2、市场 V1/V2、本地模型、设置、调试面板均为 `defineAsyncComponent`，并按 store / 本地 ref 的 **`v-if`** 挂载，首屏不解析这些大块。另将 **`AutonomousSceneNotice`、`RoleDetailView`、`PluginSidebarSlots`、`RoleplayAsidePanel`、场景条与顶栏场景对话框、`ShortcutHelp`、`VirtualTimeBar`、`RolePackBar`、`ImportProgressModal`** 懒加载；`ImportProgressModal` 配合 **`v-if="dropImportOpen"`** 避免常驻解析。
- **基线（`npm run build`，gzip）**：`index-*.js` 约 **365 kB / 118 kB**（随改动浮动）；其余为独立 async chunk。
- **`PluginManagerV2` / `ExpertModelsPanel`**：`defineAsyncComponent` 懒加载专家模型与画布（`@vue-flow`）。
- **`vite.config.js`**：`vendor-vue-flow`、`vendor-i18n`（`vue-i18n`）；不对 `@sentry` 做 `manualChunks`（避免空 chunk）。
- **Monaco**：主应用 `src/` 未使用，已从根 `package.json` 移除 `monaco-editor` 与 `vite-plugin-monaco-editor`（子目录如 `distributions/vscode` 需自管依赖）。

验收：`npm run build`；深度拆包：`npm run build:analyze`（生成 `dist/stats.html`，目录已 gitignore）。
