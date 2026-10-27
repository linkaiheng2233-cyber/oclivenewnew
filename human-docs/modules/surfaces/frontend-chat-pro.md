# 宿主面开工包 · Chat Pro 前端

> **读者**：改 Vue / Pinia / Chat Pro UI 的工程师。  
> **读完能做什么**：在不动内核编排的前提下改聊天界面与状态管理。  
> **耗时**：约 **45 min**  
> **SSOT 范围**：人类 checklist；invoke 契约见 [tauri-invoke](tauri-invoke.md)  
> **最后更新**：2026-10-27
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

首次模型引导复用 `ModelManagerBody` 的连接设置，不附带或自动下载模型。先从角色库选择角色，再设置本地模型或云端 API；可稍后设置，并从“模型与连接”重新打开。验证新安装、已有配置、读取/保存失败、切换角色时的迟到响应，以及跳过后不会误开先前选择的设置。保存成功只代表配置已写入，模型可用性仍以连接检测和真实聊天为准。

Chat Pro 安装版启动时，将缺失的内置角色和插件补充到应用数据目录的 `roles/`、`plugins/`，不覆盖已有包；导入、内核启动和文件监听共用这个可写角色库。桌面轻量状态的数据库仍在内存中，辅助文件使用应用数据目录，聊天持久化继续由独立内核负责。源码开发目录和有效的 `OCLIVE_ROLES_DIR` 覆盖值不迁移；其他发行版不采用这项 Chat Pro 默认库策略。

默认角色工作区的“帮助与恢复”接入 `SupportDiagnostics.vue`：内核/环境状态、本次启动最近 60 条 API/界面报错、应用日志尾部（最多 200 行 / 96 KB）及手动复制/保存。日志通过 `read_support_logs` 只读命令读取固定应用数据目录的 `logs/`，不接受任意路径；不会自动上传、清理或修复。显示和导出遮蔽常见密钥格式，分享前仍需查看本机路径和日志文字。

设置的分类与两壳入口见 [MODULE_MAP §13.1](../../../handoff/MODULE_MAP_AND_HANDOFF.md#131-chat-pro-壳与互动模式-ia)。验证设置改动时，除保存成功，还需检查读取失败、保存失败回退、手动重试、保存中换页、窄窗与键盘操作。布尔偏好统一使用 `UiToggle.vue`，原有多选或清理参数保持各自含义；不要把“开关看起来变了”当成已经保存成功。默认角色工作区不显示仅对旧壳生效的面板宽度重置。

Win98 彩蛋的样式依赖和触发/恢复约定见 [MODULE_MAP §13.2](../../../handoff/MODULE_MAP_AND_HANDOFF.md#132-chat-pro-外观正交轴-data-skin)。修改角色工作区或新增弹窗时，同时检查默认与 Win98 外观、关闭恢复、刷新保持和窄窗口；复古样式不应改变角色记录、草稿或模型选择。原生窗口按钮的 mock 测试只证明调用接线，真实标题栏操作需另做安装版验证。

面板右上角关闭使用共享 `UiCloseButton`，保留可访问名称与提示，不再各自写文字“关闭”、不同叉号字形或单独的点击区；业务取消/返回仍按操作含义展示文字。交互与动效的入口见 [MODULE_MAP §13.1](../../../handoff/MODULE_MAP_AND_HANDOFF.md#131-chat-pro-壳与互动模式-ia)。动效用于解释状态，不延迟关闭或保存；验证初次打开、Esc、Tab/Shift+Tab、组件卸载后的焦点返回、减少动态效果和 Win98。模型、设置或帮助改变层级时，保存中的归属检查、错误反馈和草稿保持仍需回归。

角色身份和关系的改动还需做重启恢复检查：有独立内核时，读写必须走内核持久化，不应依赖桌面临时内存状态；错误时也不能悄悄改成本地成功。

动漫功能图标的资源与消费边界见 [MODULE_MAP §13.1](../../../handoff/MODULE_MAP_AND_HANDOFF.md#131-chat-pro-壳与互动模式-ia)。新增图标时检查透明背景、深色/Win98/窄窗、真实加载与失败回退；文字标签继续承担可访问名称，插画不要挤占表单阅读空间或冒充用户选择的角色。

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
