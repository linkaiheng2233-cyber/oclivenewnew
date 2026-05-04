# Profile 文件规范 v1.0（冻结标准）

> 状态：**Draft → 建议冻结 v1.0**  
> 目标：让创作者/发行版可以用“特征码”一键复现一套内核运行环境（插件版本锁定 + 后端路由 + 市场源 + 权限确认策略），并保持安全、可审计、可回滚。

---

## 1. 定义与范围

- **Profile**：一个“发行配置配方”（不含代码），描述 **Kernel 如何装配与路由模块**。
- **不包含**：任何插件代码、二进制、zip 包内容。
- **允许包含**：
  - 六模块（固定存在）在“路由层”的后端选择（含 `none` 静音语义）。
  - 可选模块（例如 Module 7: Agent）在路由层的后端选择。
  - 插件版本锁定（从市场拉取时用于可复现）。
  - 市场源启用列表、开发者模式开关。
  - 权限预声明（**不等于自动授权**；最终以用户确认/勾选为准）。

> 注意：本规范只定义文件与字段语义；实现细节（如何 apply、如何重启生效、如何弹窗确认）不在本文档范围。

---

## 2. 文件身份与命名

- **推荐文件名**：`.oclive.profile.json`
- **必填字段**：
  - `type`: 固定为 `"profile"`
  - `schema_version`: 固定为 `"1.0"`

---

## 3. 顶层字段（v1.0）

| 字段 | 类型 | 必填 | 默认值 | 说明 |
|---|---:|---:|---:|---|
| `type` | string | ✅ | - | 固定 `"profile"` |
| `schema_version` | string | ✅ | - | 固定 `"1.0"` |
| `id` | string | ✅ | - | 特征码（稳定、不含空白），例：`"creator-min"` |
| `name` | string | ✅ | - | 展示名称，例：`"创作者最小闭环"` |
| `version` | string | ✅ | - | Profile 自身语义化版本，例：`"1.0.0"` |
| `description` | string | - | `""` | 简短说明（给人看的） |
| `developer_mode` | boolean | - | `false` | 是否开启开发者模式（影响第三方源/侧载能力开关） |
| `market_sources` | array | - | `["official"]` | 启用的市场源（见 §6） |
| `plugins` | array | - | `[]` | 需要拉取/锁定的插件清单（见 §5） |
| `permissions` | object | - | - | 权限预声明与确认策略（见 §7） |
| `backends` | object | ✅ | - | 模块后端路由配置（见 §4） |
| `compat` | object | - | - | 兼容性门禁（建议，但非强制；见 §8） |

---

## 4. `backends`：模块后端路由（冻结要点）

### 4.1 六模块（固定存在）与 `none` 语义

六模块固定存在、接口不可移除：

- `memory`
- `emotion`
- `event`
- `prompt`
- `llm`
- `complex_emotion`

每个模块在 Profile 中必须显式给出 `backend`，并允许 `backend: "none"` 作为“静音 Provider”。

> 具体 `none` 的输入/输出降级语义见 **[MODULE_NONE_SEMANTICS.md](./MODULE_NONE_SEMANTICS.md)**。

### 4.2 允许的 `backend` 值（与现有代码库对齐）

> 约定：枚举字符串采用 **snake_case**，与仓库中 `PluginBackends` 相关枚举一致（见 `crates/oclive_kernel_runtime/src/models/plugin_backends.rs` 与文档 `creator-docs/plugin-and-architecture/PLUGIN_V1.md`）。

#### `memory.backend`

- 允许：`"builtin" | "builtin_v2" | "remote" | "directory" | "local" | "none"`

#### `emotion.backend`

- 允许：`"builtin" | "builtin_v2" | "remote" | "directory" | "none"`

#### `event.backend`

- 允许：`"builtin" | "builtin_v2" | "remote" | "directory" | "none"`

#### `prompt.backend`

- 允许：`"builtin" | "builtin_v2" | "remote" | "directory" | "none"`

#### `llm.backend`

- 允许：`"ollama" | "remote" | "directory" | "none"`
- 说明：
  - `remote` 对应 HTTP JSON-RPC（`OCLIVE_REMOTE_LLM_URL`）
  - 云端直连 OpenAI-compatible（`OCLIVE_CLOUD_LLM_*`）属于 **实现侧优先级**，仍归于“LLM 路由层”；Profile 只描述“期望路由”，不承诺环境一定可用（不可用时应有安全降级/提示）

#### `complex_emotion.backend`

- 允许：`"builtin" | "remote" | "none"`
- 说明（与现状映射）：
  - `builtin`：当前内置为 `builtin_keyword_v1`（见 `crates/oclive_kernel_runtime/src/domain/complex_emotion.rs`）
  - `remote`：当前使用独立端点 `OCLIVE_COMPLEX_EMOTION_URL`（兼容旧名 `OCLIVE_REMOTE_COMPLEX_EMOTION_URL`），JSON-RPC 方法 `complex_emotion.resolve_turn`（见 `src-tauri/src/infrastructure/remote_plugin/complex_emotion_http.rs`）

### 4.3 可选模块（插件模块）示例：`agent`

