# 发行版能力配置（Distro Capability Profile）

**状态**：P1 契约（Schema + 示例）**Done**；P4 profile 调度（`HostProfile` 加载与合并）**Done**（`host_profile.rs` / spawn 时 `OCLIVE_DISTRO_PROFILE`）。  
**受众**：桌面、VS Code、启动器、硬件发行版集成方。  
**SSOT 模块形状**：与角色包 `settings.json` → `plugin_backends` 对齐，见 [`PLUGIN_V1.md`](../plugin-and-architecture/PLUGIN_V1.md) 与 `kernel/crates/oclive_validation/src/plugin_backends.rs`。

---

## 1. 定位与边界

| 层级 | 文件位置 | 作用 |
|------|----------|------|
| **发行版** | 发行版根目录 `distro.oclive.toml`（与 bundled `bin/` 同级） | spawn 时加载的 **HostProfile**：prompt/memory/post_process、`host_flags`、可选 **`[plugin_backends]` 整表替换** |
| **角色包** | `distros/chat-pro/roles/<id>/pipeline.ocblueprint` → `slot_registry`（v2/v3/v4 精确分派）；legacy `settings.json` | 六槽默认；可被发行版 profile **整表替换**（若 profile 声明 `[plugin_backends]`） |
| **会话** | 宿主 DB / 会话覆盖 | 在有效 backends 上临时覆盖字段 |

**不承载于**：蓝图文件 `pipeline.ocblueprint` / blueprint v3 `runtime_config`（v3 冻结，见 handoff）。**不替代** Monolith `monolith.toml`（仅编译期）。后处理链扩展点 RFC（预留）：[RFC_OCLIVE_POST_PROCESS_CHAIN.md](../rfc/RFC_OCLIVE_POST_PROCESS_CHAIN.md)。

**与内核二进制的关系**：配置文件描述「该发行版 spawn 时期望的有效模块矩阵 + prompt/memory 偏好」；**不**声明裁剪内核二进制。进程选择见 [DISTRO_KERNEL_LIFECYCLE.md](./DISTRO_KERNEL_LIFECYCLE.md)（bundled-first spawn · attach/replace）；范围裁定见 [KERNEL_SCHEDULER_RESCOPE.md](../../handoff/KERNEL_SCHEDULER_RESCOPE.md)。

### HostProfile ≠ OS 能力矩阵

`HostProfile` / `distro.oclive.toml` 只锁定**发行版模块矩阵**（六槽 · `host_flags` · prompt/memory），**不**声明 Windows / Linux / macOS 上的 ASR、TTS、webview 真值。OS × 语音差异 SSOT：

| OS | ASR（产品 sherpa） | TTS CosyVoice2（bundled） | Chat Pro webview 语音 UI |
|----|--------------------|---------------------------|--------------------------|
| Windows | 已交付 | 已交付 | 已交付 |
| Linux | `unsupported`（[`asr_profiles.json`](../../distros/chat-pro/plugins/com.oclive.voice.asr/asr_profiles.json)） | `unsupported`（产品化属 **K-VOICE-03**，本文不宣称） | 宿主可跑；**不**宣称三 OS 语音闭环已验 |
| macOS | 同上 `unsupported` | 同上 `unsupported` | 同上 |

详表与 human-only 行 → [`TRACK_VOICE_RECOGNITION.md`](../../human-docs/team/TRACK_VOICE_RECOGNITION.md)（平台差异节）。

**已有自动化入口（≠ 三平台实机语音证明）**：`npm run test:distro-profile-mirror` · `npm run test:distro:smoke` · `npm run test:theater:smoke` · `node scripts/test-voice-build-directive.mjs` · `node scripts/test-voice-speak-path.mjs --probe-only` · `node scripts/check-voice-tts-ratchet.mjs`。三平台设备 smoke 仅人工；本债文档化阶段**未**跑。

---

## 2. 文件位置与命名

| 产品 | `distro_id` | 建议路径 |
|------|-------------|----------|
| **Chat Pro**（桌面 Release hero） | `desktop` | 示例：`examples/distro-profiles/desktop.oclive.toml`；**安装包**：`resources/distro-profiles/desktop.oclive.toml`（Tauri bundle，默认 spawn） |
| **VS Code Flash** | `vscode` | 扩展根 `distro.oclive.toml`；主仓镜像：`examples/distro-profiles/vscode.oclive.toml` |
| **dev lab**（非 Release） | `desktop-chat` | 示例：`examples/distro-profiles/desktop-chat.oclive.toml` |
| **AI Theater**（Deferred） | `theater` | 示例：`examples/distro-profiles/theater.oclive.toml`；**安装包**：`resources/distro-profiles/theater.oclive.toml`（`OCLIVE_SHELL=theater` 时自动选用） |
| 自定义 | 任意 | 由环境变量 `OCLIVE_DISTRO_PROFILE` 指向 |

