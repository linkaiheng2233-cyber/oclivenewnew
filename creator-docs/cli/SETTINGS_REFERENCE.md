# 蓝图与系统配置参考（SETTINGS_REFERENCE）

> **蓝图文件 `pipeline.ocblueprint`** 是系统配置的唯一来源（**不以** `steps[]` 作主路径调度）。下列字段通常为**蓝图 / 宿主管理员**专属；Stable v4 `inference_profile` 是唯一可由编写器以非技术表单暴露的创作者向例外。角色身份与人格见 **[ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md) §0** · 职责边界 **[handoff/ROLE_PACK_BOUNDARY.md](../../handoff/ROLE_PACK_BOUNDARY.md)**。

## 零、蓝图专属字段（非角色包）

### `runtime_config`（Stable v4 SSOT；v3 双核 Beta）

顶层可选段 **`runtime_config`**（**仅蓝图**；角色包创作者视图除 `inference_profile` 理想配置表单外不暴露）：

| 子字段 | 类型 | 说明 |
|--------|------|------|
| `interaction_mode` | string | `immersive` \| `pure_chat` |
| `memory_config` | object | 记忆策略（`topic_weights` 等，见 [ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md)） |
| `reply_quality_anchor` | string | 主对话质量锚点全文；非空则**仅替换**引擎默认锚点。通用对话纪律由 `KERNEL_DIALOGUE_GUARDRAILS` 每轮恒追加，包级不可覆盖，勿在锚点重复 |
| `remote_fallback_to_builtin` | bool | 包级 Remote 降级建议（全局仍以 `app_settings` / 环境变量为准） |
| `dual_core.enabled` | bool | 双核开关，默认 **`false`**（见 [RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md](../rfc/RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md)） |
| `identity_binding` | string | `global` \| `per_scene` |
| `evolution` | object | 演化引擎参数（含 `personality_source` 与数值策略） |
| `ollama_model` | string | 默认 Ollama 模型（亦可写在 `slot_registry` llm 实例 `model`） |
| `remote_presence` / `autonomous_scene` | object | 异地心声 / 虚拟时间换场景 |
| `inference_profile` | object | **Stable v4 可移植理想推理配置**；只描述生成意图，不选择模型、GGUF、本地运行时或机器参数 |

- **`schema_version: 2`** 文件若含 `runtime_config`：`pack validate` **警告**，宿主**忽略**该段。  
- **`schema_version: 4`**：Stable 路径，以 `runtime_config` 为准；不接受 v3 专属 `dual_core`。
- **`schema_version: 3`**：仅保留冻结的双核 Beta；以 `runtime_config` 为准，并允许 `dual_core`。

#### `runtime_config.inference_profile`（Stable v4）

角色包编写器可把这一段作为“理想配置蓝图”展示给创作者。它表达角色希望怎样生成回复；宿主仍拥有最终决定权，会按用户设置、已安装模型、设备能力与内核安全上限进行裁剪。**实际推理后端和模型继续由 Chat Pro 设置页管理**；本段不得包含 Ollama/llama.cpp 路径、模型名、GGUF 文件、GPU 层数或线程数。

| 路径 | 类型 / 范围 | 语义 |
|------|-------------|------|
| `generation.temperature` | number，`0.0–2.0` | 采样温度偏好 |
| `generation.top_p` | number，`>0.0–1.0` | nucleus sampling 偏好 |
| `generation.preferred_output_tokens` | integer，`1–32768` | 理想回复 token 预算 |
| `generation.maximum_output_tokens` | integer，`1–32768` | 回复 token 硬上限；同时填写时不得小于 preferred |
| `context.minimum_tokens` | integer，`1–262144` | 角色可接受的最小上下文意图 |
| `context.preferred_tokens` | integer，`1–262144` | 理想上下文窗口；同时填写时不得小于 minimum |
| `reasoning.mode` | `instant` \| `adaptive` \| `deep` | 与具体模型无关的推理模式意图 |
| `reasoning.effort` | number，`0.0–1.0` | 推理投入强度意图 |
| `performance_intent.priority` | `latency` \| `balanced` \| `quality` | 延迟、均衡或质量优先 |
| `performance_intent.prefer_prefix_cache` | boolean | 希望宿主复用稳定前缀缓存 |
| `performance_intent.prefer_model_residency` | boolean | 希望模型尽量驻留；`false` 时当前 Ollama 适配会显式请求本轮后卸载（`keep_alive: 0`） |
| `performance_intent.allow_context_reduction` | boolean | 设备受限时是否允许缩小上下文 |
| `performance_intent.allow_output_reduction` | boolean | 设备受限时是否允许缩小输出预算 |

