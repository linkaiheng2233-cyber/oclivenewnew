# 角色包格式规范（ROLE_PACK_SPEC）

本文档描述 **与 oclive 主宿主加载逻辑一致** 的磁盘角色包形状，便于 **多发行版**（桌面 Tauri、无头 `kernel_server`、未来启动器）共用同一包。权威细节仍以源码与既有文档为准：

- 创作者门面与字段语义：[README_MANIFEST.md](../../roles/README_MANIFEST.md)
- 六宿主槽与编排：[PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md)、[SETTINGS_REFERENCE.md](../cli/SETTINGS_REFERENCE.md)
- 以内核为中心的模块图：[KERNEL_AND_MODULES_ARCHITECTURE.md](../getting-started/KERNEL_AND_MODULES_ARCHITECTURE.md)

**标准 JSON 无 `//` 注释**；说明请用 **`_` 前缀键**（加载时忽略），或写在包外文档。

---

## 1. 目录结构（推荐）

角色包根目录通常命名为 **`roles/{角色id}/`**（`{角色id}` 与 `manifest.json` → **`id`** 一致，便于导入与校验）。

```text
roles/{role_id}/
├── manifest.json           # 门面：展示信息、七维 default_personality、场景列表、关系等
├── settings.json           # 可选；引擎段：plugin_backends、evolution、schema_version 等
├── core_personality.txt    # 可选；核心性格长文（profile 模式等）
├── ui.json                 # 可选；前端布局
├── author.json             # 可选；作者元数据
├── scenes/
│   └── {scene_id}/
│       ├── scene.json
│       ├── description.txt # 可选
│       └── …
├── knowledge/              # 可选；世界观 Markdown（见 WORLDVIEW_KNOWLEDGE.md）
├── memories/               # 可选；预设记忆资源（若产品使用）
├── assets/                 # 可选；立绘、头像等静态资源
└── pipeline.ocblueprint    # 可选；运行时蓝图（与 monolith.toml 编译期焊接正交，见 RFC）
```

**说明**：仓库内 **不存在** 顶层 `personality.json` 文件约定；**七维人格**在 **`manifest.json` → `default_personality`**（7 个 `f32`，见下文）。`prompts/*.md` 可作为创作者自管素材，**不是**宿主加载的必需路径；主对话 Prompt 由引擎与 `plugin_backends.prompt` 决定。

---

## 2. `manifest.json`（`DiskRoleManifest`）

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `id` | string | 是 | 角色唯一 id；建议稳定、与文件夹名一致 |
| `name` | string | 是 | 展示名 |
| `version` | string | 是 | 语义化版本（字符串） |
| `author` | string | 是 | 作者 |
| `description` | string | 是 | 简短描述 |
| `default_personality` | number[] | 否 | 七维 `f32`，顺序：stubbornness, clinginess, sensitivity, assertiveness, forgiveness, talkativeness, warmth；**若非空须恰好 7 维**，每维 **0.0～1.0**（`oclive pack validate` 会校验） |
| `scenes` | string[] | 否 | 场景 id；可与 `scenes/` 子目录合并（见 `merge_role_pack_scene_ids`） |
| `user_relations` | object | 是 | 键为关系 id；值含 `initial_favorability`（0～100）、`favor_multiplier`（正数）等 |
| `default_relation` | string | 否 | 须在 `user_relations` 中存在；可空则加载时回退 |
| `evolution` | object | 否 | 见 README_MANIFEST；`personality_source`：`vector` \| `profile` |
| `memory_config` | object | 否 | `topic_weights` 的键须为已声明场景 |
| `identity_binding` | string | 否 | `global` \| `per_scene` |
| `life_trajectory` / `life_schedule` / `knowledge` / … | 可选块 | 否 | 见 README_MANIFEST |
| `min_runtime_version` | string | 否 | semver；与校验时传入的宿主版本比较 |
| `dev_only` | bool | 否 | 调试包标记 |

---

## 3. `settings.json`（`DiskRoleSettings`）

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `schema_version` | u32 | 是 | 当前宿主支持 **1**（见 `CURRENT_SETTINGS_SCHEMA_VERSION`） |
| `plugin_backends` | object | 否 | **六宿主槽** + `directory_plugins` + `local_memory_provider_id`；与 `PluginBackends` 一致（见 SETTINGS_REFERENCE）。脚手架可写 **`complex_emotion`** 扩展键，**宿主反序列化时忽略** |
| `interaction_mode` | string | 否 | `immersive` \| `pure_chat` |
| `evolution` / `memory_config` / `ollama_model` / `remote_presence` / `autonomous_scene` / `knowledge` / `reply_quality_anchor` | 可选 | 否 | 合并进 manifest 后再校验；见 README_MANIFEST |

---

## 4. 与内核概念对齐

| 概念 | 磁盘落点 |
|------|-----------|
| `PluginBackends`（memory…agent） | `settings.json` → `plugin_backends` |
| 七维人格（vector 模式） | `manifest.json` → `default_personality` |
| 交互模式 | `settings.json` → `interaction_mode` |
| 场景 | `manifest.scenes` + `scenes/{id}/` |
| Monolith 焊接 | **仅** 脚手架项目 `monolith.toml` / `process_message_monolith.rs`，**不**随角色包分发 |

---

## 5. 自动化校验

```bash
cargo run -p oclive-cli -- pack validate ./roles/my-role --host-version 0.2.0
```

- 默认 `--host-version` 为 **本 CLI 的 `CARGO_PKG_VERSION`**；与桌面宿主版本不一致时，请显式传入 **与目标 oclive 发行版一致的 semver** 再检查 `min_runtime_version`。
- 通过时输出：`✓ 角色包验证通过`；失败时逐条列出错误。

**JSON Schema**（IDE 提示 / 外部校验器）：`crates/oclive-cli/schemas/role_pack_manifest.schema.json`、`role_pack_settings.schema.json`。

---

## 6. 脚手架命令摘要

| 命令 | 作用 |
|------|------|
| `pack validate <dir>` | 目录级校验 |
| `pack create -o <out> --id <id> [--flat]` | 生成最小可校验包（`--flat` 时 `<out>` 即为角色根） |
| `pack publish <dir> [-o file.oclivepack]` | ZIP 打包；根目录为 `manifest.id` |
| `init … --skip-role-pack` | 生成内核工程时不创建 `roles/` |

详见 [OCLIVE_CLI_GUIDE.md](../cli/OCLIVE_CLI_GUIDE.md)。

---

[English](../../creator-docs-en/role-pack/ROLE_PACK_SPEC.md)