**发行版标识**（P4 传入内核）：

- 环境变量：`OCLIVE_DISTRO_ID`（如 `desktop`、`vscode`）
- HTTP（可选）：请求头 `X-OCLive-Distro-Id: vscode`

---

## 3. Schema（`schema_version = 1`）

```toml
schema_version = 1
distro_id = "vscode"          # 必填，稳定小写 id
display_name = "OCLive VS Code"

# --- 模块上限（可选；省略 = 不额外收紧，仅用 host_flags / slots）---
[plugin_backends]
memory = "builtin"            # builtin (builtin_v2 read alias) | remote | local | directory | none
emotion = "builtin"           # … | none
event = "builtin"             # … | none
prompt = "builtin"            # … | none（共景路径禁止 none，见 MODULE_NONE_SEMANTICS.md）
llm = "ollama"                # ollama | remote | directory | none（共景路径禁止 none）
agent = "builtin"             # builtin | remote | directory | none；若 host_flags.skip_agent = true，运行时强制 agent = none

# --- 槽位（第 7 模块等，非 plugin_backends 字段）---
[slots]
complex_emotion = "off"       # on | off

# --- 宿主级开关（表达「关闭」而不扩展枚举）---
[host_flags]
skip_agent = true
skip_complex_emotion = true
event_impact_llm = false          # optional; default true — false 跳过 event LLM generate_tag

# --- builtin local LLM 的发行版实现（不是角色包 backend）---
[llm_runtime]
mode = "performance"              # ollama | performance
endpoint = "http://127.0.0.1:8421"
auto_start = true                 # 已安装运行时包时允许宿主拉起 llama-server
startup_timeout_ms = 90000
retry_cooldown_ms = 30000
model_alias = "oclive-performance"

[resource_coordination]
gpu_safety_reserve_mib = 768      # 冷启动后仍须保留的设备安全余量
pending_lease_ttl_ms = 120000     # 未激活租约的竞态/取消兜底
active_lease_ttl_ms = 1800000     # 活跃租约诊断 TTL
allow_unverified_admission = true # nvidia-smi 不可用时允许内置适配器保守尝试

[turn_thinking]
default = "auto"                  # fast | deep | auto
fast_skip_complex_emotion = true  # optional
auto_deep_min_chars = 80          # optional
fast_knowledge_limit = 4          # optional
fast_memory_cap = 4               # optional

# --- Prompt / 记忆 / 后处理（P4 映射；P1 先约定语义）---
[prompt]
profile = "concise"           # full | concise

[memory]
retrieval = "default"         # default | light

[post_process]
chain = "standard"            # standard | minimal

[user_identity]
default_id = "classmate"      # optional; used when session has no explicit identity
allowed_ids = ["classmate"]   # optional whitelist for set_user_identity API

[interaction]
default_mode = "pure_chat"    # pure_chat | immersive — first-run seed when DB unset
allow_mode_switch = true
immersive_unlock_hint_after_turns = 10

[state_expression]
favor_high = "…"              # optional; appended to 【角色当前状态】 when favor ≥ 65
favor_mid  = "…"              # optional; 40 ≤ favor < 65
favor_low  = "…"              # optional; favor < 40
```

### 3.1 `plugin_backends` 枚举

与角色包 `settings.json` 相同（`snake_case`）：

| 键 | 合法值 |
|----|--------|
| `memory` | `builtin`（`builtin_v2` 读兼容 alias）, `remote`, `local`, `directory` |
| `emotion` | `builtin`（`builtin_v2` alias）, `remote`, `directory` |
| `event` | `builtin`（`builtin_v2` alias）, `remote`, `directory` |
| `prompt` | `builtin`（`builtin_v2` alias）, `remote`, `directory` |
| `llm` | `ollama`, `remote`, `directory` |
| `agent` | `builtin`, `remote`, `directory` |

`directory_plugins.*` 仅在对应 backend 为 `directory` 时使用（与角色包一致）。

### 3.2 `host_flags` 与 `slots`