当前内核会把 `temperature`、`top_p`、输出上限和理想上下文传给支持的主 LLM 适配器；其余字段先作为稳定、可前向扩展的意图保留。宿主可以不支持某项提示，但不得把它解释为本机模型选择。

```json
{
  "runtime_config": {
    "inference_profile": {
      "generation": {
        "temperature": 0.8,
        "top_p": 0.9,
        "preferred_output_tokens": 768,
        "maximum_output_tokens": 1536
      },
      "context": { "minimum_tokens": 8192, "preferred_tokens": 16384 },
      "reasoning": { "mode": "adaptive", "effort": 0.65 },
      "performance_intent": {
        "priority": "balanced",
        "prefer_prefix_cache": true,
        "prefer_model_residency": true,
        "allow_context_reduction": true,
        "allow_output_reduction": true
      }
    }
  }
}
```

---

### 槽位与其它蓝图段

| 类别 | 字段 / 段 | 说明 |
|------|-----------|------|
| 槽位实例 | **`slot_registry`**（`type`、`backend`、`plugin`、`model`、`url`、`position`…） | 六槽路由与 directory 插件 id |
| 架构图 | **`groups`** | 同 type 实例分组 |
| 运行时派生 | **`module_relations`** | **禁止落盘**；由 `slot_registry` 派生 |
| 交互与记忆 | **`interaction_mode`**、**`memory_config`**、**`identity_binding`** | Stable v4：`runtime_config.*`；v2 仅兼容 **`meta.*`** |
| 演化引擎 | **`evolution`**（含 `personality_source` 与数值策略） | Stable v4 `runtime_config.evolution`；v2 兼容 `meta.evolution` |
| 模型 | **`ollama_model`** 或 **`slot_registry` 内 `model`** | 蓝图 |
| 远程 / 场景引擎 | **`remote_presence`**、**`autonomous_scene`** | 蓝图 |
| 知识引擎块 | **`meta.knowledge`**（检索配置） | 蓝图，与 `knowledge/` 内容目录区分 |
| 双核（RFC） | **`runtime_config.dual_core.enabled`**、`pipeline.stable` / `pipeline.experimental`、`zone` | 默认 **关**；创作者包不得单独开启 |
| 发版 | **`min_runtime_version`**、**`dev_only`** | 管理员 |

**不属于蓝图文件、属宿主应用**：

| 字段 | 落点 |
|------|------|
| **`remote_fallback_to_builtin`** | `app_settings` / **`OCLIVE_REMOTE_FALLBACK_TO_BUILTIN`** |
| Monolith **`weld_modules`** | 工程 **`monolith.toml`** |

**legacy `settings.json`**（已废弃，勿与 `pipeline.ocblueprint` 并存）：下表 §一 的 `plugin_backends` 等价于今日 **`slot_registry`**。

---

**校验**：`pack validate`（默认蓝图全量）· **`pack validate --profile creator`**（仅角色包，见 [ROLE_PACK_BOUNDARY.md](../../handoff/ROLE_PACK_BOUNDARY.md)）。

**当前蓝图包（新包 Stable v4，兼容 v2）**：后端实例写在 **`pipeline.ocblueprint` → `slot_registry`**。CLI 总览 **`oclive plugin manage --tui`**。下文 §一 **`settings.json` → `plugin_backends`** 仅 **legacy** 对照。

