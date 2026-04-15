# 示例：chat_toolbar 原生 Vue 插槽

与 `directory-plugin-ui-slot` 相同，但额外声明 **`vueComponent`**，由主界面用 `vue3-sfc-loader` 编译 `.vue` 并注入 `inject('oclive')`。

若 `.vue` 加载失败，自动回退到 **`entry`** 的 iframe（`slots/toolbar.html`）。

安装：将本目录复制到 `roles/<某角色>/plugins/com.oclive.example.ui_slot_toolbar_vue/`（或全局 `plugins/` 目录，与现有目录插件机制一致），重启 Oclive。

验收：对话区输入框上方出现 **「Vue 工具栏 · N 个角色」** 按钮（样式继承宿主 CSS 变量）；删除或故意损坏 `ToolbarButton.vue` 时应回退到 iframe 文案。
