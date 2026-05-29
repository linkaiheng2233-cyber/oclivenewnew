# 角色包格式规范（ROLE_PACK_SPEC）

> **职责边界（必读）**：**角色包仅包含角色身份、人格、关系与提示词内容。系统配置（槽位、后端、模型、交互模式、双核等）由蓝图管理。** 完整划分见 **[handoff/ROLE_PACK_BOUNDARY.md](../../handoff/ROLE_PACK_BOUNDARY.md)**；系统字段清单见 **[SETTINGS_REFERENCE.md](../cli/SETTINGS_REFERENCE.md)**。

**创作者学习路径（时间盒：入门 → 进阶 → 发布）**：[CREATOR_LEARNING_PATH.md](CREATOR_LEARNING_PATH.md)

本文档描述 **与 A.I.Live 主宿主加载逻辑一致** 的磁盘角色包形状，便于 **多发行版**（桌面 Tauri、无头 `kernel_server`、未来启动器）共用同一包。权威细节仍以源码与既有文档为准：

- 创作者门面与字段语义：[README_MANIFEST.md](../../roles/README_MANIFEST.md)
- 六宿主槽与编排：[PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md)、[SETTINGS_REFERENCE.md](../cli/SETTINGS_REFERENCE.md)
- 以内核为中心的模块图：[KERNEL_AND_MODULES_ARCHITECTURE.md](../getting-started/KERNEL_AND_MODULES_ARCHITECTURE.md)

**标准 JSON 无 `//` 注释**；说明请用 **`_` 前缀键**（加载时忽略），或写在包外文档。

---

## 0. 角色包 vs 蓝图（职责）

| 组件 | 你只需关心（初级创作者） | 蓝图 / 管理员 |
|------|--------------------------|---------------|
| **角色包** | 身份、七维人格、关系、**`prompts/`** 系统提示词与开场白、场景文案 | — |
| **蓝图** | **不要改**（除非你是集成方） | **`slot_registry`**、**`groups`**、后端 **`backend`**、**`model`**、**`interaction_mode`**、**`memory_config`**、远程/自主场景策略、**`dual_core.enabled`**（RFC）等 |

v2 磁盘上常为 **同一文件** `pipeline.ocblueprint`：`meta` 中仅上表「角色包」字段由编写器默认暴露；**`slot_registry` 及引擎向 `meta` 键** 归蓝图（见 [SETTINGS_REFERENCE.md](../cli/SETTINGS_REFERENCE.md) §零）。

**创作者可编辑（`meta` 子集）**：`id`、`name`、`version`、`author`、`description`、`personality`、`relations`、`default_relation`、`scenes`；可选剧情向 `life_*`、`evolution.personality_source`。

**创作者不应接触**：`slot_registry`、`groups`、`runtime_config`、`pipeline`、各实例 `backend` / `model` / `plugin` 等（见 [SETTINGS_REFERENCE.md](../cli/SETTINGS_REFERENCE.md) §零 `runtime_config`）。

---

## 1. 目录结构（推荐）

角色包根目录通常命名为 **`roles/{角色id}/`**（v2 时 `{角色id}` 与 `meta.id` 一致）。

