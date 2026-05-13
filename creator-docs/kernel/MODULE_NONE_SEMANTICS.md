# 六模块 `none` Provider 语义规范 v1.0（冻结标准）

> 状态：**Draft → 建议冻结 v1.0**  
> 目标：当 Profile/配置将某模块设为 `backend: "none"` 时，内核行为必须 **安全、确定、可预期**，并且不会因空返回/异常导致 UI 或内核崩溃。

---

## 0. 总则（硬性）

当六模块之一设为 `backend: "none"` 时：

- **模块接口不可移除**：调用路径仍存在，只是由“空 Provider”接管。
- **不做任何实际工作**：不联网、不读写磁盘、不写 DB（除非规范明确允许的“审计/统计元数据”，且不得包含敏感内容）。
- **必须返回确定值**：不得返回“空字符串导致前端渲染异常”的结果；不得回显原始用户输入作为回复；不得 panic。
- **错误必须产品化**：需要失败时返回明确、面向用户的提示（可本地化），并携带可定位的错误码（若实现侧已有错误码体系）。

本规范覆盖六模块（固定存在）：

- Memory
- Emotion
- Event
- Prompt
- LLM
- Complex Emotion

> 注：Agent（第七模块）的 `none` 语义见 **§7**（与六模块并列说明，仍属同一 `backend: "none"` 家族）。

---

## 1) Memory（记忆）为 `none`

### 行为

- **读取相关记忆**：返回空列表（或等价“无结果”），并保证排序/截断逻辑可继续运行。
- **写入短期/长期记忆**：不写入 DB（本轮可继续完成对话，但不会产生记忆副作用）。

### 降级要求

- Prompt 侧若依赖记忆块，应按“无记忆”处理，不得因为缺少记忆导致格式化失败。

---

## 2) Emotion（用户情绪）为 `none`

### 行为

必须返回“强中性”的七维情绪结果（与现有 `EmotionResult` 形状一致）：

- `joy=0`
- `sadness=0`
- `anger=0`
- `fear=0`
- `surprise=0`
- `disgust=0`
- `neutral=1`

### 约束

- 不得抛错中断主流程（情绪只是辅助信号；`none` 应是“禁用但继续运行”）。

---

## 3) Event（事件影响估计）为 `none`

### 行为

必须返回“无事件、无影响”的估计结果：

- `event_type = Ignore`
- `impact_factor = 0.0`
- `confidence = 0.0`（或实现侧约定的最小可信度）

### 降级要求

- 人格演化/好感变化若依赖事件影响，应按“无事件”继续。

---

## 4) Prompt（Prompt 组装）为 `none`

### 行为

Prompt 模块 `none` 的核心目标是：**仍能把用户输入安全地喂给 LLM（如果 LLM 可用）**，同时不给对话编排制造不确定性。

最低要求：

- `build_prompt`：返回一个最小 prompt（不得为空），至少包含：
  - 固定 system 前缀：声明“当前为最小模式 / Prompt 模块未启用”
  - 用户输入（作为 user message 内容，而不是把用户输入当作 assistant 输出）

示例（概念）：

- system：`"[oclive] prompt module disabled (backend=none). Running minimal prompt."`
- user：`<user_message>`

### 与 LLM `none` 的组合

- 若 Prompt=`none` 且 LLM=`none`：必须以“对话引擎不可用”结束本轮，并返回明确提示（见 §5）。

---

## 5) LLM（主对话生成）为 `none`

### 行为

LLM 为 `none` 时，主对话不应继续生成。

硬性要求：

- `send_message` 必须返回**明确的失败/不可用提示**，例如：
  - `"当前对话引擎不可用（LLM 未启用）。请在 Profile 中启用 LLM 或选择可用后端。"`
- 不得：
  - 返回空字符串
  - 返回原始用户输入
  - 返回“随机占位”导致用户误以为是角色回复

### 建议

- 若实现侧有 `reply_is_fallback` / `presence_mode` 等字段，应标记为 fallback/disabled，便于前端做醒目展示。

---

## 6) Complex Emotion（复杂情感复盘）为 `none`

### 行为

复杂情感 `none` 时，必须：

- 不计算复盘/标签/叙事提示
- 输出等价于“无复盘”的结果：
  - `narrative_hint = ""`（或实现侧约定 `null` / `None`）
  - `labels = []`
  - `confidence/intensity/dissonance_score = 0`
  - `source` 明确标注为 `"none"`

### 降级要求

- 下一轮 Prompt 注入复杂情感字段时应自动跳过，不得注入空结构导致模板异常。

---

## 7) Agent（第七模块）为 `none`

### 行为

当 `plugin_backends.agent = none`（或 Profile / 会话覆盖等价配置）时，由进程内 **`DisabledAgentProvider`**（`crates/oclive_kernel_runtime/src/domain/agent.rs`）接管 **`AgentProvider::process`**：

- **`handled`**：必须为 **`false`**，以便主对话管线**不短路**：后续仍按 **`co_present` / LLM** 等路径处理用户消息（与「整轮对话失败」的 LLM `none` 不同）。
- **`reply`**：必须为**固定中文提示**（实现常量 **`AGENT_BACKEND_NONE_REPLY`**），用于观测、调试与任何直接读取 `AgentOutput` 的调用方。
- **禁止**：`reply` 为空字符串、为原始用户输入、或任何易与角色回复混淆的占位文本。
- **副作用**：不得发起 MCP 工具调用、不得访问 remote/directory Agent HTTP、不得执行 ReAct 循环。

### 与 `NoopAgent` 的区别

在 **`kernel-agent` 特性关闭**等场景下，builtin 槽可能仍装配 **`NoopAgent`**（历史行为：`reply` 为空且 `handled = false`）。**`AgentBackend::None`** 是**显式路由枚举**，须返回**非空**的 `AGENT_BACKEND_NONE_REPLY`，以满足本节的可预期性与审计需求。

---

## 8. 安全原则（禁止行为清单）

`none` Provider **禁止**：

- **隐私泄露**：把用户输入或历史对话作为“回复文本”回显（尤其是把 user→assistant 角色弄反）。
- **不确定输出**：返回空字符串、`null` 但上游未做兼容，导致 UI 崩溃。
- **未捕获异常**：panic、unwrap 导致内核崩溃。
- **暗中执行副作用**：联网、执行 shell、写文件、写 DB（除非规范明确允许的安全元数据）。

---
