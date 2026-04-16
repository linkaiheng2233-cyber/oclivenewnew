# 开发者视频教程脚本：5 分钟写一个工具栏插件

**用途**：录制「从零为聊天页增加目录插件工具栏按钮」的 **5 分钟** 演示片，可配合提词器照读。  
**示例参考**：仓库内 **`examples/directory-plugin-ui-slot-vue/`**（可简化路径与文案）。

---

## 分镜与台词

| 时间 | 画面 | 台词 / 旁白 | 备注 |
|------|------|-------------|------|
| 0:00–0:30 | 成品：聊天页工具栏出现自定义按钮，点击后界面有反馈 | 「大家好。今天用大约五分钟，带你从零写一个 Oclive **目录插件**，在聊天页工具栏加一颗自己的按钮。」 | 先展示最终效果，建立预期。 |
| 0:30–1:00 | 文件树：`plugins/` 下新建文件夹；打开 `manifest.json` | 「首先在 **`plugins/`** 下面建一个文件夹，名字建议用反向域名风格，避免和别人冲突。然后在里面放 **`manifest.json`**：写上 **`schema_version: 1`**、**`id`**、**`version`**，在 **`ui_slots`** 里声明 **`chat_toolbar`**，填好 **`entry`** 和可选的 **`vueComponent`**。」 | 屏幕同时展示 JSON 关键字段。 |
| 1:00–2:00 | 编辑器打开 `ToolbarButton.vue`（或等价 `.vue`） | 「接着写一个简单的 Vue 组件：模板里放一个按钮。脚本里用 **`inject('oclive')`** 拿到宿主注入的 API，不要用 `window.__TAURI__` 裸调。」 | 强调 `inject('oclive')` 与桥接规范。 |
| 2:00–3:00 | 在 `onClick` 里调用 `oclive.invoke('get_current_role', { … })` 或 `get_role_info` | 「按钮点击时，调用 **`oclive.invoke`**，传入命令名和参数。比如查询当前角色，参数里带上 **`role_id`**。返回值可以 **`console.log`** 出来，方便你在控制台里确认。」 | 参数名与 [BRIDGE_API_REFERENCE.md](../plugin-and-architecture/BRIDGE_API_REFERENCE.md) 一致。 |
| 3:00–4:00 | 将文件夹放到 `plugins/`，运行 `npm run tauri dev`；点击按钮，控制台打印结果 | 「保存插件文件夹，启动 Oclive。进入聊天页，你会看到工具栏上新按钮。点一下，开发者工具控制台里应该能看到刚才的返回数据。」 | 若加载极快，可切到 Network 或慢速节流展示骨架屏（可选一句带过）。 |
| 4:00–5:00 | 文档与社区页尾页；`manifest` 里 `bridge` 可选扩展 | 「你可以继续给 **`manifest`** 加 **`bridge`**，声明允许调用的命令白名单；也可以加 **`settings.panel`** 做设置页。打包成 zip 后可以上传到社区站分享。更完整的 API 列表见项目里的 **桥接 API 参考文档**。」 | 展示 `creator-docs/plugin-and-architecture/BRIDGE_API_REFERENCE.md` 标题或仓库路径。 |

---

## 备用提示（口误时可替换）

- 「如果 Vue 没出来，检查是否开了 **强制 iframe**，那会只用 HTML 入口。」
- 「开发者模式下第一次加载会做 **安全扫描**，危险 API 会弹窗，点取消就不会编译 Vue。」

---

## 检查清单（录制前）

- [ ] 本地 `plugins/` 下插件已能扫描到，`manifest.json` 无语法错误。
- [ ] `vueComponent` 路径与文件一致。
- [ ] 开发者工具已打开，便于拍控制台。