```text
roles/{role_id}/
├── pipeline.ocblueprint    # **v2 SSOT（瘦）**：meta + slot_registry + includes；见 [BLUEPRINT_FOLDER_LAYOUT.md](../../handoff/BLUEPRINT_FOLDER_LAYOUT.md)
├── blueprint/              # 可选：includes/、overlays/、revisions/、docs/（卫星，不替代本体路径）
├── config.json             # 可选；遗忘曲线、虚拟时间（沉浸模式）；见 §9
├── prompts/                # **角色包**：系统提示词、开场白等（推荐）
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

| 顶层键 | 归属 | 必填 | 说明 |
|--------|------|------|------|
| `schema_version` | 蓝图 | 是 | 固定 **2**（双核扩展见 RFC **3**） |
| `meta` | **角色包** + 过渡期引擎字段 | 是 | 创作者子集见 §0；引擎键目标迁至 `runtime_config` |
| `slot_registry` | **蓝图** | 是 | 至少一个 `type: llm` |
| `groups` | **蓝图** | 否 | 架构图分组 |
| `includes` | **蓝图** | 否 | 卫星 JSON 拉取清单（加载时 merge/replace）；见 §2.6 |
| `expert_overlay` | **蓝图** | 否 | 专家设施指针（`routing_path`、`active_revision` 等，≤ 少量字段） |
| `runtime_config` | **蓝图** | 否 | **目标 SSOT**（`interaction_mode`、`dual_core` 等；v3 草案） |

### 2.1 `meta`（角色包 · 创作者）

| 字段 | 创作者 | 说明 |
|------|--------|------|
| `id`, `name`, `version`, `author`, `description` | 是 | 门面 |
| `personality` | 是 | 七维 0.0～1.0 |
| `relations`, `default_relation` | 是 | 用户关系 |
| `scenes` | 是 | 与 `scenes/` 合并 |
| `life_trajectory` / `life_schedule` | 可选 | 剧情/异地文案 |
| `evolution.personality_source` | 可选 | `vector` \| `profile` |

**过渡期**：上述引擎字段若仍写在 `meta`，宿主 v2 加载器可读；**目标**迁至 **`runtime_config`**。

### 2.2 运行时配置（`runtime_config` · 蓝图）

**权威清单**：[SETTINGS_REFERENCE.md](../cli/SETTINGS_REFERENCE.md) §零 **`runtime_config`**。

含 `interaction_mode`、`memory_config`、`reply_quality_anchor`、`remote_fallback_to_builtin`、`dual_core.enabled` 等。**创作者校验**（`pack validate --profile creator`）**不**检查本段。

### 2.3 系统配置（蓝图 · 槽位）

含 **`slot_registry`**、**`groups`**、各实例 **`backend` / `model` / `plugin`**，以及（目标）**`runtime_config`** 中的交互模式、记忆策略、**`dual_core.enabled`** 等。主应用 **`save_role_slot_registry`** / CLI **`oclive plugin manage`** 写回蓝图段。

### 2.4 `slot_registry`（蓝图 · 开放多实例）

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

### 2.5 `groups`（蓝图 · 可选）

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

### 2.6 `includes[]`（蓝图 · 卫星拉取）

加载角色包时，宿主按数组顺序将卫星文件合并进蓝图内存态（`oclive_validation::resolve_blueprint_includes_*`）。

| 字段 | 说明 |
|------|------|
| `path` | 相对 **`roles/{role_id}/`** 的正斜杠路径；禁止 `..` |
| `target` | 点分路径，如 `meta.personality`、`expert_routing`、`runtime_config.expert_hints`、`slot_registry.<key>` |
| `mode` | `merge`（JSON 深合并）或 `replace`（整段替换） |

缺失或非法卫星文件：**warn 并跳过**，不阻塞 `load_role`。**第 2 设施子模块**（**专家模型设施子模块**）默认文件：**`blueprint/includes/expert_routing.json`**（专家路由）；实验核 pipeline 可使用 action **`slot.expert.invoke`**（须 `dual_core` + v3 `pipeline.experimental`）。命名见 [OCLIVE_ARCHITECTURE_OVERVIEW.md](../getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md#设施模块命名规范规定)。

### 2.7 `module_relations`（仅运行时）

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

- 默认 `--host-version` 为 **本 CLI 的 `CARGO_PKG_VERSION`**；与桌面宿主版本不一致时，请显式传入 **与目标 A.I.Live 发行版一致的 semver** 再检查 `min_runtime_version`。
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

## 9. 配置文件（`config.json`）

**可选** JSON 文件，位于角色包根目录 `roles/{role_id}/config.json`。宿主在 `RoleStorage::load_role` 时读取，**不**写入 `pipeline.ocblueprint`。主要用于 **沉浸模式** 下的虚拟时钟、**艾宾浩斯记忆衰减**、**关系疏远** 等行为；未提供时使用代码内默认值（与下表「默认」列一致）。

**标准 JSON 无 `//` 注释**；示例片段仅供复制，实际文件须为合法 JSON。

### 9.1 完整示例

```json
{
  "time": {
    "speed": 5.0,
    "decay_on_jump": true
  },
  "memory": {
    "decay_halflife_days": 7.0,
    "reinforcement_factor": 0.3,
    "min_strength_for_prompt": 0.1
  },
  "relation": {
    "decay_halflife_days": 30.0,
    "estrangement_threshold": 0.3
  }
}
```

参考包：`roles/mumu/config.json`（含进阶可选键）。

### 9.2 顶层结构

| 键 | 类型 | 必填 | 说明 |
|----|------|------|------|
| `time` | object | 否 | 虚拟时钟与跳转遗忘 |
| `memory` | object | 否 | 长期记忆艾宾浩斯衰减与强化 |
| `relation` | object | 否 | 亲密值疏远与关系降级 |
| `chat_storage` | object | 否 | 聊天记录存储后端、FIFO、自动清理、记忆回放阈值 |

### 9.3 `time`（虚拟时间）