本文档描述 **桌面宿主（Tauri）** 与 **`oclive-cli` 脚手架** 共用的配置语义。单一事实来源以源码为准：

- 枚举与结构体：[`kernel/crates/oclive_kernel_types/src/models/plugin_backends.rs`](../../kernel/crates/oclive_kernel_types/src/models/plugin_backends.rs)
- 解析与绑定：[`kernel/crates/oclive_kernel_host/src/domain/ports/plugin_host.rs`](../../kernel/crates/oclive_kernel_host/src/domain/ports/plugin_host.rs)
- 协议与表格：[`creator-docs/plugin-and-architecture/PLUGIN_V1.md`](../plugin-and-architecture/PLUGIN_V1.md)

**标准 JSON 无注释**：说明性文字请用 **`_` 前缀的键**（加载时忽略），或写在包外文档。`oclive-cli` 生成的示例包使用 `_comment_*` 键解释各槽。

---

## 一、六条宿主槽（`PluginBackends`）

运行时结构体 **`PluginBackends`** 含下列 **6** 个字段（Serde 反序列化时**忽略未知字段**，故 JSON 中可存在额外键如脚手架用的 `complex_emotion`，宿主不报错）。

| 字段 | 门面 trait（编排入口） | 常用内置实现（进程内） |
|------|-------------------------|-------------------------|
| `memory` | [`MemoryRetrieval`](../../kernel/crates/oclive_kernel_runtime/src/domain/memory_retrieval.rs) | 默认 `MemoryBackend::Builtin` |
| `emotion` | 用户情绪分析（见 `plugin_host` / `EmotionAnalyzer`） | `EmotionBackend::Builtin` |
| `event` | 事件影响估计（`EventEstimator`） | `EventBackend::Builtin` |
| `prompt` | `PromptAssembler` / `PromptBuilder` | `PromptBackend::Builtin` |
| `llm` | `LlmClient` | **`LlmBackend::Ollama`**（默认本地客户端；**无 `builtin` 字面量**） |
| `agent` | [`AgentProvider`](../../kernel/crates/oclive_kernel_host/src/domain/agent.rs) | `AgentBackend::Builtin` |

省略整段 `plugin_backends` 时：记忆 / 情绪 / 事件 / Prompt / Agent 为 **`builtin`**，**`llm` 为 `ollama`**（见 PLUGIN_V1 示例）。

### 1.1 各槽可选值（v1 枚举摘要）

完整表与 JSON-RPC 方法名见 **[PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md)**。下面是操作者最常用的取值：

| 槽位 | 常用值 | 选 `remote` / `directory` 时 |
|------|--------|------------------------------|
| memory | `builtin`（`builtin_v2` 读兼容 alias）/ `remote` / `directory` / `local` | `remote`：`OCLIVE_REMOTE_PLUGIN_URL`；`directory`：配置 `directory_plugins.memory` |
| emotion | `builtin`（`builtin_v2` alias）/ `remote` / `directory` | 同上 |
| event | `builtin`（`builtin_v2` alias）/ `remote` / `directory` | 同上 |
| prompt | `builtin`（`builtin_v2` alias）/ `remote` / `directory` | 同上 |
| llm | **`ollama`** / `remote` / `directory` | **`remote`**：`OCLIVE_REMOTE_LLM_URL`；可用 **`OCLIVE_LLM_BACKEND`** 在加载时覆盖；JSON-RPC vs OpenAI-compat、**本机第二本地选型**见 [REMOTE_PLUGIN_PROTOCOL.md](../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md) **§2.0** |
| agent | `builtin` / `remote` / `directory` / `none` | `remote`：侧车 `agent.process`（`OCLIVE_REMOTE_AGENT_URL` 或回退 `OCLIVE_REMOTE_PLUGIN_URL`）；`directory`：配置 `directory_plugins.agent`；协议见 [AGENT_REMOTE_PROTOCOL.md](../plugin-and-architecture/AGENT_REMOTE_PROTOCOL.md) |

