# Remote 插件协议（宿主 ↔ HTTP 侧车）— 完整说明

**实现状态**：宿主在 `src-tauri/src/infrastructure/remote_plugin/` 中实现 **HTTP POST + JSON-RPC 2.0** 客户端。角色包将子系统设为 `remote` 且配置环境变量后，请求发往侧车；**网络错误、HTTP 非 2xx、JSON-RPC `error`、或 result 无法反序列化**时，宿主**回退内置实现**并写日志（`target: oclive_plugin`），对话一般仍可继续。

---

## 1. 传输层

### 1.1 URL 与 HTTP

- 请求：**`POST`** 到环境变量给出的**完整 URL**（可含路径），例如 `http://127.0.0.1:8765/rpc`。  
- **`Content-Type`**：宿主使用 `application/json` 发送 JSON 请求体。  
- **协议头**：宿主固定附带  
  - `x-oclive-remote-protocol: oclive-remote-jsonrpc-v1`  
  - `x-oclive-client-version: <宿主版本>`（例如 `0.2.0`）  
- **认证**：若设置 `OCLIVE_REMOTE_PLUGIN_TOKEN` / `OCLIVE_REMOTE_LLM_TOKEN`，宿主在请求头加入 **`Authorization: Bearer <token>`**。  
- **超时**：由 `OCLIVE_REMOTE_PLUGIN_TIMEOUT_MS`（默认 8000 ms）与 `OCLIVE_REMOTE_LLM_TIMEOUT_MS`（默认 120000 ms）控制（宿主侧有上下限钳制，见源码 `config.rs`）。

### 1.2 JSON-RPC 2.0 请求（宿主发出）

每条 HTTP 请求体**一个** JSON 对象：

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "memory.rank",
  "params": { }
}
```

| 字段 | 说明 |
|------|------|
| `jsonrpc` | 固定字符串 `"2.0"` |
| `id` | 正整数（宿主递增），侧车应在响应中原样返回 |
| `method` | 本文档第 2 节中的方法名 |
| `params` | 对象；具体字段见各方法 |

### 1.3 JSON-RPC 2.0 成功响应（侧车返回）

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": { }
}
```

`result` 的形状由**方法**约定；宿主既支持 **`result` 为对象且含约定字段**，也支持少数方法下 **`result` 为字符串**（见各方法说明）。

