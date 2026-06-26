# 模块开工包 · `<模块名>`

> **读者**：`<谁>`  
> **读完能做什么**：`<能独立完成什么>`  
> **耗时**：约 `<N>` min  
> **SSOT 范围**：人类开工 checklist；定义见 [MODULE_MAP §N](../../handoff/MODULE_MAP_AND_HANDOFF.md)  
> **最后更新**：2026-06-26  
> **下一篇**：`<链到相关包或 L6/L7>`

---

## 1. 你插在哪

- **MODULE_MAP**：[§N `<模块>`](../../handoff/MODULE_MAP_AND_HANDOFF.md)  
- **`plugin_backends` 键**（若适用）：`<键名>`  
- **主链 hook**：`<文件名>`（编排入口仍 [`process_message.rs`](../../kernel/crates/oclive_kernel_host/src/domain/chat_engine/process_message.rs)）

---

## 2. 边界

| 能改 | 禁止 |
|------|------|
| `<目录/文件>` | `<G1/G6 等对齐项>` |

---

## 3. 阅读清单（3–5 链）

1. [MODULE_MAP §N](../../handoff/MODULE_MAP_AND_HANDOFF.md)  
2. `<creator-docs 链>`  
3. `<handoff 链>`  
4. `<可选 BUS_FACTOR 锚点>`

---

## 4. 开发流程

- [ ] `<步骤 1>`  
- [ ] `<步骤 2>`  
- [ ] `npm run check` 或槽位相关测试绿

---

## 5. 验收

- [ ] `<checkbox 1>`  
- [ ] `<checkbox 2>`  
- [ ] 未复制 MODULE_MAP / PLUGIN_V1 大表进 PR 描述

---

## 6. 联调依赖

| 相关模块 | 数据关系 |
|----------|----------|
| `<槽/设施>` | `<一句话>` |
