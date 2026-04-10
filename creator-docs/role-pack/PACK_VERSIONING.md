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

## `min_runtime_version`（manifest，可选）

- **字段**：`manifest.json` 顶层 **`min_runtime_version`**（`Option<String>`），见共享 crate [`DiskRoleManifest`](../../crates/oclive_validation/src/manifest.rs)。
- **形式**：**semver**（如 `"0.2.0"`），由 [`semver`](https://crates.io/crates/semver) 解析；与 **oclivenewnew 应用版本**比较（`src-tauri/Cargo.toml` 的 `version`，编译期 `env!("CARGO_PKG_VERSION")`）。
- **语义**：若 **宿主版本低于** `min_runtime_version`，`load_role` **拒绝加载**并返回可读错误（提示升级 oclive）。省略该字段则不检查。
- **编写器**：`oclive-pack-editor` 中 `HOST_RUNTIME_VERSION`（`src/lib/hostRuntimeVersion.ts`）应与上述 **Cargo 版本**同步；`npm run wasm:build` 生成的校验 wasm 与 [`validate_min_runtime_version`](../../crates/oclive_validation/src/validate.rs) 一致。

## 未知 JSON 键（顶层收紧）

- **`manifest.json` / `settings.json` 根对象**：加载时在反序列化前执行 **顶层键白名单**（[`json_keys`](../../crates/oclive_validation/src/json_keys.rs)）：不在白名单内的键 **报错**；**以下划线 `_` 开头的键**视为创作者说明，**允许**（见 [`roles/README_MANIFEST.md`](../../roles/README_MANIFEST.md)）。
- **嵌套对象**内多出的键仍主要由 **serde 结构体**决定是否忽略（与历史行为一致）；若需进一步收紧，再单独开契约变更。

## 校验链

- 合并后的磁盘 manifest 经 **`validate_disk_manifest`**（[`role_manifest_validate`](../../src-tauri/src/domain/role_manifest_validate.rs)）与 **`validate_min_runtime_version`** 再转为运行时 `Role`。
- 加载完成后另有 **`validate_role_interaction_mode`**；若 `plugin_backends` 声明 **`remote`** 但未设置对应 `OCLIVE_REMOTE_*` 环境变量，运行时会 **`log::warn`**（不阻止加载；运行时仍按 PLUGIN_V1 回退内置或进程内 LLM）。见 [`log_plugin_backends_remote_missing_env`](../../src-tauri/src/domain/role_manifest_validate.rs)。
- 修改契约时同步：**Rust 校验**、`roles/README_MANIFEST.md`、**本文件**、必要时 **PLUGIN_V1**。

## 第 1 月里程碑（契约边界）— 与源码对齐

路线图「第 1 月」中的**可替换子系统**在本仓库已落实为：

- **契约文档**：[`PLUGIN_V1.md`](../plugin-and-architecture/PLUGIN_V1.md)（含 `send_message` 编排）、本文与 [`VISION_ROADMAP_MONTHLY.md`](../roadmap/VISION_ROADMAP_MONTHLY.md) 第 1 月条目（已更新为 `plugin_backends` 命名）。
- **代码边界**：[`PluginHost`](../../src-tauri/src/domain/plugin_host.rs) + [`PluginBackends`](../../src-tauri/src/models/plugin_backends.rs)；**不是**单独的 `memory_backend` / `affect_backend` 顶层 manifest 字段。
- **`min_runtime_version`**：已启用，与 **`src-tauri/Cargo.toml` `version`** 对齐比较。

## `knowledge`（世界观知识，可选）

- **语义**：角色包内 **`knowledge/`** 目录下的 Markdown 资源，用于 Prompt 检索与事件关键词补充；**不是** `plugin_backends` 子系统，详见 [WORLDVIEW_KNOWLEDGE.md](./WORLDVIEW_KNOWLEDGE.md)。
- **字段**：`enabled`（`bool`）、`glob`（`string`，须以 `knowledge/` 开头；默认 `knowledge/**/*.md`）。
- **校验**：`glob` 不能为空字符串（见 `validate_knowledge_manifest_disk`）。

## `evolution.personality_source`（摘要）

- **字段**：`vector`（默认）或 `profile`，见 `EvolutionConfigDisk` / 共享校验。
- **语义摘要**：`profile` 表示 **核心性格档案**（`core_personality.txt`）+ **运行时可变档案**（DB，模型维护）；**七维**多为视图。细则见 **[docs/personality-archive-notes.md](../../docs/personality-archive-notes.md)** 与 [roles/README_MANIFEST.md](../../roles/README_MANIFEST.md) §5.3。

## 相关文档

- [EDITOR_VALIDATION_ROADMAP.md](./EDITOR_VALIDATION_ROADMAP.md) — 编写器与运行时 **校验分工**（短期以 `load_role` 为准；中期可选 crate/CLI）  
- [../getting-started/DOCUMENTATION_INDEX.md](../getting-started/DOCUMENTATION_INDEX.md) — 文档总索引与阅读顺序  
- [docs/personality-archive-notes.md](../../docs/personality-archive-notes.md) — 性格档案设计轴心  
- [../plugin-and-architecture/PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md) — 可替换子系统契约与 `plugin_backends` 枚举  
- [../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md](../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md) — HTTP 侧车 JSON-RPC  
- [../roadmap/VISION_ROADMAP_MONTHLY.md](../roadmap/VISION_ROADMAP_MONTHLY.md) — 产品里程碑  
- [roles/README_MANIFEST.md](../../roles/README_MANIFEST.md) — 创作者门面说明  
- [WORLDVIEW_KNOWLEDGE.md](./WORLDVIEW_KNOWLEDGE.md) — 世界观知识目录、front matter、`event_hints`  