### 1.4 JSON-RPC 2.0 错误响应

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32601,
    "message": "Method not found"
  }
}
```

宿主在收到 `error` 时按**失败**处理并回退内置。

#### 推荐错误码约定（产品化）

宿主会把 `error.code`/`error.message`/`error.data` 直接带入日志；建议侧车统一使用下列码（兼容 JSON-RPC 标准码）：

| code | name | 语义 |
|------|------|------|
| `-32700` | `parse_error` | 请求体不是合法 JSON |
| `-32600` | `invalid_request` | JSON-RPC 包结构不合法 |
| `-32601` | `method_not_found` | 方法不存在 |
| `-32602` | `invalid_params` | 参数缺失或类型错误 |
| `-32603` | `internal_error` | 侧车内部错误 |
| `-32010` | `plugin_timeout` | 侧车上游（模型/检索）超时 |
| `-32011` | `auth_failed` | Token 无效或权限不足 |
| `-32012` | `rate_limited` | 限流 |
| `-32013` | `upstream_unavailable` | 依赖服务不可用 |

### 1.5 HTTP 状态码

建议侧车在 JSON 可解析时返回 **HTTP 200**，错误细节放在 JSON-RPC `error` 中。若返回 **4xx/5xx**，宿主将按失败处理并回退内置。

---

## 2. 环境变量（宿主）

| 变量 | 作用 |
|------|------|
| `OCLIVE_REMOTE_PLUGIN_URL` | 非空时，为 **memory / emotion / event / prompt** 四类 Remote 提供**同一** HTTP 端点；以 `method` 区分行为 |
| `OCLIVE_REMOTE_PLUGIN_TIMEOUT_MS` | 可选；默认 `8000`（毫秒） |
| `OCLIVE_REMOTE_PLUGIN_TOKEN` | 可选；Bearer |
| `OCLIVE_REMOTE_LLM_URL` | 非空且角色包 `plugin_backends.llm = remote` 时，**LLM** 请求发往该 URL |
| `OCLIVE_REMOTE_LLM_TIMEOUT_MS` | 可选；默认 `120000` |
| `OCLIVE_REMOTE_LLM_TOKEN` | 可选；Bearer |

未设置对应 URL 时，即使角色包写了 `remote`，宿主也会使用**占位回退**（builtin 或进程内 LLM），并可能打一次警告日志。

---

## 2. 安全与产品边界（当前实现）

| 主题 | 说明 |
|------|------|
| **协议版本** | 请求头 **`x-oclive-remote-protocol: oclive-remote-jsonrpc-v1`** 标识本文档约定的 JSON-RPC 形状；侧车可据此拒绝不兼容宿主（若将来 bump 版本，须同步文档与头值）。 |
| **超时与错误码** | 见上文 **1.1 超时**、**1.4 JSON-RPC 错误**；宿主侧对超时与 HTTP 非 2xx 按失败处理并**回退内置**，避免远程不可用导致对话崩溃。 |
| **HTTP 侧车（当前）** | 宿主仅向用户配置的 **URL** 发起 **POST**；**不**随角色包自动下载或执行任意本地可执行文件。Token 经环境变量注入，**勿**把密钥写入角色包或提交到仓库。 |
| **未来：子进程 / 可执行插件** | 若以后支持启动本地侧车 exe，须在**单独文档**中定义：路径声明位置、首次运行**用户确认**、沙箱与签名策略；未落地前以本 HTTP 模型为准。 |

---

## 3. Rust 枚举的 JSON 形状（侧车必看）

宿主使用 **serde 默认枚举编码**（**externally tagged**），**不是**裸字符串。

### 3.1 `EventType`（用于 `event.estimate` 的 `result.event_type`）

合法变体：`Quarrel`、`Apology`、`Praise`、`Complaint`、`Confession`、`Joke`、`Ignore`。

**正确示例**（`Ignore`）：

```json
"event_type": { "Ignore": null }
```

**错误示例**（将导致反序列化失败并回退内置）：

```json
"event_type": "Ignore"
```

### 3.2 `Emotion`（用于 `event.estimate` 的 `params.user_emotion`）

合法变体：`Happy`、`Sad`、`Angry`、`Neutral`、`Excited`、`Confused`、`Shy`。

示例：

```json
"user_emotion": { "Neutral": null }
```

### 3.3 `EmotionResult`（用于 `emotion.analyze` 的 `result`）

扁平对象，七个 `f64` 字段：

```json
{
  "joy": 0.0,
  "sadness": 0.0,
  "anger": 0.0,
  "fear": 0.0,
  "surprise": 0.0,
  "disgust": 0.0,
  "neutral": 1.0
}
```

---

## 4. 方法详解与示例

下列 `params` 均指 JSON-RPC 请求中的 **`params` 对象**。

---

### 4.1 `memory.rank`

**params**

| 字段 | 类型 | 说明 |
|------|------|------|
| `memories` | 数组 | 元素为 `Memory` 对象（与 Rust `models::Memory` 一致，含 `id`、`role_id`、`content`、`importance`、`weight`、`created_at` ISO8601、`scene_id` 可选） |
| `user_query` | 字符串 | 当前用户句 |
| `scene_id` | 字符串或 `null` | 当前场景 |
| `limit` | 整数 | 最大条数 |

**result**

| 字段 | 类型 | 说明 |
|------|------|------|
| `ordered_ids` | 字符串数组 | 按优先级排列的 `Memory.id`；宿主按此重排本地列表；未知 id 跳过；未出现在列表中的记忆按**原数组顺序**补在末尾，直至 `limit` |

**请求示例**

```json
{
  "jsonrpc": "2.0",
  "id": 10,
  "method": "memory.rank",
  "params": {
    "memories": [
      {
        "id": "m1",
        "role_id": "demo",
        "content": "上次聊到天气",
        "importance": 0.8,
        "weight": 1.0,
        "created_at": "2026-04-01T12:00:00Z",
        "scene_id": "home"
      }
    ],
    "user_query": "今天出门吗",
    "scene_id": "home",
    "limit": 8
  }
}
```

**响应示例**

```json
{
  "jsonrpc": "2.0",
  "id": 10,
  "result": {
    "ordered_ids": ["m1"]
  }
}
```

---

### 4.2 `emotion.analyze`

**params**

| 字段 | 类型 |
|------|------|
| `text` | 字符串 |

**result**：即 **`EmotionResult`** 七维对象（见 §3.3）。

**响应示例**

```json
{
  "jsonrpc": "2.0",
  "id": 11,
  "result": {
    "joy": 0.2,
    "sadness": 0.0,
    "anger": 0.0,
    "fear": 0.0,
    "surprise": 0.1,
    "disgust": 0.0,
    "neutral": 0.7
  }
}
```

---

### 4.3 `event.estimate`

**params**

| 字段 | 类型 |
|------|------|
| `ollama_model` | 字符串 |
| `user_message` | 字符串 |
| `user_emotion` | `Emotion`（§3.2） |
| `personality` | `PersonalityVector`（七维 `f64`，与 Rust 模型一致） |
| `recent_turns` | 二元组数组 `[[user, bot], ...]`，字符串 |
| `recent_events` | `Event[]`（`event_type` 为 §3.1 形状，`user_emotion`/`bot_emotion` 为字符串） |

**result**：即 **`EventImpactEstimate`**

| 字段 | 类型 |
|------|------|
| `event_type` | `EventType`（§3.1） |
| `impact_factor` | 数字 |
| `confidence` | 数字（0～1 建议） |

**result 示例**

```json
{
  "event_type": { "Ignore": null },
  "impact_factor": 0.0,
  "confidence": 0.5
}
```

---

### 4.4 `prompt.build_prompt`

**params**：扁平对象，字段与宿主 `PromptInput` 序列化一致，主要包括：

- `role`：完整 `Role` JSON（体积较大）  
- `personality`、`memories`、`user_input`、`user_emotion`、`user_relation_id`、`relation_hint`、`relation_before`、`favorability_before`、`relation_preview`、`favorability_preview`  
- `event_type`（`EventType`）、`impact_factor`  
- `scene_label`、`scene_detail`、`topic_hint_line`、`life_context_line`  

**result**

- **对象**：含 `"prompt": "<主对话 prompt 字符串>"`  
- **或** `result` **本身为字符串**：宿主视为整段 prompt  

---

### 4.5 `prompt.top_topic_hint`

**params**

| 字段 | 类型 |
|------|------|
| `role` | `Role` |
| `scene_id` | 字符串 |

**result**

- 对象含 `"hint": "..."` 或 `"hint": null`  
- **或** `result` **本身为字符串**  

---

### 4.6 `llm.generate` / `llm.generate_tag`

**params**

| 字段 | 类型 |
|------|------|
| `model` | 字符串 |
| `prompt` | 字符串 |

**result**

- 对象含 `"text": "..."`  
- **或** `result` **本身为字符串**  

`generate_tag` 用于低温度短输出（立绘标签、位移意图等）。

---

## 5. 安全与运维

- 侧车地址与 Token 应由用户或部署环境**显式配置**；不要在角色包中硬编码生产密钥（角色包可能被分享）。  
- 建议侧车限制请求体大小、连接数，并对日志脱敏。  
- **禁止**在侧车内静默下载并执行未经验证的二进制（与角色包安全原则一致）。

---

## 6. 版本与兼容

- 文档与实现以仓库 **v1** 为准；若将来增加字段，建议侧车忽略未知 `params` 键、宿主忽略未知 `result` 键（当前宿主按固定结构反序列化，**未知形状会回退内置**）。  
- 子进程侧车（非 HTTP）仍可作为实现方式之一，但**当前宿主仅实现 HTTP 客户端**；进程模式见历史草案中的安全说明。

---

## 7. 相关文档

- [CREATOR_PLUGIN_ARCHITECTURE.md](CREATOR_PLUGIN_ARCHITECTURE.md) — 创作者总览与联调步骤  
- [PLUGIN_V1.md](PLUGIN_V1.md) — `plugin_backends` 枚举  
- [../getting-started/DOCUMENTATION_INDEX.md](../getting-started/DOCUMENTATION_INDEX.md) — 文档索引  
