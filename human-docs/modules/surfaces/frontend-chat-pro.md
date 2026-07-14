# 宿主面开工包 · Chat Pro 前端

> **读者**：改 Vue / Pinia / Chat Pro UI 的工程师。  
> **读完能做什么**：在不动内核编排的前提下改聊天界面与状态管理。  
> **耗时**：约 **45 min**  
> **SSOT 范围**：人类 checklist；invoke 契约见 [tauri-invoke](tauri-invoke.md)  
> **最后更新**：2026-07-14
> **下一篇**：[paths/frontend](../../paths/frontend.md) · [08 资料地图](../../08_REFERENCE_MAP.md)

---

## 1. 你插在哪

- **代码根**：`distros/chat-pro/` + `distros/shared/`  
- **发消息链**：`distros/shared/src/api/chat.ts` → `send_message`  
- **状态**：`distros/shared/src/stores/chatStore.ts`  
- **不进**：`process_message.rs` 编排（业务下沉内核）

---

## 2. 边界

| 能改 | 禁止 |
|------|------|
| Vue 组件 · Pinia store · 样式 | 在 UI 内二次调 LLM（立绘/闲聊绕过槽位） |
| `distros/shared/src/api/` camelCase 封装 | 用 `response` 代替 DTO **`reply`** |
| 插件管理面板 · 模型管理面板 | 在 `lib.rs` 堆业务逻辑 |

---

## 3. 阅读清单

1. [paths/frontend](../../paths/frontend.md)  
2. [tauri-invoke](tauri-invoke.md)  
3. [04 工程约束 §3–§4](../../04_ENGINEERING_RULES.md)  
4. [NAMING §8 前端对照](../../../creator-docs/NAMING_CONVENTIONS.md#8-前端--后端术语对照)
5. [facilities/visual-stage](../facilities/visual-stage.md)（视觉线）

---

## 4. 开发流程

- [ ] L2 跑通 `npm run tauri:dev`  
- [ ] 新 UI 功能先查是否已有 shared API  
- [ ] 新 invoke → 链 [tauri-invoke](tauri-invoke.md)，**勿**直写 Rust api  
- [ ] `npm run test:unit` · `npm run build`

---

## 5. 验收

- [ ] invoke 键 camelCase · 响应读 **`reply`**  
- [ ] 未在内核 api 层堆编排  
- [ ] 视觉表现不发起选图 LLM

---

## 6. 联调依赖

| 相关模块 | 数据关系 |
|----------|----------|
| [tauri-invoke](tauri-invoke.md) | IPC 契约 |
| [slots/llm](../slots/llm.md) | 后端生成 reply |
| [facilities/portrait](../facilities/portrait.md) | `visual_state_id` 展示 |
