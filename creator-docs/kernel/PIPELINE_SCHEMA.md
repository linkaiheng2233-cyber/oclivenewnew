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
| `steps` | `array` | 是 | **有序** 步骤列表；每项至少含 `action`。 |
| `onFailure` | `string` | 否 | 某步返回错误时的策略，见下文。省略时等价于 **`HALT`**。 |

可选扩展字段（解析器当前忽略未知键，便于后续演进）：

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `description` | `string` | 否 | 人类可读说明。 |

## `steps[]` 元素

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `action` | `string` | 是 | 原子操作标识，须为内核 **白名单** 之一（与 `pipeline_actions` 中 `pub async fn` 的 snake_case 名一致）。 |
| `id` | `string` | 否 | 步骤别名，便于日志。 |
| `description` | `string` | 否 | 步骤说明。 |

### 步骤数量

单文件 `steps.length` 不得超过实现定义的上限（当前 **64**），防止异常大包。

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

## 当前允许的原子 `action` 列表（v1.0 白名单）

与 `crates/oclive_kernel_runtime/src/domain/chat_engine/pipeline_loader.rs` 中 `ALLOWED_PIPELINE_ACTIONS` 保持一致（不含 `validate_scene` 于可执行蓝图内；该符号仍保留在实现内部供其它路径使用）：

- `init_turn`
- `ensure_role_runtime`
- `load_role`
- `seed_interaction_mode`
- `log_effective_plugin_backends`
- `resolve_plugins`
- `resolve_main_llm_model`
- `run_agent`
- `set_user_presence_scene`
- `load_presence_routing`
- `analyze_emotion_user`

## 最小示例（片段）

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

完整官方示例见仓库 **`examples/blueprints/simple_companion.ocblueprint`**。