**不存在于 v1 枚举的字符串**（如字面量 `none`）会导致 **角色包解析失败**。若脚手架或文档写「none」，表示**逻辑上关闭/不声明**；写入主应用可加载的 JSON 时请 **省略该键**（回退默认）或改为合法枚举。

### 1.2 `directory_plugins` 对象

当任一槽为 **`directory`** 时，应在 `plugin_backends.directory_plugins` 中为对应槽填写 **`manifest.id`**（字符串）。详见 [DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md)。

---

## 二、复杂情感：`plugin_backends` 扩展键（非宿主第六/第七槽）

**架构定位**：**第 1 设施子模块**（规范全名：**复杂情感设施子模块**）。编号与命名见 **[OCLIVE_ARCHITECTURE_OVERVIEW.md](../getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md)**（[English](../../creator-docs-en/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md)）。**专家模型**专名仅指 **第 2 设施子模块**（专家模型设施子模块 / 专家路由），见同文档 § 第 2 设施子模块。

**当前 `PluginBackends` 不含 `complex_emotion` 字段。** `oclive-cli` 将 `complex_emotion` 写在 **`plugin_backends` 对象内** 便于工厂预设与文档对齐；宿主 Serde **忽略**该键，不影响 `load_role`。主路径在 `co_present` 内经 **`PluginHost` / `SlotRunner`** 解析：蓝图 **`slot_registry`** 中 `type: complex_emotion` + `backend: builtin|remote|directory|none` 时 **last-wins**（`resolve_complex_emotion_winner`）。**省略（无条目）= 不启用**（跳过 provider、不产 hint）；显式 `builtin` = 开启；显式 `none` = 明确关闭（与省略等价）。

| 项 | 说明 |
|----|------|
| 与 **emotion 后端模块** | emotion 产出 `EmotionResult`；本设施消费其推导指标，产出 `narrative_hint` 供 **prompt 后端模块** |
| 与 **后端模块插件模块** | 侧车方法 `complex_emotion.resolve_turn`（`OCLIVE_COMPLEX_EMOTION_URL` 或 `slot_registry` remote/directory）；**不**占第 7 模块号；**不**经 `plugin_backends.complex_emotion` 六槽键切换 |
| 与 **Monolith** | 焊接键名 `complex_emotion`（七焊接键之一），≠ 宿主槽位 |

- 侧车 wire：[REMOTE_PLUGIN_PROTOCOL.md](../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md)。
- 与 **`oclive-cli`** `CONFIG_REFERENCE.md`、`init --help` 预设矩阵一致。

---

## 三、`oclive-cli` 内核工厂模板（`--template`）

与 **`init --help`** 末尾模板表一致；显式 **`--preset`** / **`--monolith`** / **`--monolith-preset`** / **`--project-type`** / **`--with-role-pack`** / **`--with-example-plugin`** 优先。愿景：[KERNEL_FACTORY_VISION.md](../getting-started/KERNEL_FACTORY_VISION.md)

| template | preset | Monolith 默认 | project-type |
|----------|--------|---------------|--------------|
| `robot-soul` | minimal | 启用 | kernel_server |
| `robot-gateway` | mixed | 启用 | kernel_server |
| `dialogue-only` | full | 关闭 | kernel_server |
| `headless-api` | full | 关闭 | kernel_server |
| `library-embed` | minimal | 关闭 | library |

角色包跨发行版底座：`pack validate --profile portable-core`。它只检查 Portable Core（基础人格 Prompt + 七张默认情绪图片），不限制发行版自己的 HostProfile、UI、语音、视觉或硬件扩展。

**`--monolith-preset`**（Monolith 启用时）：`latency` | `memory` | `embedded` — 预填 `weld_modules`（见生成工程 `CONFIG_REFERENCE.md`）。

**`--monolith-bench-preset`**：同上档位；生成后自动 bench（5 轮），结果见 `bench_results/report.json` 与 `docs/WELD_BENCH_REPORT.md`。

