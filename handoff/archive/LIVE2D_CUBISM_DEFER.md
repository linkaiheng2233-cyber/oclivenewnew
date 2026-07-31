# Live2D Cubism · 实装决策（Sprint C）

**状态**：2026-06-12 · **Deferred**（v0.4.x 再评估 bundling）

## 决策

- **Chat Pro / VS Code**：继续使用 PNG/WebP fallback（`CharacterInfo` / catalog path）。
- **AI Theater**：`Live2DStageAdapter.vue` 已接线；Cubism SDK **未 bundled**。
- 当 `performance_directive.kind=live2d` 且模型路径存在时，UI 显示可见提示并 fallback 到 `fallback_image`。

## 约束

| 项 | 说明 |
|----|------|
| SDK 许可 | Live2D Cubism SDK 需单独许可与体积评估 |
| 构建 | 不纳入默认 `npm run build` / CI 阻塞 |
| Theater | `theater.oclive.toml` `[visual_presentation].mode = stage_full` 已同步 bundled profile |

## 解冻条件（v0.4.x）

1. 选定 Web Cubism 4 SDK 或官方 WASM 路径
2. `Live2DStageAdapter` 挂载 `model3.json` + 表情/动作参数
3. optional CI job `theater-visual-smoke` 改为阻塞（当前为 `continue-on-error`）

## 相关文件

- `distros/shared/src/components/visual/Live2DStageAdapter.vue`
- `distros/shared/src/adapters/visual/index.ts`
- `distros/theater/src/shells/theater/TheaterStagePanel.vue`
