# 角色包创作者学习路径

按 **时间盒** 划分，便于第一次上手与进阶排期。权威格式仍以 **[ROLE_PACK_SPEC.md](ROLE_PACK_SPEC.md)** 与 **[distros/chat-pro/roles/README_MANIFEST.md](../../distros/chat-pro/roles/README_MANIFEST.md)** 为准；磁盘校验命令见 **`oclive-cli`**（仓库根执行 **`cargo run -p oclive-cli -- pack …`**）。

---

## 从 v1 迁移到 v2（约 10 分钟）

已有 **`manifest.json` + `settings.json`** 的包，请先阅读 **[V1_TO_V2_MIGRATION.md](V1_TO_V2_MIGRATION.md)**：`pack migrate-to-blueprint` → 默认 `pack validate`（v2）→ 主应用试聊。

## 从 v2 升级到 v3（约 10 分钟）

需要 **`runtime_config`** 或可选 **双核** 时，阅读 **[V2_TO_V3_MIGRATION.md](V2_TO_V3_MIGRATION.md)**：改 `schema_version: 3` → 添加 `runtime_config` → `pack validate` → `oclive doctor`（v3 专项检查）。**双核默认关**时行为与 v2 一致。

---

## 入门（约 30 分钟）

| 步骤 | 做什么 | 读什么 / 做什么 |
|------|--------|------------------|
| 1 | 理解角色包长什么样 | [ROLE_PACK_SPEC.md](ROLE_PACK_SPEC.md) **§1 目录结构** |
| 2 | 生成第一个可校验的最小包 | `cargo run -p oclive-cli -- pack create -o <输出父目录> --id my_first_role --format-blueprint-v2`（写入 `pipeline.ocblueprint`；见 [../cli/OCLIVE_CLI_GUIDE.md](../cli/OCLIVE_CLI_GUIDE.md)） |
| 3 | 在编写器中打开 | **oclive-pack-editor** 编辑 v2 蓝图或 legacy 双文件（分工见 [CREATOR_WORKFLOW.md](../getting-started/CREATOR_WORKFLOW.md)） |
| 4 | 改门面信息 | v2：仅编辑 **角色包字段**（`meta` 中 `name`、`personality`、`relations`、`reply_quality_anchor` 与 **`prompts/`**）；勿改 `slot_registry`（见上表） |

**验收**：`cargo run -p oclive-cli -- pack validate <角色根>` **默认 v2** 通过；维护中的旧包用 `--profile legacy`。

### 权限边界（入门必读）

| 你只需关心 | 不必关心（交给蓝图 / 管理员） |
|------------|-------------------------------|
| `meta` 门面、**`personality`**、**`relations`**、**`reply_quality_anchor`** | **`slot_registry`**、**`backend`**、**`model`** |
| **`prompts/`** 系统提示词与开场白 | **`interaction_mode`**、**`memory_config`**、远程策略 |
| **`scenes/`** 场景文案 | **`groups`**、双核 **`dual_core.enabled`** |

