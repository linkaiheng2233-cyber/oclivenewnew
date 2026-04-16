# 编写器（oclive-pack-editor）与主程序（oclivenewnew）版本兼容说明

本文档说明 **角色包内 `ui.json`** 与 **主程序** 的兼容关系，避免「编写器导出的字段主程序不认识」或「主程序已支持但编写器未导出」的困惑。

**版本号格式**：两项目均采用 **语义化版本（SemVer）** `MAJOR.MINOR.PATCH`，见各仓库根目录 **`package.json`** 的 **`version`** 字段。

**当前仓库快照（文档更新时）**：

- **oclivenewnew**（主程序）：`0.2.x`（见根 `package.json` / `CHANGELOG.md`）
- **oclive-pack-editor**（编写器）：`0.2.x`（见编写器仓库 `package.json`）

> 下列表格中 **0.3.x / 0.4.x** 行为 **规划行**：发版后请以对应 `CHANGELOG.md` 与 `ui.json.schema.json` 为准并更新本表。

---

## 兼容性表

| 编写器版本 | 主程序最低版本 | 新增或强依赖的 `ui.json` 能力 | 备注 |
|------------|----------------|--------------------------------|------|
| **0.2.x** | **0.2.0** | `shell`、`slots`（`chat_toolbar`、`settings_panel`、`role_detail` 等）、基础 `theme` / `layout`（以 schema 为准） | 当前主线；与 [role-pack/ui.json.schema.json](role-pack/ui.json.schema.json) 对齐 |
| **0.3.x**（规划） | **0.3.0**（规划） | 若 schema 扩展 **主题/布局** 细分字段，以发版说明为准 | 主程序较低版本可能 **忽略未知字段**（JSON 反序列化通常带 `default`） |
| **0.4.x**（规划） | **0.4.0**（规划） | **`sidebar`、`chat.header`** 等插槽在编写器中完整配置时，需主程序 **Directory 插件引导** 已支持对应插槽（见 [DIRECTORY_PLUGINS.md](plugin-and-architecture/DIRECTORY_PLUGINS.md)） | 插槽名与宿主 `pluginStore` 常量一致 |
| **开发版** | **同开发版** | schema 与主程序 `UiConfig` 同分支 | 仅建议开发者本地对拍 |

---

## 升级与降级行为

1. **主程序版本低于编写器目标**  
   - **`ui.json`** 中主程序 **不认识的字段**：若 Rust/TS 模型使用 **`serde` 默认 + 可选字段**，通常 **静默忽略**；若某版本改为 **拒绝未知字段**，以该版本 `CHANGELOG` 为准。  
   - **已声明但宿主未实现的插槽**：该插槽在 UI 中可能 **不显示** 或 **无操作**，需升级主程序。

2. **编写器版本低于主程序**  
   - 主程序 **新插槽 / 新主题键** 可能无法在旧编写器中编辑；可 **手动编辑 `ui.json`** 并参照 [ui.json.schema.json](role-pack/ui.json.schema.json)。

3. **角色包 `settings.json` 与 `plugin_backends`**  
   - 兼容性与 **`min_runtime_version`**、宿主 `load_role` 校验相关，见 [PACK_VERSIONING.md](role-pack/PACK_VERSIONING.md)、[CHANGELOG.md](../CHANGELOG.md)。

---

## 如何查看版本

| 产品 | 查看方式 |
|------|----------|
| **主程序** | 应用内 **设置 / 关于**（若有）；或安装包名与仓库 **`package.json`** / **`CHANGELOG.md`** |
| **编写器** | 编写器窗口 **关于**；或仓库 **`package.json`** |

---

## 相关文档

- [role-pack/ui.json.schema.json](role-pack/ui.json.schema.json)
- [plugin-and-architecture/DIRECTORY_PLUGINS.md](plugin-and-architecture/DIRECTORY_PLUGINS.md)
- [CHANGELOG.md](../CHANGELOG.md)