**`--list-templates`**：列出五套模板；交互 `init` 默认含「不使用模板」项。

**`robot-gateway`**：附带 `mcp_servers/` 与 `distros/chat-pro/roles/gateway/settings.json`（`agent` = builtin，`agent_mcp` 占位）。

**`--quick` / `-q`**：full 预设、无 Monolith、无示例角色包。

**`oclive doctor`**：环境一键诊断（`--json` 可编程解析）。

**`oclive bench`**：报告 schema v2 含 `binary_size`、`peak_memory`、`build_time`；**`--history`** / **`--watch`**。

**`registry`**：`~/.oclive/registry.json`（`OCLIVE_HOME`）；`init` 后自动注册。

**`compose`**：`oclive-compose.yml` + `.oclive-compose.pids.json`。

**`template pack` / `--template-url`**：隐藏兼容的旧 `.oclive-template.tar.gz` + `template.json` 工程归档；顶层 `publish` 已移除，新 Scaffold Package 不复用这条链。

**质量深耕**：`init --from-existing`、`bench --stress`、`test --ci-parity`、`lint --deps`、`doctor --watch`、`kernel info` — 见 [OCLIVE_CLI_GUIDE.md](OCLIVE_CLI_GUIDE.md)。

**`init --tui`** · **`debug`**（`OCLIVE_DEBUG_TRACE`）：见 [OCLIVE_CLI_GUIDE.md](OCLIVE_CLI_GUIDE.md)。

**`market`**：`OCLIVE_MARKET_INDEX_URL` 或 **`OCLIVE_PLUGIN_INDEX_URL`**；离线 **`~/.oclive/plugin_index_cache.json`**。

**`registry` 云端**：**`OCLIVE_REGISTRY_URL`**、**`OCLIVE_REGISTRY_TOKEN`**；凭据 **`~/.oclive/auth.json`**（`registry login`）。

**`collab`**：角色包目录 **`.oclive-collab.yml`**（Git `remote` / `branch`）。

**`oclive config`**：用户级 **`~/.oclive/config.toml`**、工程级 **`.oclive.toml`**；`config list` 合并已知 `OCLIVE_*` 键。`registry` / `market` 优先读进程环境变量，其次配置文件。

**`oclive ci init`**：生成 **`.github/workflows/ci.yml`**（ubuntu / windows / macos 矩阵）。

**`bench --regression`**：对比 **`bench_history.json`** 末条；默认阈值 p50 5% / P95 10% / 内存与二进制 5–10%。

## 四、`oclive-cli` 预设矩阵（逻辑 → JSON）

| 槽位 | minimal | mixed | full |
|------|---------|-------|------|
| memory / emotion / event / prompt | builtin | builtin | builtin |
| llm | ollama | ollama | remote |
| agent | **省略键**（语义 none） | builtin | builtin |
| complex_emotion | none | builtin | remote |

---

## 四、从 `builtin` / `ollama` 切换到 `remote`（步骤）

1. 准备侧车（HTTP JSON-RPC），实现 PLUGIN_V1 / REMOTE_PLUGIN_PROTOCOL 对应方法。
2. 在运行环境中设置 URL，例如 **`OCLIVE_REMOTE_PLUGIN_URL`**（多子系统共用侧车时）与 **`OCLIVE_REMOTE_LLM_URL`**（仅 LLM）。
3. 编辑 **`settings.json`** → `plugin_backends`：将目标槽改为 **`remote`**（`llm` 改为 **`remote`**，不是 `builtin`）。
4. 重启宿主或重新加载角色；观察日志中降级/回退提示（未配置 URL 时可能回退内置实现）。

---

## 五、`monolith.toml`（编译期，非运行时）

由 **`oclive-cli init`** 在启用 Monolith 时写入**项目根目录**；**仅编译期**消费（**`cargo run -p oclive-cli -- --experimental build`** 读取并再生成 `process_message_monolith.rs`；亦可仅用手动 **`cargo build --features monolith`**）。与 **`settings.json` → `plugin_backends`** 正交：角色包加载**不**读取本文件。

