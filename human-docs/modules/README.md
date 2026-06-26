# 模块开工包 · 选择器

> **读者**：已完成 L0–L3（约 90 min）、要改**单一模块**而非整条主链的工程师。  
> **读完能做什么**：按 MODULE_MAP 四大类选对开工包，避免默认误读 L5 内核路径。  
> **耗时**：本页约 **30 秒**；单包约 **30–60 min**。  
> **SSOT 范围**：人类开工路由；**模块定义**仍只维护于 [MODULE_MAP §2–§12](../../handoff/MODULE_MAP_AND_HANDOFF.md)。  
> **最后更新**：2026-06-26  
> **下一篇**：下表对应目录中的一份 `.md`。

---

## 怎么用

1. 读完 [00–04](../README.md#学习阶梯) 通用层（L0–L3）。  
2. 在下表选 **一个** 与你任务最贴近的包。  
3. 包内 §3 链到 creator-docs / handoff SSOT；**禁止**在包内复制六槽表或 PLUGIN_V1 全文。  
4. 若你要改 `process_message` 编排顺序 → 走 [06 内核学习路径](../06_KERNEL_LEARNING_PATH.md)（**内核主链维护者专用**）。

模板：[`_TEMPLATE.md`](_TEMPLATE.md)

---

## 按 MODULE_MAP 四大类

| 大类 | 包目录 | 开工包 | 状态 |
|------|--------|--------|------|
| **六槽** | [`slots/`](slots/) | memory · emotion · event · prompt · llm · agent | **Partial**（llm · agent · memory Done） |
| **设施** | [`facilities/`](facilities/) | complex-emotion · portrait · visual-stage | OPEN |
| **独立通道** | [`side-channels/`](side-channels/) | user-identity · reply-post-process · chat-storage | OPEN |
| **编排行** | [`orchestration/`](orchestration/) | turn-thinking · model-tier（摘要链） | OPEN |
| **角色包** | [`packs/`](packs/) | role-pack-content · role-pack-config | OPEN |
| **宿主面** | [`surfaces/`](surfaces/) | frontend-chat-pro · tauri-invoke · distro-hostprofile | OPEN |

---

## 按角色快捷入口

| 你是谁 | 路径 |
|--------|------|
| 插件 / LLM 后端作者 | [paths/plugin-author.md](../paths/plugin-author.md) → `slots/llm` 或 `slots/agent` |
| 角色包文案作者 | L0–L2 → [`packs/role-pack-content.md`](packs/role-pack-content.md) |
| Chat Pro 前端 | [paths/frontend.md](../paths/frontend.md) → `surfaces/` |
| 集成方 / 无头 HTTP | [paths/integrator.md](../paths/integrator.md) → `surfaces/distro-hostprofile` |
| 内核主链维护者 | L5 [06](../06_KERNEL_LEARNING_PATH.md) + MODULE_MAP 深读 |

---

*进度跟踪：[human-docs/README §H-DOC-04](../README.md#文档包进度与-ai-包同步--2026-06-26)*
