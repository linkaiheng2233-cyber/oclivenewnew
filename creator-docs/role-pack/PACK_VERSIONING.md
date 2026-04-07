# 角色包版本与兼容性

本文说明 **manifest / settings** 的版本字段、未知字段策略，以及与 [`PLUGIN_V1.md`](../plugin-and-architecture/PLUGIN_V1.md)、[`roles/README_MANIFEST.md`](../../roles/README_MANIFEST.md) 的关系。实现以源码为准。**全库文档索引**：[../getting-started/DOCUMENTATION_INDEX.md](../getting-started/DOCUMENTATION_INDEX.md)。

## `settings.json`：`schema_version`

- **字段**：`schema_version`（`u32`），见 [`DiskRoleSettings`](../../src-tauri/src/models/role_settings_disk.rs)。
- **默认值**：`1`（见 `default_schema_version`）。
- **用途**：为未来「破坏性结构调整」预留；当前运行时以 **最新契约** 解析，若将来引入不兼容变更，可提高版本并由加载器分支处理。

## `plugin_backends` 与 PLUGIN_V1

- **`plugin_backends`**：可选；省略时记忆 / 情绪 / 事件 / Prompt 为 **builtin**，**`llm` 为 `ollama`**（见 [`PluginBackends`](../../src-tauri/src/models/plugin_backends.rs)）。
- 各后端枚举与语义见 **[PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md)**。
- **未知枚举值**：反序列化可能失败，需修正拼写或扩展 `serde` 接受别名（若日后增加）。

## `min_runtime_version`（预留）

- **状态**：路线图字段；**当前仓库可不写**。若将来加入，建议形式为 **semver 字符串**（如 `"0.3.0"`），由宿主在加载角色前比较 **应用版本 ≥ min**，否则拒绝加载并提示升级 oclive。
- 落地时需与 **Tauri / Cargo 版本** 单一来源对齐，避免手写漂移。

## 未知 JSON 键

- **`manifest.json` / `settings.json`**：以各结构体的 `serde` 属性为准；未使用 `deny_unknown_fields` 的表在解析时可能 **忽略**多余键（便于向前兼容）。若某结构体启用 **拒绝未知字段**，多余键将导致加载失败——以该类型的定义为准。
- **推荐**：创作者专用键使用 **`_` 前缀**（见 README_MANIFEST），加载器忽略。

## 校验链

- 合并后的磁盘 manifest 经 **`validate_disk_manifest`**（[`role_manifest_validate`](../../src-tauri/src/domain/role_manifest_validate.rs)）再转为运行时 `Role`。
- 修改契约时同步：**Rust 校验**、`roles/README_MANIFEST.md`、**本文件**、必要时 **PLUGIN_V1**。

## `knowledge`（世界观知识，可选）

- **语义**：角色包内 **`knowledge/`** 目录下的 Markdown 资源，用于 Prompt 检索与事件关键词补充；**不是** `plugin_backends` 子系统，详见 [WORLDVIEW_KNOWLEDGE.md](./WORLDVIEW_KNOWLEDGE.md)。
- **字段**：`enabled`（`bool`）、`glob`（`string`，须以 `knowledge/` 开头；默认 `knowledge/**/*.md`）。
- **校验**：`glob` 不能为空字符串（见 `validate_knowledge_manifest_disk`）。

## 相关文档

- [EDITOR_VALIDATION_ROADMAP.md](./EDITOR_VALIDATION_ROADMAP.md) — 编写器与运行时 **校验分工**（短期以 `load_role` 为准；中期可选 crate/CLI）  
- [../getting-started/DOCUMENTATION_INDEX.md](../getting-started/DOCUMENTATION_INDEX.md) — 文档总索引与阅读顺序  
- [../plugin-and-architecture/PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md) — 可替换子系统契约与 `plugin_backends` 枚举  
- [../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md](../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md) — HTTP 侧车 JSON-RPC  
- [../roadmap/VISION_ROADMAP_MONTHLY.md](../roadmap/VISION_ROADMAP_MONTHLY.md) — 产品里程碑  
- [roles/README_MANIFEST.md](../../roles/README_MANIFEST.md) — 创作者门面说明  
- [WORLDVIEW_KNOWLEDGE.md](./WORLDVIEW_KNOWLEDGE.md) — 世界观知识目录、front matter、`event_hints`  