| 字段 | 类型 | 默认 | 说明 |
|------|------|------|------|
| `speed` | number | `5.0` | 现实:虚拟 **分钟** 比（`1` 现实分钟 = `speed` 虚拟分钟） |
| `decay_on_jump` | bool | `false` | 手动跳转虚拟时间后，是否对性格 delta 叠加时间遗忘 |
| `decay_per_day` | number | `1.0` | 每虚拟日性格 delta 向 0 收缩的强度（跳转/空闲衰减用） |
| `memory_decay_per_day` | number | `1.0` | 旧版记忆衰减强度（艾宾浩斯路径以 `memory.*` 为准） |

**行为摘要**：沉浸模式下宿主按锚点同步虚拟时钟；若角色包配置了 `life_schedule` 且尚无锚点，**首次**进入时虚拟起点对齐日程 **第一条** 片段的星期与 `time_start`（见 [CREATOR_LEARNING_PATH.md § 高级](CREATOR_LEARNING_PATH.md#配置记忆与关系演化)）。

### 9.4 `memory`（记忆遗忘与强化）

| 字段 | 类型 | 默认 | 说明 |
|------|------|------|------|
| `decay_halflife_days` | number | `7.0` | 记忆权重半衰期（**虚拟日**）；约 7 虚拟日后强度降至初始一半 |
| `reinforcement_factor` | number | `0.3` | 重复提及强化系数；有效半衰期 = 基础半衰期 × (1 + factor × (mention_count − 1)) |
| `min_strength_for_prompt` | number | `0.1` | 衰减后 `importance × weight` 低于此值的记忆 **不进入** 主对话 Prompt |
| `similarity_threshold` | number | `0.6` | 写入长期记忆时，与已有记忆关键词重叠度 ≥ 此值则 **强化**（`mention_count + 1`）而非新插一条 |
| `reinforced_mention_threshold` | integer | `3` | `mention_count` 达到此值后，可微幅推动性格演化（见 §9.6 与 CHANGELOG） |

**公式（宿主实现）**：剩余强度 ≈ 初始 `weight` × e^(−λ × 虚拟日龄)，λ = ln(2) / 有效半衰期。

### 9.5 `relation`（亲密值疏远）

| 字段 | 类型 | 默认 | 说明 |
|------|------|------|------|
| `decay_halflife_days` | number | `30.0` | 亲密值（好感 0～100）半衰期（**虚拟日**，自上次实际互动起算） |
| `estrangement_threshold` | number | `0.3` | 归一化亲密值（好感/100）低于此值时，关系阶段 **自动降一级**（如 Friend → Acquaintance） |
| `interaction_recovery` | number | `0.12` | 本回合实际对话时，在疏远衰减后按 `(1 + recovery)` 小幅回升，避免「一开口就被衰减抵消」 |

**行为摘要**：仅 **沉浸模式** 下、每回合对话开始前应用疏远衰减；`profile` 人格模式下可在可变性格档案「社交关系」小节记录已疏远状态。

### 9.5a `chat_storage`（聊天记录后端与回放）

| 字段 | 类型 | 默认 | 说明 |
|------|------|------|------|
| `backend` | string | `hybrid` | `hybrid` \| `file` \| `sqlite`；见 [STORAGE_BACKEND_GUIDE.md](../storage/STORAGE_BACKEND_GUIDE.md) |
| `max_messages_per_session` | integer | 宿主 500 | 单会话消息 FIFO 上限 |
| `auto_cleanup_days` | integer | — | 自动清理：保留最近 N 天 |
| `auto_cleanup_max_sessions` | integer | — | 自动清理：每角色最多 N 会话 |
| `replay_similarity_threshold` | number | `0.6` | 记忆回放去重阈值（0.1–1.0） |

### 9.6 与蓝图 / 数据库的关系

| 概念 | `config.json` | 蓝图 / DB |
|------|---------------|-----------|
| 记忆 FIFO 条数 | — | `runtime_config.memory_config` / `policy.toml` |
| 好感初值与事件 delta | — | `meta.relations` + 回合事件引擎 |
| 长期记忆内容与 `mention_count` | 控制衰减/强化参数 | SQLite `long_term_memory` |
| 虚拟时间锚点 | `time.speed` 等 | `role_runtime.virtual_time_*` 列 |

校验：`config.json` 解析失败时宿主 **warn 并回退默认**，不阻塞 `load_role`。类型定义见 `oclive_kernel_types::RolePackConfigFile`。

---

[English](../../creator-docs-en/role-pack/ROLE_PACK_SPEC.md)