详见 **[ROLE_PACK_SPEC.md §0](ROLE_PACK_SPEC.md#0-角色包-vs-蓝图职责)** · **[handoff/ROLE_PACK_BOUNDARY.md](../../handoff/ROLE_PACK_BOUNDARY.md)**。进阶阶段再学槽位时，请使用 **蓝图 / 高级** 视图或 **`oclive plugin manage`**。

---

## 进阶（约 1–2 小时）

| 主题 | 读什么 |
|------|--------|
| **七维人格向量** | [README_MANIFEST § default_personality](../../distros/chat-pro/roles/README_MANIFEST.md) · [docs/personality-archive-notes.md](../../docs/personality-archive-notes.md) |
| **系统提示词与开场白** | 角色包内 `prompts/` 自管素材与引擎组装关系见 ROLE_PACK_SPEC 与 [WORLDVIEW_KNOWLEDGE.md](WORLDVIEW_KNOWLEDGE.md)；主对话 Prompt 由 **`slot_registry` 中 `type: prompt`** 实例与内置策略决定 |
| **槽位与第 1–6 模块** | [OCLIVE_ARCHITECTURE_OVERVIEW.md](../getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md) · [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md) · [SETTINGS_REFERENCE.md](../cli/SETTINGS_REFERENCE.md) |

**验收**：能在 **`pipeline.ocblueprint` → `slot_registry`** 中为各 `type` 配置 `backend`（`builtin` / `remote` / `directory` 等），并理解 directory 槽的 `plugin` / `plugins` 与目录插件 manifest `id` 的对应关系。

| **团队协作** | `oclive collab init/status/pull/push` · [ROLE_PACK_SPEC.md §7](ROLE_PACK_SPEC.md#7-团队协作oclive-collab) |

---

## 高级（约半天）

| 主题 | 读什么 / 注意 |
|------|----------------|
| **`reply_quality_anchor` 与回复风格** | [README_MANIFEST](../../distros/chat-pro/roles/README_MANIFEST.md) · ROLE_PACK_SPEC 中 `settings` 合并表；用于锚定回复质量/风格相关配置（以校验 crate 与宿主加载为准） |
| **`pipeline.ocblueprint` v2（推荐 SSOT）** | [ROLE_PACK_SPEC.md](ROLE_PACK_SPEC.md) · [handoff/BLUEPRINT_V2_IMPLEMENTATION_PLAN.md](../../handoff/BLUEPRINT_V2_IMPLEMENTATION_PLAN.md)。**桌面编排**以 **`process_message` → `co_present`** 为准，**不**再读蓝图 `steps[]`（见 [AGENTS.md](../../AGENTS.md)）。主应用架构图可 **`save_role_slot_registry`** 写回 `slot_registry` |
| **双核双态（实验 pipeline）** | [DEVELOPER_GUIDE.md](../dual-core/DEVELOPER_GUIDE.md) · [METHOD_REGISTRY.md](../dual-core/METHOD_REGISTRY.md) · `oclive init --dual-core`；创作者包勿默认 `enabled: true` |
| **配置专家模型设施子模块**（专家路由 · 须在主应用操作） | 见下「配置专家模型」；**勿**在编写器 `.oclexpert` 实验页新建路由 |
| **校验** | 默认 v2：`pack validate <角色根>`（**完整包**：含 `slot_registry`、引擎字段）；**创作者子集**：`pack validate --profile creator <包根>`（只校验 §2 角色包字段 + `prompts/`，**不**校验蓝图槽位）。**勿**用 `distros/chat-pro/roles/mumu` 测 creator — mumu 是带完整蓝图的示例包，对该 profile 会报错属正常。旧包 `--profile legacy`；无头交付 `--profile robot-soul`（须 legacy 形状，见 ROLE_PACK_SPEC §6） |
| **编写器侧 wasm 校验** | [oclive-pack-editor](https://github.com/linkaiheng2233-cyber/oclive-pack-editor) 的 `wasm:build` 与「运行全部检查」 |

**验收**：`pack validate` 无错误；理解「哪些键会进宿主合并校验、哪些仅作者自管」。

### 配置专家模型（主应用）

1. 用 **oclive-pack-editor** 或 `oclive pack` 准备好 v2 角色包并放入 `distros/chat-pro/roles/{id}/`。
2. 打开 **A.I.Live（本仓库桌面端）** → **Ctrl+Shift+F** → **插件与后端管理** → **架构图**。
3. 在 **专家模型设施**（架构图节点 / 齿轮）中编辑触发条件与步骤，保存为 **`blueprint/includes/expert_routing.json`**（必要时在蓝图 `includes[]` 中引用）。
4. 若之后在编写器中修改人设并保存，编写器会**保留** `includes` / `groups` 等扩展字段；专家路由文件本身不会被编写器删除。
5. 编写器高级页中的 **`expert/default.oclexpert`** 为遗留实验格式，**不是**运行时专家路由。

参考：[OCLIVE_ARCHITECTURE_OVERVIEW.md](../getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md) · [BLUEPRINT_FOLDER_LAYOUT.md](../../handoff/BLUEPRINT_FOLDER_LAYOUT.md) · [ROLE_PACK_SPEC.md](ROLE_PACK_SPEC.md) §2.6 · 编写器 [ROLE_PACK_EDITOR.md](https://github.com/linkaiheng2233-cyber/oclive-pack-editor/blob/main/creator-docs/ROLE_PACK_EDITOR.md)。

### 配置记忆与关系演化

沉浸模式下，宿主用 **`config.json`**（非蓝图）驱动「像人一样会忘、久不联系会疏远」的行为。完整字段见 **[ROLE_PACK_SPEC.md §9](ROLE_PACK_SPEC.md#9-配置文件configjson)**。

| 机制 | 原理（简述） | 主要配置 |
|------|--------------|----------|
| **记忆衰减** | 长期记忆权重按虚拟日龄 **指数衰减**（艾宾浩斯）；太弱的记忆不进 Prompt | `memory.decay_halflife_days`、`memory.min_strength_for_prompt` |
| **记忆强化** | 用户反复提同一话题时，相似记忆 `mention_count` 增加，**有效半衰期变长** | `memory.reinforcement_factor`、`memory.similarity_threshold` |
| **亲密值疏远** | 距上次互动越久，好感按虚拟日衰减；过低则关系阶段降一级 | `relation.decay_halflife_days`、`relation.estrangement_threshold` |
| **虚拟时间** | 现实 1 分钟 = 虚拟 `speed` 分钟；可选跳转后叠加遗忘 | `time.speed`、`time.decay_on_jump` |

**推荐起步配置**（复制到 `distros/chat-pro/roles/{id}/config.json` 后按需微调）：

```json
{
  "time": { "speed": 5.0, "decay_on_jump": true },
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

**调参提示**：

- 想让角色「忘得慢、记得牢」→ 增大 `memory.decay_halflife_days` 或 `reinforcement_factor`。
- 想让久不聊天明显变冷淡 → 减小 `relation.decay_halflife_days` 或提高 `estrangement_threshold`（更早触发降级）。
- 日程驱动的角色：在 `meta.life_schedule` 配好片段后，首次沉浸时虚拟时间会从 **日程第一条** 的起点开始（无需手填时间戳）。

**验收**：改完 `config.json` 后主应用沉浸模式多轮对话；重复同一话题后 DB `long_term_memory.mention_count` 上升；长时间不互动后好感与关系阶段可见下降。

---

## 发布

| 步骤 | 命令 / 文档 |
|------|-------------|
| **打出 `.oclivepack`** | `cargo run -p oclive-cli -- pack publish <角色根目录> -o <输出路径>`（默认生成 `<id>-<version>.oclivepack`；见 CLI 指南） |
| **社区索引 JSON** | [ROLE_PACK_INDEX.md](ROLE_PACK_INDEX.md) · 市场/站点侧流程见 [../roadmap/MARKET_LAUNCHER_INTEGRATION.md](../roadmap/MARKET_LAUNCHER_INTEGRATION.md) |
| **与主程序版本对齐** | [COMPATIBILITY.md](../COMPATIBILITY.md) · `manifest.min_runtime_version` |

---

## 学完以后

- 维护包版本与 `schema_version`：[PACK_VERSIONING.md](PACK_VERSIONING.md)  
- 深度校验路线：[EDITOR_VALIDATION_ROADMAP.md](EDITOR_VALIDATION_ROADMAP.md)