虽然本规范聚焦“六模块固定存在”，但 Profile 允许对 **额外模块**给出路由建议（例如 Module 7: Agent）：

- `agent.backend` 允许：`"builtin" | "remote" | "directory" | "none"`
  - 说明：与 `crates/oclive_kernel_runtime` 中 **`AgentBackend::None`** 一致；`none` 时见 **[MODULE_NONE_SEMANTICS.md](./MODULE_NONE_SEMANTICS.md) §7**（`DisabledAgentProvider` / `AGENT_BACKEND_NONE_REPLY`）。

### 4.4 `directory_plugins` 槽位（当 backend=directory 时）

当某模块 `backend="directory"` 时，必须同时提供对应槽位插件 id（manifest `id`）：

```json
{
  "backends": {
    "memory": { "backend": "directory" },
    "directory_plugins": {
      "memory": "my.memory.provider"
    }
  }
}
```

> 槽位命名与 `PluginBackends.directory_plugins.*` 一致（见 `crates/oclive_kernel_runtime/src/models/plugin_backends.rs`）。

### 4.5 `local_memory_provider_id`（当 memory=local 时）

当 `memory.backend="local"` 时，允许指定 `local_memory_provider_id` 来消除歧义（与 `PluginBackends.local_memory_provider_id` 语义一致）。

---

## 5. `plugins`：插件版本锁定（可复现）

### 5.1 字段

`plugins` 为数组，每一项为：

| 字段 | 类型 | 必填 | 说明 |
|---|---:|---:|---|
| `id` | string | ✅ | 插件 id（manifest `id`） |
| `version` | string | ✅ | 锁定版本（语义化版本字符串） |
| `source` | string | ✅ | `"official"` 或 `"third_party:<source_id>"` |
| `download_url` | string | - | 可选：指向具体版本不可变下载地址（建议为 release asset） |
| `signature_url` | string | - | 可选：指向具体版本签名文件 URL（与插件市场签名体系对齐） |
| `pubkey_id` | string | - | 可选：期望的 `pubkey_id`（用于校验/审计提示） |

> 说明：v1.0 只冻结“字段存在与语义”，不强制每项都带 URL；但为满足“可回滚/可复现”，推荐在官方 profile 中提供不可变 `download_url`/`signature_url`。

---

## 6. `market_sources`：市场源启用列表

- 类型：`string[]`
- 默认：`["official"]`
- 约束：
  - `"official"`：官方默认索引（推荐）
  - `"third_party:<source_id>"`：第三方源（**仅开发者模式**可启用；实现侧必须给出强提示/风险告知）

> 注：仓库当前对“市场源配置”的运行时结构以实现为准；Profile 的目标是“可复现输入”，因此用字符串 id 表达，不绑定具体 UI。

---

## 7. `permissions`：权限预声明与确认策略（安全优先）

Profile 允许预声明“本 profile 预计会用到哪些权限”，但 **不得跳过用户确认**。

建议形状：

```json
{
  "permissions": {
    "predeclared": ["network.http", "filesystem.read"],
    "require_confirm": ["shell.exec"]
  }
}
```

- `predeclared`：提示用户“这套 profile 可能会请求这些权限”
- `require_confirm`：高风险权限，必须二次确认（或更显著的确认）

硬性规则（冻结）：

- Profile 里的任何权限字段 **不等于自动授权**。
- Apply Profile 时必须触发权限确认流程；最终写入的 grants 以“用户确认结果”为准。
- 对于非官方市场源或未签名/签名状态异常条目，确认 UI 必须额外显示风险提示。

> 权限 token 的权威清单以现有权限体系文档与实现为准（Profile 仅复用 token 字符串，不引入新 token）。

---

## 8. `compat`：兼容性门禁（建议字段）

为支持“OOCP 协议版本 + 内核能力版本”双门禁，建议引入（v1.0 允许可选）：

```json
{
  "compat": {
    "oocp_min": "0.1.0",
    "kernel_caps_min": "0.2.0"
  }
}
```

规则建议：

- 不满足门禁时：必须拒绝 apply，并返回可读错误（提示升级内核/更换 profile）。
- Profile 可以被更高版本内核使用（向前兼容），但实现侧不得悄悄降级到“行为不确定”。

> 具体字段名与 capabilities 的权威来源以 OOCP capabilities 为准；Profile 只定义一个“门禁表达层”。

---

## 9. 示例：创作者最小闭环 Profile（节选）

```json
{
  "type": "profile",
  "schema_version": "1.0",
  "id": "creator-min",
  "name": "创作者最小闭环",
  "version": "1.0.0",
  "developer_mode": false,
  "market_sources": ["official"],
  "permissions": {
    "predeclared": ["network.http"],
    "require_confirm": ["shell.exec"]
  },
  "plugins": [
    {
      "id": "example.llm.remote.bridge",
      "version": "1.2.3",
      "source": "official"
    }
  ],
  "backends": {
    "memory": { "backend": "builtin_v2" },
    "emotion": { "backend": "builtin" },
    "event": { "backend": "builtin" },
    "prompt": { "backend": "builtin" },
    "llm": { "backend": "ollama" },
    "complex_emotion": { "backend": "builtin" },
    "agent": { "backend": "none" }
  }
}
```

