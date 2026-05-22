# oclive 工作室用户指南

**oclive 工作室（oclive-studio）** 合并原启动器与角色包编写器：同一安装包内提供 **启动模式** 与 **创作模式**，通过磁盘上的 **roles 根** 与 **oclivenewnew** 运行时衔接。

## 安装与整合包

- 单独安装工作室：从 [oclive-studio Releases](https://github.com/oclive-app/oclive-studio/releases) 获取安装包。
- **整合包**（工作室 + 运行时）：在仓库根目录执行 `scripts/package-studio-release.ps1`（Windows）或 `scripts/package-studio-release.sh`，生成 `oclive-studio-*-vX.Y.Z.zip`，内含 `oclive-studio` 与 `oclive-runtime/oclivenewnew` 及 `README.txt`。

## 配置（studio-config.json）

主配置文件位于应用数据目录下的 **`studio-config.json`**（Windows 示例：`%APPDATA%\com.oclive.studio\studio-config.json`）。

| 字段 | 说明 |
|------|------|
| `rolesDir` | roles 根目录 → 对应环境变量 `OCLIVE_ROLES_DIR` |
| `ocliveExe` / `ocliveProjectRoot` / `ocliveMode` | 拉起运行时的 exe 或 dev 工程 |
| `ocliveLlmMode` + `ocliveRemote*` | 本机 Ollama 或 Remote LLM |
| `lastMode` | 上次模式：`launch` / `create` |

**兼容层**：若仅存在旧版 `launcher-config.json`，首次启动会自动迁移为 `studio-config.json` 并将旧文件重命名为 `launcher-config.json.migrated.bak`。若两者并存，**以 studio-config.json 为准**。

## 启动模式

1. 配置 **roles 根** 与 **Ollama / Remote LLM**。
2. 使用 **环境诊断**（对齐 `oclive doctor` 思路）检查 Rust、Ollama、roles 可写与磁盘空间；可对部分项 **一键修复**。
3. 点击 **启动 oclive** 加载角色包并对话。

## 创作模式

顶栏或启动页进入 **创作角色包**（路由 `/create`）；首次进入会 **异步加载** 创作模块以减小首包体积。

### 创作模式编辑流程

1. **选择或新建角色包**：在左侧列表选中 `roles/<roleId>/`，或「新建角色包」生成 `manifest.json` + `settings.json` 骨架。
2. **编辑 manifest**：填写 `name`、`description`、`plugin_backends`（七槽）、可选 `slot_registry` / `pipeline.ocblueprint`（v2 蓝图）。保存后运行 **校验**（对齐 `oclive pack validate` 规则）。
3. **编辑 settings**：配置人格向量、提示词片段、复杂情感等；顶栏 **架构图** 可查看槽位与蓝图分组（`groups` 只读派生，边由 `slot_registry` 生成）。
4. **保存**：写入当前 roles 根；若配置了 `ocliveProjectRoot`，部分操作可触发运行时热重载（见 `oclive dev`）。

> 截图占位：创作模式主界面（左栏角色列表 + 中央 manifest/settings 标签 + 右侧架构图）— 发版素材目录 `creator-docs/studio/assets/`。

### 试聊（测试角色包）

1. 在创作模式打开目标角色包，点击 **试聊** / **预览对话**。
2. 工作室按 `studio-config.json` 拉起 **oclivenewnew `--api`**（或已配置的 exe），注入 `OCLIVE_ROLES_DIR` 与 LLM（Ollama / Remote）。
3. 在试聊面板输入消息，检查回复、`reply` 字段与插件槽降级提示；失败时查看 **环境诊断** 与运行时日志（`RUST_LOG=info`）。
4. 修改 manifest/settings 后再次试聊，无需重启整个工作室（若 API 进程已退出，试聊会自动重拉）。

> 截图占位：试聊侧栏与一轮对话结果。

### 导出角色包

1. 确认 **校验通过**（无阻塞级 manifest/settings 错误）。
2. 使用 **导出** / **发布到 roles 根**：将当前目录同步到 `studio-config.json` 的 `rolesDir` 下同名文件夹（覆盖前会提示）。
3. 可选 **打包**：生成 `.oclive-plugin` 或 zip 归档（与 `oclive pack` 行为一致时显示 SHA-256 摘要）；导出后可在 **启动模式** 中选择该角色包对话。

> 截图占位：导出确认对话框与 roles 根目录中的目标文件夹。

## 首次使用引导

首次启动显示三步引导（环境 → 角色包 → 对话）。完成后写入 `localStorage` 键 `studio.onboarding.completed`。顶栏 **重新显示引导** 可再次打开。

## 深链接

注册协议 **`oclive-studio://`**（macOS 安装包已声明；Windows 需安装后由系统关联）。

| URL | 行为 |
|-----|------|
| `oclive-studio://create` | 打开工作室并进入创作模式 |
| `oclive-studio://create?roleId=xxx` | 创作模式并提示打开 `roles/xxx` |

## 与 oclive-cli 的关系

环境排查亦可使用主仓 `cargo run -p oclive-cli -- doctor`；工作室内 **诊断环境** 为面向创作者的可视化子集。

## 相关文档

- [创作者流程](../getting-started/CREATOR_WORKFLOW.md)
- [工作室合并 RFC](../rfc/RFC_STUDIO_MERGE.md)
- [SETTINGS_REFERENCE（CLI 配置形状对照）](../cli/SETTINGS_REFERENCE.md)