- **`host_flags.skip_agent`**：为 `true` 时，运行时强制 `plugin_backends.agent = none`（与角色包声明 `agent: none` 等效）。
- **`host_flags.skip_complex_emotion`**：为 `true` 时，跳过共景复杂情感解析（`co_present` 阶段）。
- **`host_flags.event_impact_llm`**：为 `false` 时，**全局**跳过第 3 模块 event 的 LLM `estimate_event_impact`（`generate_tag`）；仍走规则 `EventDetector` / `estimate_event_impact_rules_only`。环境变量 `OCLIVE_EVENT_IMPACT_LLM=0` 等价。与 **Turn Thinking** 组合：Fast 轮本就不调 event LLM；Deep 轮仍受本开关约束。见 [`handoff/TTFT_BENCHMARK.md`](../../handoff/TTFT_BENCHMARK.md)。
- **`slots.complex_emotion`**：`off` 等价于 `skip_complex_emotion`（二者任一为 off 即关闭）。

### 3.2.1 `[llm_runtime]`（发行版运行时 · 非角色包 backend）

`plugin_backends.llm` 仍只有 `ollama | remote | directory | none`。`[llm_runtime]` 只决定发行版本身如何实现 builtin local 槽，因此不要求角色包、编写器或目录插件增加新的 backend 枚举。

| `mode` | 主路径 | 降级 |
|--------|--------|------|
| `ollama` | 现有 `OllamaClient` | 无第二本地运行时 |
| `performance` | loopback OpenAI-compatible `llama-server` 真流式接口 | 仅在首个 token 尚未发出时回退 Ollama；已输出部分内容后禁止重跑，避免重复回复 |

性能模式的可选组件边界：

1. **本体**：只包含 `LlmClient` 契约、运行时发现/生命周期、SSE 流式适配和 Ollama fallback，不捆绑权重。
2. **LLM runtime 组件**：安装到 `{app_data}/components/llm-runtime/`；manifest 固定名 `llm_runtime_pack.json`，必填 `schema_version = 1`、非空 `component_id`、`component_type = "llm_runtime"`、`engine = "llama.cpp"`、SemVer `version`、相对 `executable` 与 64 位十六进制 `executable_sha256`。宿主拒绝哈希不符、绝对路径和含 `..` 的路径。无 manifest 的约定路径仅限 debug 构建；开发环境也可显式设置 `OCLIVE_LLAMA_SERVER_PATH`。
3. **官方模型组件**：千问 7B GGUF 与本体分发；安装后由模型管理器选择文件。用户自带 GGUF/BIN 走相同选择路径，不先导入 Ollama。
4. **语音组件**：保持独立扩展；不得因为安装语音包而改变 LLM backend/schema。

有效模型路径保存在用户设置 `user_local_llm_model_path`，进程镜像为 `OCLIVE_LOCAL_LLM_MODEL_PATH`。Ollama 模型名仍单独保存，作为降级目标；不得把 Windows/GGUF 路径写入 Ollama model id。

运行时在发行版启动后后台预热：性能组件和模型齐全时只加载 llama-server；任一缺失时才预热 Ollama，避免两个模型同时占用显存。GGUF 切换会终止宿主管理的旧 llama-server 并按新路径重启。

### 3.2.2 `[resource_coordination]`（宿主控制面 · 非蓝图字段）

该段提供发行版级默认策略，不允许蓝图直接分配显存或发出卸载命令。桌面默认值为：

| 字段 | 默认 | 语义 |
|------|------|------|
| `gpu_safety_reserve_mib` | `768` | 新冷启动工作负载批准后仍需保留的显存余量 |
| `pending_lease_ttl_ms` | `120000` | reserved 租约在取消/崩溃时的兜底过期时间 |
| `active_lease_ttl_ms` | `1800000` | observe-only 活动租约的诊断 TTL；managed 常驻运行时由适配器显式释放 |
| `allow_unverified_admission` | `true` | `nvidia-smi` 不可用时允许内置适配器继续保守尝试，并把状态标为 degraded |

环境覆盖：`OCLIVE_GPU_SAFETY_RESERVE_MIB`、`OCLIVE_RESOURCE_ALLOW_UNVERIFIED`。适配器估算可用 `OCLIVE_LLAMA_GPU_RESERVATION_MIB`、`OCLIVE_COSYVOICE_GPU_RESERVATION_MIB` 覆盖；目标设备可用 `OCLIVE_GPU_DEVICE_INDEX`（其次读取 `CUDA_VISIBLE_DEVICES`）选择。这些适配器覆盖不属于角色包格式。

