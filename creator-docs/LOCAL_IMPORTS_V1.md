## 本地导入（文件夹投放）v1

Oclive 支持一种“**加法**”导入习惯：把文件放进指定目录，Oclive Manager 负责**发现**，但**不会自动启用**。你必须在 Oclive Manager 中确认（权限/风险提示）后才会导入或安装。

### 目录结构

根目录位于运行时的 `app_data/imports/`（在 Oclive Manager → 本地导入中会显示实际路径）。

- `roles/`
  - 角色包：`.ocpak` / `.zip`
- `plugins/plugin/`
  - 插件包：`.zip` / `.oclive-plugin`
  - 插件目录：包含 `manifest.json` 的目录
- `plugins/module/`
  - 模块条目：`type: "module"` 的 JSON（**与市场同款格式**）
- `profiles/`
  - Profile：`type: "profile"` 的 JSON（**与市场同款格式**）

### 安全与确认（非常重要）

- **发现 ≠ 启用**：投放目录只会让条目“出现在待处理列表”，不会自动运行。
- **插件安装需要开发者模式**：本地 ZIP/目录/离线包安装属于开发者模式能力，且必须通过权限勾选确认。
- **权限授权**：安装前会展示 manifest 声明权限，用户勾选后才会写入 grants；若权限为高风险会触发二次确认。
- **离线包签名（`.oclive-plugin`）**：
  - 若同目录存在 `xxx.signature.json` 且能与本地缓存索引条目匹配，会进行签名校验并展示结果；
  - 若本地未缓存索引（未同步官方索引），可能无法验签，会提示原因。

### 本地 module/profile JSON 的格式

本地 `plugins/module/*.json` 与 `profiles/*.json` **必须为市场同款条目**，例如：

```json
{
  "type": "module",
  "id": "module.example.starter",
  "name": "Starter Module",
  "version": "1.0.0",
  "description": "依赖插件 + 可选后端覆盖（无代码）",
  "module": {
    "plugins": [
      { "id": "com.example.pluginA", "version": "1.2.3", "source": "official" }
    ],
    "backends": { "llm": "remote" }
  }
}
```

### 常见用法

- **创作者**：把离线包和签名文件丢进 `plugins/plugin/`，打开 Oclive Manager → 本地导入 → 安装并勾选权限。
- **配置配方**：把 module/profile JSON 丢进对应目录，在 Oclive Manager 的 “模块（Module）/Profile” 页签里应用。

