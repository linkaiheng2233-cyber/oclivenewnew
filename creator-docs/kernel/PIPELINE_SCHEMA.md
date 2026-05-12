# 可编程对话流水线 · 蓝图 JSON（`*.ocblueprint`）

内核在角色目录下可选读取 **`pipeline.ocblueprint`**（与 `manifest.json` 同级），用于描述本角色在 `process_message` 中 **一段** 原子步骤顺序。首版与 `oclive_kernel_runtime` 中的 `pipeline_actions` 原子标识对齐。

## 文件位置与发现

- **运行时路径**：`{roles_dir}/{manifest_role_id}/pipeline.ocblueprint`
- **仓库示例**：`examples/blueprints/*.ocblueprint`（文件名可任意，便于分发；拷入角色目录后须命名为 `pipeline.ocblueprint`）

若文件不存在，内核使用内置默认线性入口序列（与未提供蓝图时一致）。若存在但校验失败，内核记录告警并同样回退到默认序列。

## 顶层字段

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `schemaVersion` | `string` | 是 | 蓝图格式版本；当前仅支持 **`"1.0"`**。 |
| `name` | `string` | 是 | 蓝图逻辑名称（日志与排障）。 |
| `steps` | `array` | 是 | **有序** 根步骤列表；每项为线性原子、**`branch`** 或 **`parallel`**（见下文），根数组长度上限 **64**。 |
| `onFailure` | `string` | 否 | 某步返回错误时的策略，见下文。省略时等价于 **`HALT`**。 |

可选扩展字段（解析器当前忽略未知键，便于后续演进）：

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `description` | `string` | 否 | 人类可读说明。 |

## `steps[]` 元素（线性 / `branch` / `parallel`）

三者 **互斥**：同一步骤不得同时带 `branch` 与 `parallel`；带 `branch` 或 `parallel` 时 **不得** 再设非空 `action`。

### 线性步骤

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `action` | `string` | 是 | 原子操作标识，须为白名单之一。 |
| `id` | `string` | 否 | 步骤别名，便于日志。 |
| `description` | `string` | 否 | 步骤说明。 |

### 条件分支 `branch`

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `branch` | `object` | 是 | 见下表。 |

`branch` 对象：

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `predicate` | `object` | 是 | 受限谓词（见「谓词 `predicate`」）。 |
| `onTrue` | `array` | 否 | 谓词为真时递归执行的子步骤列表（默认可空数组）。 |
| `onFalse` | `array` | 否 | 谓词为假时递归执行的子步骤列表。 |

子步骤的结构与根 `steps` 元素相同，可继续嵌套 `branch`（嵌套深度上限见实现常量 `MAX_PIPELINE_BRANCH_DEPTH`）。整棵树节点总数上限见 `MAX_PIPELINE_TREE_NODES`。

### 谓词 `predicate`（`type` + 字段）

JSON 使用 **`type` 判别字段**（camelCase），与 `PipelinePredicate` 一致：

| `type` | 附加字段 | 语义 |
|--------|-----------|------|
| `agentHandled` | 无 | `ctx.flags.agent_handled == true`（通常在 `run_agent` 之后才有意义）。 |
| `sceneIdEquals` | `sceneId`（string） | 当前 `effective_scene_id` 与该字符串完全相等。 |
| `emotionDominant` | `emotion`（string） | 用户七维结果中 **数值最大的一维** 名称与该字符串匹配（忽略大小写），如 `sadness`；亦可与离散主导标签的蛇形名比较（如 `sad`）。缺少情绪结果时视为假。 |
| `emotionAbove` | `emotion`（string）、`min`（number） | 指定七维维度（如 `sadness`）的数值 **严格大于** `min`；无 `analyze_emotion_user` 结果时为假。 |

### 受限并行 `parallel`

`parallel` 为 **JSON 数组**，每个元素是一个 **arm**（该 arm 内为顺序执行的子步骤列表）。所有 arm 由 `tokio::join!` / `try_join_all` **同时启动**；因共享 `TurnContext`，运行时将其置于 `Arc<Mutex<…>>` 中，各原子仍串行拿锁执行（主要保证 **join 调度语义** 与错误聚合；极短只读原子可有部分重叠）。

**硬限制（加载期拒绝）：**

- 任一 arm 的子树中 **不得** 出现 `branch`（避免在并行臂内做条件分裂；后续版本可放宽）。
- 任一 arm 的每个**线性叶子** `action` 必须为 **`READ_ONLY`**（见 `pipeline_actions::ACTION_IO_TYPES`）。出现 **`WRITE`** 原子时返回 `ParallelContainsWrite`。
- 允许 **嵌套** `parallel`（嵌套 arm 仍须满足上述只读约束）。

**失败语义**：`try_join_all` 中任一 arm 返回 `Err` 时，整段 `parallel` 视为失败，并遵循蓝图根级 `onFailure`（`HALT` / `DEGRADE`）。

### 步骤数量

根 `steps.length` 不得超过 **64**；整棵蓝图树（含所有 `onTrue` / `onFalse` / `parallel` 内子步骤）节点总数不得超过 **`MAX_PIPELINE_TREE_NODES`**（当前实现为 **200**）。

## `onFailure` 策略