当前统一协调器已覆盖 NVIDIA 设备快照、并发 pending reservation、managed llama-server 冷启动/释放、observe-only Ollama 前台活动与官方 bundled CosyVoice2 准入。Performance LLM 的资源暂停会关闭统一请求门、排空在途 primary/fallback 请求并卸载本运行时追踪的模型；普通故障降级不受影响，显式预热/应用模型选择负责恢复。低显存下，bundled `voice.speak` 只在 LLM 空闲且资源可控时暂停本地 Performance，Chat Pro 会把生成期间遭拒的语音延后到最终文本完成后通过 RPC 重试；插件确认 CosyVoice 卸载后，宿主才撤销 Voice 租约并恢复 LLM，未确认则保守保留状态。宿主 Resource Adapter Registry 同时通过资源诊断 v2 暴露这些适配器的控制模式、adapter-local 运行档位、驻留能力、生命周期动作和当前租约；注册表描述能力但不自行调度。纯 Plan Compiler / CLI doctor 不探测硬件，返回 `not_evaluated`；桌面 `get_execution_plan_diagnostics` 与 `get_resource_coordination_diagnostics` 刷新运行态。当前适配器尚不允许协调器通用自动切档；第三方注册入口、公平队列、RAM/CPU、渲染适配器和真实共享显存 soak 仍是后续债务。完整边界见 [蓝图扩展与资源协调 RFC](../rfc/RFC_BLUEPRINT_EXTENSION_AND_RESOURCE_COORDINATION.md)。

### 3.2.3 `[turn_thinking]`（编排行 · 非六槽）

每轮 **Fast / Deep** 思考档位，由 `co_present` 内 `TurnThinkingRouter` stage 解析（`turn_thinking.rs`）。**不是**设施子模块号，**不**写入 `plugin_backends`。

| 字段 | 合法值 / 类型 | 说明 |
|------|----------------|------|
| `default` | `fast` \| `deep` \| `auto` | `auto`：闲聊→Fast；长句 / 高唤醒情绪 / Quarrel 事件链 / 关键词→Deep |
| `fast_skip_complex_emotion` | bool | Fast 轮跳过复杂情感（可与 `host_flags.skip_complex_emotion` 叠加） |
| `auto_deep_min_chars` | usize | Auto 触发 Deep 的最小用户句字符数 |
| `fast_knowledge_limit` | usize | Fast 轮知识检索条数上限 |
| `fast_memory_cap` | usize | Fast 轮注入 prompt 的记忆条数上限 |
| `deep_capsule` | bool（**Wave D**，兼容字段名） | 发行版强制开/关离线 persona capsule；`true` 且角色含 `prompts/deep_capsule.txt` + `meta.deep_capsule_enabled` 时，Small 模型的 Fast/Deep 轮均用 capsule 替代全量 `core_personality` |
| `prompt_prefix_cache` | bool（**Wave D-T3**） | `true` 时 Fast/Deep + builtin local LLM + 内置 prompt 后端走 `build_prompt_segments`（稳定前缀在前）；Ollama 使用 `keep_alive`，llama-server 复用稳定前缀。目录/远程 prompt 后端保持自身 `build_prompt` 契约。亦可用 `OCLIVE_PROMPT_PREFIX_CACHE=1` 覆盖。bench 见 [`handoff/TTFT_BENCHMARK.md`](../../handoff/TTFT_BENCHMARK.md) |
| `fast_persistence` | `"legacy"` \| `"strong_only"`（**Wave E**） | 默认 **`legacy`**（Fast 仍全量巩固）；`strong_only` 时 Fast 闲聊不写 long_term / 好感 / 演化，**Quarrel / Apology / Confession / Praise** 仍正常写入。RFC [`RFC_TURN_THINKING_PERSISTENCE.md`](../rfc/RFC_TURN_THINKING_PERSISTENCE.md) |

示例（latency bench）：[`examples/distro-profiles/desktop-latency.oclive.toml`](../../examples/distro-profiles/desktop-latency.oclive.toml)。架构归类与 Deep 蒸馏路线图：[`handoff/DEEP_PROMPT_DISTILLATION.md`](../../handoff/DEEP_PROMPT_DISTILLATION.md)。

#### Chat Pro 默认行为（`distro_id = desktop`）

Release 安装包 bundled [`resources/distro-profiles/desktop.oclive.toml`](../../distros/desktop-tauri/resources/distro-profiles/desktop.oclive.toml) 与示例 [`examples/distro-profiles/desktop.oclive.toml`](../../examples/distro-profiles/desktop.oclive.toml) 对齐：

