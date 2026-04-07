# 创作者与开源文档（oclive）

本目录为 **面向创作者、侧车开发者与插件扩展者** 的文档根目录，按主题分子文件夹。**不再使用**旧的扁平 `docs/*.md` 布局（见 `docs/README.md` 说明）。

---

## 目录结构

| 文件夹 | 内容 |
|--------|------|
| **[getting-started/](getting-started/)** | 文档总索引、入门与角色包工作流、[GitHub 仓库清单](getting-started/GITHUB_REPO_CHECKLIST.md) |
| **[plugin-and-architecture/](plugin-and-architecture/)** | `plugin_backends` 契约、扩展点、HTTP JSON-RPC 协议、替换模块 |
| **[role-pack/](role-pack/)** | 包版本、场景、用户身份、角色包自定义 |
| **[roadmap/](roadmap/)** | 愿景与按月路线图、[体验差异化 backlog](roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md) |

---

## 从这里开始

1. 打开 **[getting-started/DOCUMENTATION_INDEX.md](getting-started/DOCUMENTATION_INDEX.md)**，按「快速入口」表选读。  
2. 先看错误与排障：**[getting-started/ERROR_CODES.md](getting-started/ERROR_CODES.md)**（用户提示与开发日志对齐）。  
3. 做角色包内容： **[getting-started/CREATOR_WORKFLOW.md](getting-started/CREATOR_WORKFLOW.md)**（运行时与**独立编写器**分工、**`OCLIVE_ROLES_DIR`**）+ 仓库 **[roles/README_MANIFEST.md](../roles/README_MANIFEST.md)**。编写器为另仓（如 **`oclive-pack-editor`**），包为唯一对接面。  
4. 做 HTTP 侧车： **[plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md](plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md)** + **[plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md](plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md)**。  
5. 联调示例： **[examples/remote_plugin_minimal/README.md](../examples/remote_plugin_minimal/README.md)**。

---

## 与仓库其他文档的关系

| 位置 | 说明 |
|------|------|
| 根目录 **[README.md](../README.md)** | 项目简介、构建命令 |
| 根目录 **[ARCHIVE_PROJECT_HISTORY.md](../ARCHIVE_PROJECT_HISTORY.md)** | 开发日志与交接材料归档索引（非创作者必读） |
| **[CONTRIBUTING.md](../CONTRIBUTING.md)** / **[SECURITY.md](../SECURITY.md)** | 贡献与安全 |
| **[CHANGELOG.md](../CHANGELOG.md)** | 版本变更 |
