# Chat Pro 垂直功能 · 组员交接包

> **读者**：加入「视觉升级」或「语音识别」两条线的工程师（可不熟悉 OClive）。  
> **维护者**：填写 [CHAT_PRO_VERTICAL_HANDOFF.md](./CHAT_PRO_VERTICAL_HANDOFF.md) 顶部「项目信息表」后转发。  
> **先看边界**：[SCOPE_AND_BOUNDARIES.md](./SCOPE_AND_BOUNDARIES.md) —— **你只需在局部目录开发，不必啃全仓。**

---

## 30 秒分流

| 你是谁 | 打开 | 工作区（只改这些） |
|--------|------|-------------------|
| **视觉 A** | [TRACK_VISUAL_UPGRADE.md](./TRACK_VISUAL_UPGRADE.md) | `src/components/visual/` · Shell · `roles/demo-doll/` |
| **语音 B** | [TRACK_VOICE_RECOGNITION.md](./TRACK_VOICE_RECOGNITION.md) | **`examples/voice-loop-minimal/`** |
| **延迟 / stream UI** | [CHAT_PRO §2.1](./CHAT_PRO_VERTICAL_HANDOFF.md) | **组长** · Chat Pro 打字机（本 sprint 默认） |

共用：[DEV_ENVIRONMENT.md](./DEV_ENVIRONMENT.md) · [SCOPE_AND_BOUNDARIES.md](./SCOPE_AND_BOUNDARIES.md) · [CHAT_PRO_VERTICAL_HANDOFF.md](./CHAT_PRO_VERTICAL_HANDOFF.md)

---

## 文档结构

| 文档 | 谁读 |
|------|------|
| **[SCOPE_AND_BOUNDARIES.md](./SCOPE_AND_BOUNDARIES.md)** | **两人必读** — 解耦三层、目录白名单/黑名单、不必读的 doc |
| **[DEV_ENVIRONMENT.md](./DEV_ENVIRONMENT.md)** | **Day 0** — 按角色裁剪的安装（语音可不装 Node） |
| **[CHAT_PRO_VERTICAL_HANDOFF.md](./CHAT_PRO_VERTICAL_HANDOFF.md)** | 背景、联调、PR 总则 |
| **[TRACK_VISUAL_UPGRADE.md](./TRACK_VISUAL_UPGRADE.md)** | 视觉任务 |
| **[TRACK_VOICE_RECOGNITION.md](./TRACK_VOICE_RECOGNITION.md)** | 语音任务 |

---

## 建议阅读顺序

### 视觉线（约 3 小时）

1. [SCOPE_AND_BOUNDARIES.md §2](./SCOPE_AND_BOUNDARIES.md)  
2. [DEV_ENVIRONMENT.md §3、§9 视觉](./DEV_ENVIRONMENT.md)  
3. [TRACK_VISUAL_UPGRADE.md](./TRACK_VISUAL_UPGRADE.md) Week 1  
4. `HARDWARE_INTEGRATION.md` §5.2（字段表）  

**跳过**：内核学习路径、PLUGIN_V1、语音 example。

### 语音线（约 2 小时）

1. [SCOPE_AND_BOUNDARIES.md §3](./SCOPE_AND_BOUNDARIES.md)  
2. [DEV_ENVIRONMENT.md §2 语音列、§4–§5、§9 语音](./DEV_ENVIRONMENT.md)  
3. [TRACK_VOICE_RECOGNITION.md](./TRACK_VOICE_RECOGNITION.md) Week 1  
4. `HARDWARE_INTEGRATION.md` §4  

**跳过**：`npm run tauri:dev` 章节（除非联调日）、立绘/Live2D 全部文档、整个 `src/`。

---

## 与 human-docs 全站入门的关系

本目录是 **垂直 sprint 窄入口**。不要求完成 [human-docs/README.md](../README.md) 的 L5–L6 内核验收；那是内核贡献者路径。

| 需要 | 不需要 |
|------|--------|
| 理解 HTTP `reply` + `performance_directive` | 读懂 `process_message.rs` |
| 会跑自己的 daily 命令 | `cargo test --workspace` 全绿（除非越界改 crate） |

---

## 深度文档（按需）

| 类型 | 位置 |
|------|------|
| 立绘 RFC | `creator-docs/rfc/RFC_PORTRAIT_FACILITY.md`（视觉 Week 2+） |
| 硬件 SSOT | [HARDWARE_INTEGRATION.md](../../HARDWARE_INTEGRATION.md) |
| AI / 维护者 | [handoff/README.md](../../handoff/README.md) |