| 能力 | 默认 | 用户感知 |
|------|------|----------|
| **`[turn_thinking]`** | `default = "auto"` · `fast_persistence = "strong_only"` | 闲聊走 Fast；长句 / 高唤醒情绪 / Quarrel 链 → Deep；Fast 闲聊不写 long_term / favor（强事件仍写） |
| **流式回复** | 主 UI `sendMessageStream` → `POST /chat/stream` | 回复**逐字显示**（降低**感知延迟**）；SSE 失败自动回退 blocking `/chat` |
| **Persona capsule** | 角色包 `meta.deep_capsule_enabled` + `prompts/deep_capsule.txt` | Small 模型的 Fast/Deep 轮使用离线 capsule（沿用 Wave D 文件名） |
| **本地性能模式** | `[llm_runtime].mode = "performance"` | 可选 llama-server runtime + GGUF；组件缺失或首 token 前失败时降级 Ollama |

**Bench 区分**：`desktop-latency` profile（`event_impact_llm = false`）用于 TTFT 开发 bench；**正式用户默认**为 `desktop` profile。流式改善**感知延迟**，不改变 [`handoff/TTFT_BENCHMARK.md`](../../handoff/TTFT_BENCHMARK.md) 中 stream TTFT 数值定义。设置 → 常规 → 高级可关闭「流式回复」。

> 说明：六槽 `none` 语义见 [MODULE_NONE_SEMANTICS.md](./MODULE_NONE_SEMANTICS.md)。发行版关闭 Agent 可用 `host_flags.skip_agent` 或 `[plugin_backends] agent = "none"`。

### 3.3 Prompt / 记忆 / 后处理（P4 映射表）

| 字段 | `full`（桌面默认） | `concise`（VS Code 示例） |
|------|-------------------|---------------------------|
| `prompt.profile` | 角色包 + 引擎锚点完整叠加 | 额外叠加「简洁回复」overlay，不删减包级人设 |
| `memory.retrieval` | 默认 8 条相关记忆 | `light`：4 条（`HostProfile.memory_retrieval`） |
| `post_process.chain` | `standard` | `minimal`（强制 builtin `profile=minimal`；`enabled=false` 仍关闭） |
| `visual_presentation.mode` | 未设（跟随角色包 `visual_presentation.enabled`） | `off` \| `image_only` \| `stage_full`（已接线；Theater 可用 `stage_full`） |
| `user_identity.default_id` | 未设 | 会话无显式身份且非 sentinel 时作为默认 catalog id |
| `user_identity.allowed_ids` | 未设（不限制） | API 层拒绝列表外 id |
| `state_expression.favor_*` | 未设 | 按好感分档追加一句语气调节到 Prompt「角色当前状态」 |
| `[theater].director_plugin` | 未设（builtin prompt 模板） | Theater：`com.oclive.theater_director_official`；可被 env `OCLIVE_THEATER_DIRECTOR_PLUGIN` 覆盖 |

```toml
[theater]
director_plugin = "com.oclive.theater_director_official"
```

**合并规则（Theater Scene Director）**：仅当 profile 或 env 声明 `director_plugin` 且 `{app_data}/plugins` 中存在对应 manifest（`provides: theater_director`）时使用 directory RPC `theater.build_prompt`；否则 **builtin**（`scene_director.rs` / `patch_scene.rs`）。RPC 失败不 500，fallback builtin。

**合并优先级（User Identity）**：DB 会话/场景覆盖 → `HostProfile.user_identity.default_id` → catalog `default_identity_id` → legacy `user_relations.prompt_hint`。

**合并规则（Reply Post-Processor）**：`post_process.chain=minimal` 时 effective `builtin.profile=minimal`；remote/directory 仍可按角色包配置解析，失败降级 builtin → raw。

**合并规则（Visual Presentation）**：`visual_presentation.mode=off` 时宿主不下发 `performance_directive`；`image_only` 仅 `kind=image`；`stage_full` 允许 `live2d` / `rig3d` adapter（Theater）。

```toml
[visual_presentation]
mode = "off"   # off | image_only | stage_full
```

---

## 4. 合并规则（P4 实现对照）

内核在 `effective_plugin_backends_for_session` 路径上合并（`host_backends.rs`）：

