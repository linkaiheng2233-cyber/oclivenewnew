# mumu 前端模块验收清单

用于发版前快速确认 `roles/mumu/ui.json` 推荐布局与目录插件状态是否可用。

---

## 1) 默认模块与插槽

- `chat.header` → `com.oclive.mumu.chat-header-status`
- `chat_toolbar` → `com.oclive.mumu.quick-actions`
- `role.detail` → `com.oclive.mumu.role-detail-card`
- `sidebar` → `com.oclive.mumu.sidebar-glance`
- `settings.panel` → `com.oclive.mumu.settings-panel`

验收动作：

1. 启动应用后按 `Ctrl+Shift+F` 打开插件管理。
2. 点一次 **重置为角色包推荐**，确认以上 5 个模块都在对应插槽可见列表中。
3. 保存后重启应用，确认布局保持一致。

---

## 2) 关键交互检查

- `chat_toolbar`：点击快捷短语会直接发送；场景按钮能触发“仅我过去 / 同行前往”。
- `sidebar`：点击“建议下一句”只填充输入框，不自动发送。
- `settings.panel`：点击“恢复默认”会弹确认框，执行后出现成功/失败状态提示。
- `role.detail`：角色切换后卡片字段随 `role:switched` 更新。

---

## 3) 回退与异常

1. 在插件管理启用 **强制 iframe 模式**，确认 5 个模块仍能显示基础内容。
2. 任意模块读取失败时应出现错误文案，不影响主聊天流程。
3. 关闭强制 iframe 后，Vue 组件可恢复正常渲染。

---

## 4) 发版建议

- 导出角色包前，确认 `roles/mumu/ui.json` 的 `slots` 配置包含上述 5 个插件 ID。
- 如调整了模块文案或视觉，请同步更新 `creator-docs/FAQ.md` 对应说明。
