# RFC：蓝图驱动的开放后端模块系统（`pipeline.ocblueprint` v2）

| 项 | 值 |
|----|-----|
| 状态 | **Accepted**（2026-05；Q1–Q8 已关闭） |
| 目标版本 | **0.3.0**（`meta.oclive_version`） |
| 前提 | 软件尚未发布；**不兼容**旧 `manifest.json` / `settings.json` / `personality.json` |
| 实施计划 | [BLUEPRINT_V2_IMPLEMENTATION_PLAN.md](BLUEPRINT_V2_IMPLEMENTATION_PLAN.md) |
| 关联 | [BREAKING_CHANGE_PROCESS.md](BREAKING_CHANGE_PROCESS.md) |

---

## 1. 目标

**`pipeline.ocblueprint`** 为角色包 **唯一配置 SSOT**；**`slot_registry`** 支持 7 种 `type`、**全部可多实例**；内核按 **共景阶段表 + position + 合并策略** 调度；架构图 **由蓝图生成**，工具栏写配置；**会话覆盖** 仅内存、不落盘。

---

## 2. 角色包目录

```text
roles/{role_id}/
├── pipeline.ocblueprint
├── prompts/          # system.md, greeting.md, reply_quality_anchor.md
├── memories/         # 可选
├── assets/           # 可选
├── scenes/           # 可选
└── plugins/          # 可选
```

可选保留：`core_personality.txt`（profile 模式）、`knowledge/`、`ui.json`（不进蓝图 SSOT）。

---

## 3. `meta`（门面 + 人格 + 引擎段）

### 3.1 `meta.personality`（Q1：Accepted）

**保持现网七维**，与 `PersonalityDefaults` / `PersonalityVector` 一致，**不**引入 VAD/Big Five。

| 维度键 | 含义 |
|--------|------|
| `stubbornness` | 倔强 |
| `clinginess` | 黏人 |
| `sensitivity` | 敏感 |
| `assertiveness` | 强势 |
| `forgiveness` | 宽容 |
| `talkativeness` | 话多 |
| `warmth` | 温暖 |

- 取值：**0.0～1.0**（与现校验一致）。
- JSON 形态（二选一，校验均接受）：
  - **对象**：上表七键（推荐，与 Rust struct 一致）；
  - **数组**：长度 **7**，顺序与上表相同（便于从旧 `default_personality` 迁移）。
- **数据搬家**：原 `manifest.json` → `default_personality` 迁入 `meta.personality`；**`process_message`、情绪分析、关系、Prompt 注入、complex_emotion 用 valence/dominance 推导、演化逻辑均不变**。
- VAD/Big Five：**未来独立 RFC**，不在本重构范围。

### 3.2 `meta.relations`（Q2）

结构 **等同于** 原 `manifest.user_relations`（仅改名）。`default_relation` 保留在 `meta`。

### 3.3 其他 `meta` 字段（Q5）

自原 manifest/settings 合并：`scenes`、`evolution`、`memory_config`、`interaction_mode`、`ollama_model`、`life_schedule`、`identity_binding`、`author_profile` 等（完整表见实施计划附录 A）。

---

## 4. `slot_registry`

### 4.1 `type` 枚举（7）

`memory` | `emotion` | `event` | `prompt` | `llm` | `agent` | `complex_emotion`

### 4.2 多实例与合并（Q3–Q4、Q8）

| `type` | 合并策略 |
|--------|----------|
| `memory` | 串行 → 合并去重 → 统一排序 |
| `emotion` | 串行 → **last-wins** |
| `event` | 串行 → 事件合并去重 |
| `prompt` | 串行 → **last-wins** |
| `llm` | 串行 → **last-wins**（中间实例仍调用、打日志） |
| `agent` | 单槽 `plugins[]` 合并工具集；**多 agent 槽位** → **合并全部 tools 到同一调度器**（Q3） |
| `complex_emotion` | 串行 → **last-wins**（Q8） |

**必填**：≥1 个 `llm`。`position` 驱动同 type 内顺序；跨 type 顺序由 §5 阶段表约束。

### 4.3 `backend`

`builtin` | `builtin_v2` | `remote` | `directory` | `ollama`（llm）| `local`（memory）

`complex_emotion`：**builtin / remote / directory**；目录插件 `provides` 可含 `"complex_emotion"`。

### 4.4 `module_relations`

**派生视图**（非 SSOT）；由 `slot_registry` 中 directory 槽自动生成。

---

## 5. 共景编排（阶段表 — P4 前不改代码）

| 序 | 阶段 | `type` |
|----|------|--------|
| 1 | 用户情绪 | `emotion` |
| 2 | 复杂情感 | `complex_emotion` |
| 3 | 人格/知识 | （非槽） |
| 4 | 事件 | `event` |
| 5 | 记忆 | `memory` |
| 6 | 好感/关系 | （非槽） |
| 7 | Prompt | `prompt` |
| 8 | LLM | `llm` |
| 9 | Agent | `agent` |

**P1–P3**：不修改 `co_present` 执行顺序，仅 Schema/加载/解析。

---

## 6. 会话级后端覆盖（Accepted）

### 6.1 语义

- 覆盖写入 **`SessionSlotRegistry` 内存副本**（按 `srid`），**不**写入 `pipeline.ocblueprint`。
- 会话结束丢弃；重启从蓝图重载。
- **替代** 现 `set_session_plugin_backend`：**行为语义保留**（仍不修改角色包文件）；API 改为按 **`slot_registry` 实例键**（如 `memory_short_term`）覆盖 `backend` / `plugin` / `plugins` 等字段。

### 6.2 架构图交互

1. 用户点击槽位节点 → 弹出 **后端切换面板**（当前 backend + 合法枚举/插件列表）。
2. 选择后点 **「应用」** → 更新会话副本 → 节点显示 **「本次覆盖」**（小标签或虚线边框）。
3. 内核下次解析该实例时使用覆盖值。
4. 节点提供 **「重置为默认」** → 清除该实例的会话覆盖项。
5. `get_role_info` 继续暴露 effective 快照 + 来源（pack vs session），字段对齐新 registry 模型。

---

## 7. 架构图

- 节点/边：蓝图 `slot_registry` + 派生 `module_relations` + 会话覆盖视图。
- **禁止**手拖连线（预研 `useArchitectureGraphConnections` 主路径在 P5 移除）。
- 工具栏：**添加槽位** / **添加插件** / **多对一** → 写蓝图文件 → 刷新图。

---

## 8. 废弃与迁移（Q6–Q7）

- 旧 CLI 蓝图 **`steps[]` / `entry`**：**废弃**；`oclive-cli blueprint validate` **仅** v2。
- 仓库 `roles/*`：**P6 与 P2 同迭代** 迁移为 v2；CI 黄金包切换 v2。

---

## 9. 已关闭待决（记录）

| ID | 决策 |
|----|------|
| Q1 | 七维 `meta.personality`，不改 VAD/Big Five |
| Q2 | `meta.relations` ≡ `user_relations` |
| Q3 | 多 agent 槽 → 合并 tools |
| Q4 | 多 llm 串行全调用，last-wins |
| Q5 | evolution 等进 meta；`core_personality.txt` 可选保留 |
| Q6 | 废弃 steps 蓝图 |
| Q7 | P6∥P2 迁移 roles |
| Q8 | complex_emotion last-wins |

---

## 10. 审阅

- [x] 产品关闭 Q1–Q8  
- [x] RFC Accepted  
- [ ] 维护者确认 [BLUEPRINT_V2_IMPLEMENTATION_PLAN.md](BLUEPRINT_V2_IMPLEMENTATION_PLAN.md) 后开工  