| 取值 | 语义 |
|------|------|
| **`HALT`** | 某步失败后 **立即** 结束蓝图执行；`process_message` 对入口蓝图会 **降级** 再跑一遍与历史版本一致的默认入口线性序列，保证对话不中断。 |
| **`DEGRADE`** | 记录告警并 **跳过** 当前失败步，继续执行后续步骤。 |

大小写不敏感（实现归一化为大写枚举）。

## 与「分析情绪 → 组装提示词 → 生成回复」的关系（概念）

产品文档中常把一轮对话概括为三步。当前 v0 原子仍按 **编排实际顺序** 拆分（例如入口段在 `run_agent` 之前完成插件与模型解析；用户情绪分析在 Agent 早退分支内等）。官方示例 **`simple_companion.ocblueprint`** 展示的是 **与默认一致的入口八步**（从 `init_turn` 到 `run_agent`），与「极简陪伴」默认路径对齐；后续若增加 `assemble_prompt` 等独立原子，再在 Schema 与白名单中扩展即可。

## 入口蓝图与 `validate_scene`

`process_message` 在加载蓝图 **之前** 已执行 `validate_scene`（含场景列表与 `effective_scene_id` 注入）。因此 **`pipeline.ocblueprint` 的 `steps` 不应再包含 `validate_scene`**，否则会在同一轮内重复校验。加载器会 **拒绝** 含该 `action` 的蓝图文件并回退到默认序列。

## 已知原子 `action` 与 I/O 类型

与 `pipeline_actions::ACTION_IO_TYPES` 及 `ALLOWED_PIPELINE_BLUEPRINT_ACTIONS` 同步（扩展时须同时改代码与下表）。

| `action` | I/O | 说明 |
|----------|-----|------|
| `init_turn` | WRITE | 清除本轮生成取消标志。 |
| `ensure_role_runtime` | WRITE | 确保会话命名空间 DB 表。 |
| `load_role` | WRITE | 加载角色 `Arc`。 |
| `seed_interaction_mode` | WRITE | 播种交互模式。 |
| `log_effective_plugin_backends` | READ_ONLY | 调试日志（不写 DB）。 |
| `resolve_plugins` | WRITE | 解析 `ResolvedRolePlugins` 写入 `ctx`。 |
| `resolve_main_llm_model` | WRITE | 解析主对话模型名写入 `ctx`。 |
| `run_agent` | WRITE | 调用 Agent / LLM。 |
| `set_user_presence_scene` | WRITE | 写用户 presence 场景。 |
| `load_presence_routing` | WRITE | 读 DB 并写 `ctx.presence` / `preflight_ms`。 |
| `analyze_emotion_user` | WRITE | 用户句情绪分析写入 `ctx`。 |
| `memory_retrieve_short_term` | READ_ONLY | 占位：短期记忆检索（示例 / 并行烟测）。 |
| `memory_retrieve_long_term` | READ_ONLY | 占位：长期记忆检索。 |
| `assemble_prompt` | READ_ONLY | 占位：Prompt 组装（真实组装仍在共景路径）。 |
| `generate_response` | WRITE | 与 `run_agent` 等价，便于蓝图命名。 |
| `expert_empathy_touch` | WRITE | 占位：高共情专家触发器（审计日志，可接专家图）。 |

## 官方蓝图示例（`examples/blueprints/`）

| 文件 | 说明 |
|------|------|
| [`simple_companion.ocblueprint`](../../examples/blueprints/simple_companion.ocblueprint) | 与默认入口八步一致。 |
| [`minimal_chat.ocblueprint`](../../examples/blueprints/minimal_chat.ocblueprint) | 前置 + `assemble_prompt` + `generate_response`。 |
| [`memory_heavy.ocblueprint`](../../examples/blueprints/memory_heavy.ocblueprint) | 只读并行「双路记忆检索」占位 + 组装 + 生成。 |
| [`deep_empathy.ocblueprint`](../../examples/blueprints/deep_empathy.ocblueprint) | `emotionAbove`（sadness>0.7）分支 + `expert_empathy_touch`。 |

部署到角色包时请将所选文件 **复制为** 角色目录下的 `pipeline.ocblueprint`。

## 最小示例（线性入口片段）

```json
{
  "schemaVersion": "1.0",
  "name": "example_entry",
  "onFailure": "HALT",
  "steps": [
    { "action": "init_turn", "id": "gate" },
    { "action": "ensure_role_runtime" },
    { "action": "load_role" },
    { "action": "seed_interaction_mode" },
    { "action": "log_effective_plugin_backends" },
    { "action": "resolve_plugins" },
    { "action": "resolve_main_llm_model" },
    { "action": "run_agent" }
  ]
}
```

### `branch` 示例（按场景拆分加载路径）

```json
{
  "schemaVersion": "1.0",
  "name": "branch_scene",
  "onFailure": "HALT",
  "steps": [
    { "action": "init_turn" },
    { "action": "ensure_role_runtime" },
    {
      "id": "scene_gate",
      "branch": {
        "predicate": { "type": "sceneIdEquals", "sceneId": "default" },
        "onTrue": [
          { "action": "load_role" },
          { "action": "seed_interaction_mode" }
        ],
        "onFalse": [{ "action": "load_role" }]
      }
    },
    { "action": "log_effective_plugin_backends" }
  ]
}
```

完整官方示例见仓库 **`examples/blueprints/simple_companion.ocblueprint`** 及上表「官方蓝图示例」。