| 字段 | 说明 |
|------|------|
| **`[monolith].enabled`** | 是否为该项目启用 Monolith 编译路径（`oclive build` 在 `false` 时跳过第二次带 `monolith` 的 `cargo build`）。 |
| **`weld_modules`** | 焊接模块名列表；**空数组** 表示「从全槽焊接出发，再应用 `exclude`」。与 **`exclude` 不能同时非空**。 |
| **`exclude`** | 当 **`weld_modules` 为空** 时，从全槽焊接中排除所列槽；这些槽在生成代码中走 trait/PluginHost 占位。 |

**基准报告 JSON Schema**（`oclive bench`）：[`kernel/crates/oclive-cli/schemas/oclive_bench_report.schema.json`](../../kernel/crates/oclive-cli/schemas/oclive_bench_report.schema.json)（仓库内相对链接以克隆路径为准）。

**本地历史**：`bench --save` 追加 **`bench_history.json`**；`bench --history` 打印趋势表；`bench --compare` 对比最近两次。勿提交 `bench_history.json`。

详见 [RFC_OCLIVE_MONOLITH_MODE.md](../rfc/RFC_OCLIVE_MONOLITH_MODE.md) 第 4 节。

---

## 六、角色包 `config.json` → `chat_storage`

位于 `distros/chat-pro/roles/{role_id}/config.json` 的 **`chat_storage`** 对象（可选）。宿主在 `RoleStorage::load_role` 时读取；类型见 `oclive_kernel_types::RolePackChatStorageConfig`。与蓝图 `pipeline.ocblueprint` **无关**。

| 键 | 类型 | 默认值 | 说明 |
|----|------|--------|------|
| `backend` | `"hybrid"` \| `"file"` \| `"sqlite"` | `"hybrid"` | 聊天记录存储后端（进程级可被 `OCLIVE_CHAT_STORAGE_BACKEND` 覆盖） |
| `max_messages_per_session` | `u32` | 宿主 **500** | 单会话 FIFO 上限（user+assistant 合计条数） |
| `auto_cleanup_days` | `u32` | 未设=关闭 | 删除 `updated_at` 早于 N 天的会话 |
| `auto_cleanup_max_sessions` | `u32` | 未设=关闭 | 每角色最多保留 N 个会话（删最旧） |
| `replay_similarity_threshold` | `f64` | `0.6` | 记忆回放去重相似度（**0.1–1.0**）；越大越严格，重复记忆越少 |
| `location` | `"role_pack"` \| `"global"` | `"global"` | 聊天记录存储位置：角色包子目录 `chats/` 或全局 `{app_data}/chats/`；`role_pack` 不可写时自动退回到全局 |

选型说明：[STORAGE_BACKEND_GUIDE.md](../storage/STORAGE_BACKEND_GUIDE.md) · 架构：[CHAT_STORAGE_ARCHITECTURE.md](../../handoff/CHAT_STORAGE_ARCHITECTURE.md)。

---

## 七、相关文档索引

| 主题 | 文档 |
|------|------|
| CLI 使用与参数 | [OCLIVE_CLI_GUIDE.md](OCLIVE_CLI_GUIDE.md) |
| 生成项目内预设表 | 运行 `init` 后的 **`CONFIG_REFERENCE.md`** |
| 聊天存储后端选型 | [STORAGE_BACKEND_GUIDE.md](../storage/STORAGE_BACKEND_GUIDE.md) |
| 插件与侧车总览 | [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md) |
| 目录插件 | [DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md) |
| 编译期高耦合模式（Monolith） | [RFC_OCLIVE_MONOLITH_MODE.md](../rfc/RFC_OCLIVE_MONOLITH_MODE.md)（`monolith.toml`、`build` / `bench`、双 `[[bin]]`） |

---

[English](../../creator-docs-en/cli/SETTINGS_REFERENCE.md)
