# 角色包格式规范（ROLE_PACK_SPEC）

**创作者学习路径（时间盒：入门 → 进阶 → 发布）**：[CREATOR_LEARNING_PATH.md](CREATOR_LEARNING_PATH.md)

本文档描述 **与 oclive 主宿主加载逻辑一致** 的磁盘角色包形状，便于 **多发行版**（桌面 Tauri、无头 `kernel_server`、未来启动器）共用同一包。权威细节仍以源码与既有文档为准：

- 创作者门面与字段语义：[README_MANIFEST.md](../../roles/README_MANIFEST.md)
- 六宿主槽与编排：[PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md)、[SETTINGS_REFERENCE.md](../cli/SETTINGS_REFERENCE.md)
- 以内核为中心的模块图：[KERNEL_AND_MODULES_ARCHITECTURE.md](../getting-started/KERNEL_AND_MODULES_ARCHITECTURE.md)

**标准 JSON 无 `//` 注释**；说明请用 **`_` 前缀键**（加载时忽略），或写在包外文档。

---

## 1. 目录结构（推荐）

角色包根目录通常命名为 **`roles/{角色id}/`**（v2 时 `{角色id}` 与 `meta.id` 一致）。

```text
roles/{role_id}/
├── pipeline.ocblueprint    # **v2 SSOT（推荐）**：schema_version 2 · meta · slot_registry
├── manifest.json           # **已废弃（legacy）**：勿与 v2 蓝图并存
├── settings.json           # **已废弃（legacy）**：勿与 v2 蓝图并存
├── core_personality.txt    # 可选；profile 模式长文
├── ui.json                 # 可选；前端布局
├── author.json             # 可选；作者元数据
├── scenes/
│   └── {scene_id}/ …
├── knowledge/              # 可选
├── memories/               # 可选
└── assets/                 # 可选
```

**说明**：v2 包 **不得** 同时存在 `manifest.json` / `settings.json` 与 `pipeline.ocblueprint`。七维人格在 v2 写入 **`meta.personality`**（对象或 7 元数组）。`prompts/*.md` 非宿主必需路径。

---

## 2. `pipeline.ocblueprint`（v2 SSOT）

| 顶层键 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| `schema_version` | number | 是 | 固定 **2** |
| `meta` | object | 是 | 原 `manifest.json` + 原 `settings.json` 引擎字段（见下表） |
| `slot_registry` | object | 是 | 实例键 → 槽位配置；**至少一个 `type: llm`** |

### 2.1 `meta`（门面与引擎）

| 字段 | 说明 |
|------|------|
| `id`, `name`, `version`, `author`, `description` | 与 legacy manifest 同义 |
| `personality` | 七维：对象（`stubbornness`…`warmth`）或 `[f32; 7]`，0.0～1.0 |
| `relations`, `default_relation` | 用户关系 |
| `scenes` | 场景 id 列表；与 `scenes/` 子目录合并 |
| `evolution`, `memory_config`, `identity_binding`, `life_*`, `knowledge` | 见 README_MANIFEST |
| `interaction_mode` | `immersive` \| `pure_chat` |
| `ollama_model`, `remote_presence`, `autonomous_scene`, `reply_quality_anchor` | 可选 |
| `min_runtime_version`, `dev_only` | 可选 |

### 2.2 `slot_registry`（开放多实例）

键为**用户定义的实例名**（如 `memory`、`memory_short`、`llm`）。值：

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `type` | string | 是 | `memory` \| `emotion` \| `event` \| `prompt` \| `llm` \| `agent` \| `complex_emotion` |
| `label` | string | 是 | 架构图展示名 |
| `backend` | string | 是 | 与该 `type` 对应的 PLUGIN_V1 枚举 |
| `position` | number | 是 | 同 `type` 排序；折叠六槽时 **last-wins** |
| `plugin` / `plugins` | string / string[] | directory 时必填 | 目录插件 id |
| `model`, `url`, `local_memory_provider_id` | 可选 | 见 SETTINGS_REFERENCE |

同 `type` 多实例由 **`SlotRunner`** 合并（见 RFC）；主应用架构图可通过 **`save_role_slot_registry`** 写回本文件。

**架构图编辑规则（主应用）**：可增删 `slot_registry` 键；**至少一个 `type: llm`**；删除时 **不可移除最后一个 llm** 实例。字段校验与写盘逻辑见 `oclive_validation` 与 Tauri `save_role_slot_registry`。

### 2.3 `groups`（可选 · 架构图分组）

将同 **`type`** 的多个 `slot_registry` 实例归到逻辑分组，供主应用架构图绘制边框（可折叠）。

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `label` | string | 是 | 分组标题（架构图边框顶栏） |
| `description` | string | 否 | 创作者备注 |
| `type` | string | 是 | 六种模块之一：`memory` \| `emotion` \| `event` \| `prompt` \| `llm` \| `agent` |
| `members` | string[] | 是 | `slot_registry` 实例键列表；每项须存在且 `type` 与分组 `type` 一致 |

**规则**：`members` 非空；同一实例键只能属于一个分组；`complex_emotion` 不参与分组。

示例：

```json
"groups": {
  "memory_tier": {
    "label": "记忆层",
    "type": "memory",
    "members": ["memory_long", "memory_short"]
  }
}
```

### 2.4 `module_relations`（仅运行时）

**禁止**在 `pipeline.ocblueprint` 文件中出现 `module_relations`、`steps`、`entry`（校验报错）。运行时由 `slot_registry` **派生**模块间示意关系，供架构图只读连线。

