# 端到端性能验收

固定路径：冷启动→可输入、切角色、首条消息、长会话滚动、插件管理/调试。**Release** 用 `tauri build` 产物验收；dev 仅对照。

后端计时基线：[`13_PERF_BASELINE_2026-04-01.md`](./13_PERF_BASELINE_2026-04-01.md)、[`12_BACKEND_PERF_RUNBOOK.md`](./12_BACKEND_PERF_RUNBOOK.md)。前端 chunk：[`FRONTEND_CHUNK_OPTIMIZATION.md`](./FRONTEND_CHUNK_OPTIMIZATION.md)。

**前端体积对照（Vite 生产构建，约 2026-05）**：主入口 `index-*.js` 约 **224 kB / gzip ~68 kB**（locale 文案按需 chunk，见 `FRONTEND_CHUNK_OPTIMIZATION.md`）；插件/市场/设置/调试及场景相关壳多为 **async chunk + `v-if`**。进一步分析：`npm run build:analyze`，打开 **`dist/stats.html`**（`dist/` 已 `.gitignore`，勿提交）。

**Windows 提示**：多文件 `git add` 在 PowerShell 中勿用 Bash 式反斜杠续行；用**单行**或 PowerShell 反引号续行，否则 `git add` 易失败。