1. **角色基础**：从 **`slot_registry`**（v2）或 legacy `plugin_backends` 解析六槽；`directory_plugins` 取自角色包。
2. **用户 LLM 设置 / env**：`resolve_effective_ollama_model` 等 override（在 profile 之前或之后按现有路径）。
3. **发行版 profile**：若 `distro.oclive.toml` 声明 **`[plugin_backends]`**，则 **`profile_override`（实现名 `apply_host_ceiling`）用 profile 值整表替换六槽**（`directory_plugins` **不**被 profile 覆盖）。
4. **`host_flags`**：`skip_agent = true` → 强制 `agent = none`；`skip_complex_emotion` / `slots.complex_emotion = off` → 跳过共景复杂情感。
5. **会话覆盖**：`PluginBackendsOverride` 在有效 backends 上再叠一层（仍受 startup health 与 none 语义约束）。

**与旧文档差异**：**不是** `role.apply_within_ceiling(host_ceiling)` 的「交集上限」模型。稳定发行版（vscode / theater）通过 **显式 `[plugin_backends]`** 锁定矩阵；实验场（desktop-chat）**省略**该段，角色蓝图说了算。详见 [DISTRO_DEFAULT_PLUGINS.md](./DISTRO_DEFAULT_PLUGINS.md) §2。

**单进程多宿主（v1）**：`HostProfile` 为进程级；最后 spawn 的 `OCLIVE_DISTRO_ID` + env 生效。同一 `:8420` 单写者；profile 冲突 → **replace 重启**，非热切换。

---

## 5. 发行版 bundled 内核与 shared 兜底

与 [DISTRO_KERNEL_LIFECYCLE.md](./DISTRO_KERNEL_LIFECYCLE.md) 对齐：

| 场景 | 行为 |
|------|------|
| **冷启动** | 优先 spawn **本发行版 bundled** `oclive-kernel-server` |
| **bundled 失败** | 同 `OCLIVE_APP_DATA` + `OCLIVE_DISTRO_PROFILE` + `OCLIVE_ROLES_DIR` 下 spawn **shared 兜底核**；`{app_data}/distros/chat-pro/plugins/` 自动复用 |
| **`promote`** | 开发者将本机构建写入 shared runtime — **维护通道**，非终端用户默认路径 |
| **Deferred** | 每发行版裁剪 binary · 进程内自升级（P3b） |

**术语**：旧称 **logical seed** = 今日 **发行版 bundled 全量核**（非裁剪体积）；spawn **优先级**由产品策略决定，与 discovery `SCORE_BUNDLED = 50` 数值无关。

---

## 6. 示例文件

| 发行版 | 路径 |
|--------|------|
| 桌面 | [`../../examples/distro-profiles/desktop.oclive.toml`](../../examples/distro-profiles/desktop.oclive.toml) |
| VS Code | [`../../examples/distro-profiles/vscode.oclive.toml`](../../examples/distro-profiles/vscode.oclive.toml) |

---

## 7. 验收（P1）

- [x] 本文档与 `plugin_backends` / slot 命名无冲突
- [x] 桌面、VS Code、theater 各有可复制的示例 TOML
- [x] 合并规则（整表替换 vs 省略 profile）与 `DISTRO_DEFAULT_PLUGINS` / `DISTRO_KERNEL_LIFECYCLE` / `KERNEL_SCHEDULER_RESCOPE` 交叉引用完整

---

## Related

- [DISTRO_KERNEL_LIFECYCLE.md](./DISTRO_KERNEL_LIFECYCLE.md)
- [KERNEL_SCHEDULER_RESCOPE.md](../../handoff/KERNEL_SCHEDULER_RESCOPE.md) — 进程调度收窄 · 与插件矩阵分责
- [DISTRO_DEFAULT_PLUGINS.md](./DISTRO_DEFAULT_PLUGINS.md) — 发行版默认六槽矩阵与三 personas
- [TRACK_VOICE_RECOGNITION.md](../../human-docs/team/TRACK_VOICE_RECOGNITION.md) — OS × ASR/TTS/webview 差异 · smoke 入口（≠ HostProfile）
- [TTFT_BENCHMARK.md](../../handoff/TTFT_BENCHMARK.md) — co-present 首字延迟复现
- [DEEP_PROMPT_DISTILLATION.md](../../handoff/DEEP_PROMPT_DISTILLATION.md) — Deep capsule · 前缀 KV 延续（Wave D）
- [VSCODE_DISTRIBUTION.md](../../handoff/vscode/VSCODE_DISTRIBUTION.md)
- [CROSS_HOST_MEMORY.md](../role-pack/CROSS_HOST_MEMORY.md)
- [OCLIVE_APP_DATA.md](./OCLIVE_APP_DATA.md)