JSON Schema：`crates/oclive-cli/schemas/pipeline.ocblueprint.v2.schema.json`。

---

## 3. `manifest.json`（legacy · `DiskRoleManifest`）

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

## 4. `settings.json`（legacy · `DiskRoleSettings`）

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `schema_version` | u32 | 是 | 当前宿主支持 **1**（见 `CURRENT_SETTINGS_SCHEMA_VERSION`） |
| `plugin_backends` | object | 否 | **六宿主槽** + `directory_plugins` + `local_memory_provider_id`；与 `PluginBackends` 一致（见 SETTINGS_REFERENCE）。脚手架可写 **`complex_emotion`** 扩展键，**宿主反序列化时忽略** |
| `interaction_mode` | string | 否 | `immersive` \| `pure_chat` |
| `evolution` / `memory_config` / `ollama_model` / `remote_presence` / `autonomous_scene` / `knowledge` / `reply_quality_anchor` | 可选 | 否 | 合并进 manifest 后再校验；见 README_MANIFEST |

---

## 5. 与内核概念对齐

| 概念 | 磁盘落点（v2 蓝图） | legacy |
|------|---------------------|--------|
| `PluginBackends`（memory…agent） | `pipeline.ocblueprint` → `slot_registry`（同 `type` 多实例，折叠为六槽时 **last-wins**） | `settings.json` → `plugin_backends` |
| 七维人格（vector 模式） | `meta.personality`（对象或 7 元数组） | `manifest.json` → `default_personality` |
| 交互模式 | `meta.interaction_mode` | `settings.json` → `interaction_mode` |
| 场景 | `meta.scenes` + `scenes/{id}/` | `manifest.scenes` + `scenes/{id}/` |
| 会话槽覆盖 | 内存 overlay；架构图改包默认经 `save_role_slot_registry` 写盘 | `set_session_plugin_backend` |
| Monolith 焊接 | **仅** 脚手架 `monolith.toml` / `process_message_monolith.rs`，**不**随角色包分发 | 同左 |

校验：`cargo run -p oclive-cli -- pack validate <dir>`（**默认 v2**）；legacy 包用 `--profile legacy`。另：`blueprint validate <dir>`。路线图 [`handoff/BLUEPRINT_V2_IMPLEMENTATION_PLAN.md`](../../handoff/BLUEPRINT_V2_IMPLEMENTATION_PLAN.md)。

---

## 6. 自动化校验

```bash
cargo run -p oclive-cli -- pack validate ./roles/mumu --host-version 0.2.0
```

- 默认 `--host-version` 为 **本 CLI 的 `CARGO_PKG_VERSION`**；与桌面宿主版本不一致时，请显式传入 **与目标 oclive 发行版一致的 semver** 再检查 `min_runtime_version`。
- 通过时输出：`✓ 角色包验证通过`；失败时逐条列出错误。

**JSON Schema**（IDE 提示 / 外部校验器）：`crates/oclive-cli/schemas/role_pack_manifest.schema.json`、`role_pack_settings.schema.json`。

### RobotSoulPack（`--profile robot-soul`）

在标准目录校验通过后追加，用于 **机器人 / 无头 / 嵌入式** 最小可交付「灵魂包」：

| 规则 | 说明 |
|------|------|
| `manifest.min_runtime_version` | 必填、非空 semver，与目标宿主对齐 |
| `settings.json` | 必须存在 |
| `settings.plugin_backends` | 必须显式写出对象（六槽；可选 `complex_emotion` 等扩展键） |
| `settings.interaction_mode` | 必填：`immersive` 或 `pure_chat` |
| 人格载体 | **二选一**：非空 `core_personality.txt`，或 `manifest.default_personality` 恰好 7 维（0.0～1.0） |
| `remote_presence` | 可选 |

```bash
cargo run -p oclive-cli -- pack validate ./roles/my-role --host-version 0.2.0 --profile robot-soul
```

示例：`examples/robot-soul-minimal/roles/default/`。

---

## 7. 脚手架命令摘要

| 命令 | 作用 |
|------|------|
| `pack validate <dir>` | **默认** v2 蓝图目录校验 |
| `pack validate <dir> --profile legacy` | legacy manifest/settings |
| `pack validate <dir> --profile robot-soul` | legacy + RobotSoulPack（见 §6） |
| `pack create -o <out> --id <id> [--flat]` | 生成最小可校验包（`--flat` 时 `<out>` 即为角色根） |
| `pack publish <dir> [-o file.oclivepack]` | ZIP 打包；根目录为 `manifest.id` |
| `init … --skip-role-pack` | 生成内核工程时不创建 `roles/` |

详见 [OCLIVE_CLI_GUIDE.md](../cli/OCLIVE_CLI_GUIDE.md)。

---

## 8. 团队协作（`oclive collab`）

在角色包根目录使用 Git 同步多人编辑：

```yaml
# .oclive-collab.yml（由 oclive collab init 生成）
remote: git@github.com:user/role-pack.git
branch: main
auto_sync: false
```

| 命令 | 说明 |
|------|------|
| `collab init --remote <url>` | 写入上述文件并配置 `origin` |
| `collab status` | 工作区是否干净；相对 `origin/<branch>` 领先/落后提交数 |
| `collab pull` | 拉取远程（本地有未推送提交时会警告） |
| `collab push` | 推送（要求已 commit；远程领先时须先 pull） |
| `collab diff` | `git diff origin/<branch>` |

冲突解决：手动合并文件 → `git add` → `git commit` → `oclive collab push`。

---

[English](../../creator-docs-en/role-pack/ROLE_PACK_SPEC.md)
