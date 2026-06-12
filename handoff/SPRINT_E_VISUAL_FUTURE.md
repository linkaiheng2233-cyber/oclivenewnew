# Sprint E · 视觉远期（Phase 6）

**状态**：路线图占位（**不阻塞 v0.4.0**）

## 范围

| 能力 | 落点 | 状态 |
|------|------|------|
| `rig3d` WebGL adapter | `src/adapters/visual/index.ts` | stub |
| `procedural` adapter | 同上 | stub |
| directory `visual_presentation.materialize` | 目录插件契约 + 示例 | 未开始 |
| `context: inner` 双壳 UI | `src/shells/inner/InnerVisualShell.vue` | 占位组件 |

## 契约要点

- **directory**：插件声明 `provides: ["visual_presentation.materialize"]`；宿主 post_llm 仍产出 `visual_state_id`，materialize 可委托插件。
- **inner context**：`portrait_catalog.assets[].context = "inner"` 时，社交壳隐藏该状态，内向叙事壳消费同一 `performance_directive`。
- **rig3d / procedural**：Theater `stage_full` 发行版优先；Chat Pro 默认 `image_only`。

## 参考

- [RFC_VISUAL_PRESENTATION_FACILITY.md](../creator-docs/rfc/RFC_VISUAL_PRESENTATION_FACILITY.md)
- [RFC_PORTRAIT_FACILITY.md](../creator-docs/rfc/RFC_PORTRAIT_FACILITY.md) §catalog `context`
