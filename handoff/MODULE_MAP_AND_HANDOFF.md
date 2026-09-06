# 模块注册表（Module Registry）

**最后更新**：2026-09-06
**SSOT 范围**：**模块定义 · 架构划分 · 槽位/设施/独立通道之间的联系 · 在边界内如何改**。  
**非 SSOT**：发版进度 → [`TECHNICAL_DEBT_INVENTORY.md`](./TECHNICAL_DEBT_INVENTORY.md) · 版本快照 → [`PROJECT_CURRENT_STATUS.md`](../creator-docs/getting-started/PROJECT_CURRENT_STATUS.md) · 关键文件路径 → [`BUS_FACTOR_NOTES.md`](./BUS_FACTOR_NOTES.md) · 文档分责 → [`handoff/README.md`](./README.md) §文档分层。

**改本文的条件**：新增/重命名模块或设施、变更六槽合并规则、新增编排行能力（非六槽）、或术语混淆需补对照表。**禁止**在本文堆进度叙事或复制 PLUGIN_V1 全文。

---

## 0. 五条铁律（关系骨架）

**先分清本体与装配**：OCLive 的最小概念核心是 **唯一回合/生命周期编排 + 权威状态提交约束与故障边界 + 六个稳定能力端口**。六槽是 `memory`、`emotion`、legacy `event`、`prompt`、`llm`、`agent` 六类可替换能力，不是六个平级决策内核，也不要求每种具体实现都启用。当前共景路径的健康门槛仍是 `prompt + llm`；其余槽位的 `none` / Noop 语义以 [`MODULE_NONE_SEMANTICS.md`](../creator-docs/kernel/MODULE_NONE_SEMANTICS.md) 和真实性矩阵为准。

### 0.1 小 Kernel、合同、实现、Host 与 Adapter 的权责

| 层 | 负责什么 | 不授予什么 |
|----|----------|------------|
| **小 Kernel（职责边界）** | 守住回合/生命周期公共语义、能力结果的有效性与合并/提交约束、错误语义和故障边界 | 不因此等同于已独立编译的 crate；Kernel v0 语义已收敛，具体 API、字段与物理拆分仍未定 |
| **六槽合同** | 规定 `memory` / `emotion` / `event` / `prompt` / `llm` / `agent` 能力端口的输入、结果和错误边界 | 不把槽实现变成平级状态权威，也不规定某个 SQLite、HTTP 或 RichRole 产品格式 |
| **具体槽实现** | 在合同和宿主授权内提供检索、分析、估计、组装、生成或动作结果；可由授权 Adapter 使用网络、存储、工具或 LLM | 不因获得资源访问就取得领域提交权、权限授予权或第二条回合管线；重试不会增加授权 |
| **Host** | 负责准备输入/上下文、绑定六槽、管理资源与权限、应用结果；在当前参考实现中由可信 Rust 服务与 composition root 在 Core 约束内执行产品状态提交和资源操作 | 不是 IPC 桥、Docker 或安全沙箱；不能降低 Core 不变量，不能授予 Session、LLM 或普通插件直接权威写权；不与 Distro 天然一对一 |
| **Adapter / 传输 / UI** | 把外部输入、能力调用和输出映射到 Host 获准的合同边界；持久化或资源 Adapter 可执行 Host 合法授权的写入/操作 | 不自行决定或批准领域状态更新；传输/UI 不是 Host，也不能绕过 Host 另建编排 |

**Kernel v0 公共契约（语义已收敛，API 未定）**：Host 每次回合提交一次归一化调用、本次已准入上下文、能力声明和资源范围；Kernel 校验调用、输入、能力使用、依赖、结果归属与终态合法性，并允许 Host 在有限调度自由内合法调度，但不规定固定六槽流水线、唯一调用链或固定并发。能力只能消费当前调用中已声明、已准入、有效且归属当前调用的输入；必要偏序只表达执行资格与结果可见性，依赖主 LLM 的 Prompt 结果须先于该主 LLM，且未提供、跳过、失败不自动互换。六槽状态不自动互转，槽不拥有领域提交权、权限授予权或第二条回合管线；`Agent handled` 只终止当前调用未完成的常规主链。单调用终态唯一，流式片段不等于生成、领域提交或外部送达成功，终态后的片段、结果与提交无效。Host 执行领域提交与资源操作，Kernel 只守住最小授权、调用归属与终态不变量；Event Ring 与 Runtime Event Stream 均属外围，不构成六槽依赖。完整语义与排除项见 [PURE_KERNEL_BOUNDARY.md](../creator-docs/getting-started/PURE_KERNEL_BOUNDARY.md)。

权责/成功口径须区分**生成成功**、**领域提交成功**与**外部送达成功**三类结果；这不暗定未来 Core 输出包含 delivery，也不新增“领域提交完成后才准 stream”的要求。`oclive_kernel_host::OcliveKernel` 是当前**完整嵌入运行时门面**，物理上仍装配 SQLite、Event Ring、HTTP 依赖和具体设施；它不能被等同为已经独立编译出来的最小 core。物理拆薄状态只看 [`TECHNICAL_DEBT_INVENTORY.md`](./TECHNICAL_DEBT_INVENTORY.md) 的 `K-CORE-BOUNDARY-01`。

| # | 铁律 | 一句话 |
|---|------|--------|
| 1 | **编排** | [`process_message`](../kernel/crates/oclive_kernel_host/src/domain/chat_engine/process_message.rs) → 共在 [`co_present`](../kernel/crates/oclive_kernel_host/src/domain/chat_engine/turn_pipeline/co_present.rs)；蓝图 **`steps[]` 不调度**。 |
| 2 | **六槽** | `slot_registry` → `PluginBackends` → `PluginHost::resolve_for_role`；键 **`memory` · `emotion` · `event` · `prompt` · `llm` · `agent`**。其中 legacy `event` 只负责 `event.impact` 估计，结果进入通用 Event Ring。 |
| 3 | **记忆三套存储** | 聊天日志 **`chat_messages`** ≠ **`short_term_memory`** ≠ **`long_term_memory`**；删聊天 **不清** 记忆表。 |
| 4 | **配置四层** | 角色内容层 → 包内蓝图配置层 → 发行版 `HostProfile` → `SessionCache` 会话内存覆盖；分责 [`ROLE_PACK_BOUNDARY.md`](./ROLE_PACK_BOUNDARY.md)。角色运行时状态可另行持久化，但不是槽位配置覆盖。 |
| 5 | **图纸 ≠ 资源执行** | 蓝图只声明能力意图；宿主编译内部 `ExecutionPlan`；Resource Coordinator 根据真实设备和策略发放资源租约。 |

**六槽版本纪律**：v1 的六类端口是稳定分类。新能力通常应先归入某一槽的实现、设施、独立通道或宿主能力；不得把普通扩展顺延命名为“第 7 槽”。若真实场景证明必须改变六槽分类，它是需要迁移、兼容期和 Breaking 说明的核心契约修订，而不是常规插件扩展。

Event Ring 的 wire、注册、权威与主动授权契约只维护于 [`EVENT_RING.md`](../creator-docs/plugin-and-architecture/EVENT_RING.md)；本文只登记它与模块的关系。

`RuntimeEventTrace` 是默认关闭的基础设施观察器：它只在成功 Ring dispatch 后非阻塞记录脱敏事实头，不是六槽、Event 模块、Event 决策模块或 Runtime Event Stream，也不向 Prompt/记忆/主动回合提供输入。完整状态见 [`RFC_RUNTIME_EVENT_STREAM`](../creator-docs/rfc/RFC_RUNTIME_EVENT_STREAM.md) 与 `K-EVENT-STREAM-01`。

`oclive_kernel_types::runtime_event_stream` 目前只是 Stage A.1 纯契约层；A.2.1 以版本化夹具冻结本地持久化设计，A.2.2.1 再以 OneBot v11 规范样本冻结 ACK、不确定投递、撤回非擦除与多宿主否定证据。A.2.2.2-R1/R2 只有独立开发者探针取得的一次真实 QQ 同步成功路径和一次真实提交后超时/显式对账路径；R3 只验证 AES-256-GCM 私有 locator 的跨进程存续，R4 只验证 NapCat 群历史扩展下的一次受控 ACK→locator 窗口恢复，R5 只用 synthetic-only 独立 SQLite 验证同一宿主内的跨进程 owner lease 与 fencing，R6 只冻结私聊或无可信历史能力时的失败关闭裁决。它们都不是 Production Output adapter/store、私聊/通用 OneBot 自动恢复或多宿主协调能力。当前仍没有 Production 存储/读取端口、consumer loop、Output 接线、Replay、Prompt 或回合接线，宿主级密钥恢复与多宿主 owner lease/fencing 也未关闭，不能被列为已运行模块。

**稳定宿主入口**：可信 Rust composition root 通过 [`oclive_kernel_host::OcliveKernel`](../kernel/crates/oclive_kernel_host/src/role_kernel.rs) 进入角色加载、回合、Event Ring 与关闭；HTTP/Tauri 仍是薄传输适配。`AppState` 是内部装配根，不是发行版/硬件集成合同；新增宿主入口必须委托同一 `process_message` / `process_proactive_turn`，不得复制 pipeline。

---

## 1. 术语对照（防「三层」混淆）

| 说法 | 指什么 | 深入 SSOT |
|------|--------|-----------|
| **记忆三套存储** | 聊天日志 · STM · LTM | [`CHAT_STORAGE_ARCHITECTURE.md`](./CHAT_STORAGE_ARCHITECTURE.md) |
| **Prompt 三区块** | 系统 / 角色 / 用户 Tier0 + 页脚 | `prompt_builder/mod.rs` |
| **架构四大类** | 1–6 后端 · 第 N 设施 · 独立通道 · 插件实现 | 本文 §2 |
| **集成三层** | UI/语音 → HTTP → 内核 | `human-docs/team/SCOPE_AND_BOUNDARIES.md` |
| **测试三层** | 协议 / 编写器 / 插件范式 | `creator-docs/testing/OVERVIEW.md` |

---

## 2. 模块四大类（划分）

| 大类 | 与六槽折叠 `PluginBackends` 的关系 | 编号 | 改动的文档 SSOT |
|------|------------------------------------|------|-----------------|
| **后端模块（六槽）** | 蓝图六种稳定 `slot_registry.type` 折叠为六字段 | 第 1–6 模块 | **本文 §3–§8** + [`PLUGIN_V1.md`](../creator-docs/plugin-and-architecture/PLUGIN_V1.md)（DTO/顺序） |
| **设施子模块** | **不进入六槽折叠**；可有自有蓝图声明（如 `complex_emotion`） | 第 1–4 设施 | **本文 §9** + 各 RFC |
| **独立通道能力增强** | **不进入六槽折叠**；走自有 Resolver / 锚点 | 注册表 `id` | **本文 §10** + [`RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md`](../creator-docs/rfc/RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md) |
| **后端模块插件** | 实现某槽的 `backend`，沿用该槽字段 | 无独立号 | [`DIRECTORY_PLUGINS.md`](../creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md) · [`SLOT_BACKEND_REALITY_MATRIX.md`](./SLOT_BACKEND_REALITY_MATRIX.md) |

**对外叙述**（产品文案、编号脚注）：[`OCLIVE_ARCHITECTURE_OVERVIEW.md`](../creator-docs/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md) — **不**在此重复长文。

**蓝图 `extensions` 不是第五类模块**：它是跨上述分类的声明/装配外壳。扩展声明的 Capability 必须解析为已有六槽实现、设施、独立通道或宿主 UI/硬件消费者；仅出现一个未知 JSON 节点不构成新模块。目标契约见 [`RFC_BLUEPRINT_EXTENSION_AND_RESOURCE_COORDINATION.md`](../creator-docs/rfc/RFC_BLUEPRINT_EXTENSION_AND_RESOURCE_COORDINATION.md)。

---

## 3. 六槽解耦机制（共用）

六槽合同的目标边界是稳定能力结果、调用边界和错误语义；目标上不规定特定的 SQLite、HTTP、RichRole 或其他产品载体。具体实现可以经授权 Adapter 使用网络、存储、工具或 LLM；这属于能力实现的资源访问，不改变 Core 的领域提交权边界。失败重试沿用原授权，不会获得额外权限。当前公开的 `oclive_kernel_types::PromptInput` 仍有 `role: &Role` 耦合，这是已知的参考运行时实现事实，不在本文替 Kernel v0 API 作决定。

### 3.1 三层解耦

| 层 | 含义 |
|----|------|
| **编译期** | 各槽 `trait` + `PluginHost`；换实现 **不改** `process_message` 顺序 |
| **配置期** | 当前参考宿主 v2/v3/v4 蓝图的 `slot_registry` 多实例 → 折叠 `PluginBackends`（同 `type` **last-wins**，`position` 大者优先）；该格式族的新 Stable 样例使用 v4，这条装配链不属于内核最小角色 contract |
| **运行期** | `set_session_slot_override` 叠在有效快照上（**不写盘**） |

### 3.2 有效 backends 解析链

```text
角色包运行配置快照
  ├─ legacy v1：`manifest.json` + `settings.json.plugin_backends`
  └─ v2/v3/v4：`pipeline.ocblueprint.slot_registry`
       （多实例 → 有效注册表 → 六种稳定 type 折叠为 `PluginBackends`；同 type last-wins）
       ↓
用户 LLM 设置（DB `app_settings`）/ `OCLIVE_LLM_BACKEND` 等 env
       ↓
发行版 `distro.oclive.toml` `[plugin_backends]` 整表替换（若声明）
       ↓
`host_flags`（`skip_agent` → `agent=none`；`skip_complex_emotion` 等）
       ↓
**内存** `SessionCache` 会话槽 override（`set_session_slot_override` / UI「会话后端」；**不写盘**）
       ↓
`startup_health`（Remote 槽探测；失败可降级并写 `startup_warnings`）
       ↓
`PluginHost::resolve_for_role` → `ResolvedRolePlugins`
```

| 层 | 代码锚点 | 说明 |
|----|----------|------|
| **纯解析 SSOT** | `oclive_kernel_runtime` · `plugin_resolution.rs` | `resolve_session_plugin_backends`（legacy/v2 + env + host ceiling + session override；**无 DB/I/O**） |
| 折叠 v2 | `slot_resolver.rs` · `plugin_backends.rs` | `slot_registry_to_plugin_backends` |
| 发行版/env 合并 | `host_backends.rs` · `effective_llm_model.rs` | HostProfile + env 天花板；host 调 runtime 纯函数 |
| 会话 override | `state/session_cache.rs` · `service/role/slot_session.rs` | **进程内** `PluginBackendsOverride` |
| 每回合快照 | `state/effective_session_config.rs` | `EffectiveSessionConfig`（`process_message` 每轮一次） |
| 诊断输出 | `build_plugin_resolution_debug_info` · `oclive doctor config-resolve` | CLI **默认** runtime 纯路径；`--via-host`（feature `diagnostics-host`）可选全 host bootstrap；**禁止第二套解析** |
| 装配 | `plugin_host.rs` · `backend_registry.rs` | `resolve_for_role` |
| 共在调用 | `slot_runner.rs` · `co_present.rs` | 六槽 + `complex_emotion` 设施 |

### 3.3 多实例合并

| 槽 | 策略 |
|----|------|
| memory | 去重合并检索 |
| llm | 非流式读取最后一个 LLM 实例的 `policy`：`ensemble` = 串行 last-wins、`fastest` = 首个成功、`fallback` = 按序首个成功；流式当前统一串行 last-wins |
| agent | 当前只执行折叠后的单一 Agent；多实例/`plugins[]` 仅收集诊断 ID，工具并集尚未实现（`K-AGENT-MERGE-01`） |
| emotion / event / prompt / complex_emotion | 串行 last-wins；memory 与 LLM 例外见上 |

**backend 真值矩阵（24 格）**：只维护于 [`SLOT_BACKEND_REALITY_MATRIX.md`](./SLOT_BACKEND_REALITY_MATRIX.md)，本文 **不** 复制该表。

`groups` / `module_relations`：仅 UI 派生边；**禁止** `module_relations` 落盘。

---

## 4. 第 1 模块 · `memory`

| 项 | 内容 |
|----|------|
| **定义** | `MemoryRetrieval` 为 Prompt 提供 **相关记忆检索**；STM 写入与 LTM 归档由内核编排/持久化实现负责，不属于该 trait 的写权限 |
| **`plugin_backends` 键** | `memory` |
| **Trait** | `MemoryRetrieval`（`oclive_kernel_contracts`） |
| **合法 backend** | `builtin` · `remote` · `directory` · `local` · `none` |
| **Builtin** | `BuiltinMemoryRetrieval` + `MemoryEngine`（STM/LTM 衰减、阈值） |
| **主链 hook** | 槽位在 `turn_pipeline/pre.rs` 检索；内核在 `post_llm` 写入 STM/LTM |
| **Event Ring 接入** | 本轮已检索且与当前用户句相关的最高候选由 `builtin.memory_recollection` 提交 `kernel.memory.recall.candidate`（事件只携带记忆 ID、置信度与相关度，不复制正文）；`builtin.event_decision` 按注册基础权重与证据决定是否发出 `kernel.memory.recollection.activated`。只有已激活回忆进入专门的长文本回复上下文，默认 `weave`，且 TTL 固定为当前一轮；没有候选或未采纳时沿用原记忆 Prompt 行为 |
| **与聊天存储** | **无关** — `chat_messages` 不进 MemoryEngine；回放见 `replay_memory_extraction` |
| **合并** | 多 memory 实例 → 去重合并 |
| **可移植边界** | `.ocmemory` 只携带 `memory_seed` + LTM；STM 是可重建缓存，临时局面状态不属于 memory/persona 迁移 |
| **`none`** | `NoopMemoryRetrieval`；共景路径允许，不检索长期记忆并返回空列表（见 MODULE_NONE_SEMANTICS） |
| **允许改** | 检索算法、decay、archive 阈值、remote/directory 协议 |
| **禁止** | 用聊天记录表当记忆真源；角色任务改 `slot_registry` |

**职责 / 不授予**：本模块负责检索、排序和形成记忆上下文；它不是整个 `MemorySystem`，不因此获得 STM/LTM 写权限，也不取得领域状态提交权。

**记忆三套存储（与第 1 模块配合）**：

| 存储 | 表 / 组件 | 进 Prompt |
|------|-----------|-----------|
| 聊天日志 | `HybridConversationStore` · `chat_*` | **否** |
| 短期 | `short_term_memory` | **是** |
| 长期 | `long_term_memory` | **是** |

---

## 5. 第 2 模块 · `emotion`

> **T0 / T1+ 分层与情感·展示分轨（Draft）**：[RFC_MODULE_MVL_AND_AFFECT_ARCHITECTURE.md](../creator-docs/rfc/RFC_MODULE_MVL_AND_AFFECT_ARCHITECTURE.md) — T0 = `analyze`；T2 角色模拟、T3 `display_metrics` 为扩展；好感数值非 Prompt 力学。

| 项 | 内容 |
|----|------|
| **定义** | 分析 **用户句** 情绪（**T0**）；可选角色情绪模拟（**T2**）与展示快照（**T3**） |
| **键** | `emotion` |
| **Trait** | `UserEmotionAnalyzer` |
| **Backend** | `builtin` · `remote` · `directory` · `none` |
| **主链 hook** | `pre.rs` → `EmotionResult` → Prompt · Turn Thinking Auto 路由 |
| **与复杂情感** | **不同模块** — 复杂情感是 **第 1 设施**；本槽的用户情绪只是其降级证据之一，本轮角色回复情绪以有效主 LLM `[EMO]` 为权威 |
| **允许改** | 分析器、remote 协议 |
| **禁止** | 把 `slot_registry` 中的 `complex_emotion` 设施实例冒充稳定六槽，或写入 `plugin_backends` 六键 |

**职责 / 不授予**：本模块负责分析用户输入情绪；该结果不是全部角色状态的权威，也不因此取得关系、好感或其他领域状态的提交权。

---

## 6. 第 3 模块 · `event`

| 项 | 内容 |
|----|------|
| **定义** | Event Ring 的 legacy `event.impact` 子槽：估计本回合 **事件类型** 与 **影响因子**，再以 `kernel.chat.event_impact.estimated` 信封进入外环，驱动后续性格演化与好感 |
| **键** | `event` |
| **Trait** | `EventEstimator` |
| **Backend** | `builtin` · `remote` · `directory` · `none` |
| **Builtin 双路径** | ① **规则** `EventDetector` / `estimate_event_impact_rules_only` ② **LLM** `estimate_event_impact`（`generate_tag`） |
| **LLM 开关** | **`HostProfile.event_impact_llm`**（非六槽）；Fast 轮 Turn Thinking **不调** LLM 路径 |
| **主链 hook** | `co_present` `EventEstimate` stage → `EventRing` 兼容桥 → `PersonalityEngine::evolve_by_event`；无注册事件模块时估计结果逐字段不变 |
| **允许改** | 规则表、LLM 提示、remote |
| **禁止** | 把 Turn Thinking 登记为第七槽 |

**职责 / 不授予**：本模块负责估计事件类型与影响因子；它不是 Event Ring，也不是 Runtime Event Stream，不自行写入好感或关系状态。

---

## 7. 第 4 模块 · `prompt`

| 项 | 内容 |
|----|------|
| **定义** | 组装发往 LLM 的 **完整 prompt 字符串**（Tier0 三区块 + 设施段落 + 页脚） |
| **键** | `prompt` |
| **Trait** | `PromptAssembler` → 内置 **`PromptBuilder::build_prompt`** |
| **Backend** | `builtin` · `remote` · `directory` · `none`（共景 **禁止** none） |
| **Tier0 人设真源** | **`core_personality.txt`**（非 `prompts/system.md`） |
| **页脚** | `reply_quality_anchor`（包级 **可替**）+ **`KERNEL_DIALOGUE_GUARDRAILS`**（**不可替**） |
| **主链 hook** | `co_present` `BuildPrompt` · `PromptInput` |
| **Wave D persona capsule** | **`prompts/deep_capsule.txt`**（兼容文件名）— [`DEEP_PROMPT_DISTILLATION.md`](./DEEP_PROMPT_DISTILLATION.md) · **Small 模型 Fast/Deep 已接线** |
| **允许改** | 段落公式 `sections.rs`、overlay（concise profile） |
| **禁止** | 运行时 LLM 压缩 prompt；用 capsule 替换 guardrails |

**职责 / 不授予**：本模块负责组装已由宿主准入的上下文；它不获得任意状态读取权，也不拥有身份或授权的所有权。

---

## 8. 第 5 模块 · `llm`

| 项 | 内容 |
|----|------|
| **定义** | 主对话 **文本生成**（含 stream） |
| **键** | `llm` |
| **Trait** | `LlmClient` |
| **Backend** | `ollama` · `remote` · `directory` · `none`（共景 **禁止** none） |
| **发行版 builtin 实现** | `[llm_runtime].mode=performance`：`llama-server/GGUF → Ollama`；不新增角色包 backend 枚举 |
| **合并** | 多 llm 非流式按最后一个 LLM 实例的 `policy` 选择 `ensemble`（串行 last-wins，默认）/ `fastest` / `fallback`；流式当前统一串行 last-wins |
| **主链 hook** | `co_present` generate / stream |
| **性能闭环** | `performance_llm.rs` 管理 runtime pack/进程/熔断；`openai_compatible_llm.rs` 解析 SSE；GGUF 路径与 Ollama fallback model 分开保存 |
| **流式回退规则** | 首 token 前失败可回退 Ollama；已产生 token 后必须返回错误，不得重跑造成重复文本/语音 |
| **允许改** | Ollama 适配、llama-server builtin 适配、directory RPC、TTFT 客户端选项 |
| **禁止** | UI 内二次调 LLM 选立绘 |

**职责 / 不授予**：本模块负责生成主对话文本及其流；生成结果不等于领域状态提交，也不为自身或其他实现授予权限。

---

## 9. 第 6 模块 · `agent`

| 项 | 内容 |
|----|------|
| **定义** | ReAct / MCP 工具编排；可 **短路** `process_message` |
| **键** | `agent` |
| **Trait** | `AgentProvider` |
| **Backend** | `builtin` · `remote` · `directory` · `none` |
| **合并** | 当前折叠后只执行单一 Agent；`merged_agent_directory_plugin_ids` 仅供诊断，`wrap_agent_if_merged` 为 no-op。多 Agent 工具并集见 `K-AGENT-MERGE-01` |
| **发行版** | `host_flags.skip_agent` → 强制 `none` |
| **MCP** | `{app_data}/mcp-servers/*.json` · 须 `network:*` / `process:spawn` 授权 |
| **允许改** | Agent 协议、MCP 客户端、调试 trace |
| **禁止** | 跳过 MCP 授权；把 ASR 写进 agent 槽 |

**职责 / 不授予**：本模块负责在已授权范围内执行工具调用和多步任务；当前实现可在本回合内执行工具，仍复用既有回合入口，不构成 proposal-only 规则或第二条 pipeline。

---

## 10. 第 N 设施子模块（编排行内 · 非六键）

| # | 名称 | 输入 / 输出 | 主链锚点 | 默认 | 改动 SSOT |
|---|------|-------------|----------|------|-----------|
| **1** | 复杂情感 | 上一轮 hint + 用户情绪/上下文降级证据 + 主 LLM `[EMO]` → 本轮回复情绪与下一轮 `narrative_hint` | `pre.rs` 读旧 hint → `run_middle.rs` 只给 Prompt 去内容连续性信号 / Fast 强度 → `post_llm.rs` 解析并剥离 marker、插件兜底、持久化 | **省略 / `none` = hint 读写关；`builtin` = hint 读写 + Fast 强度；`remote` / `directory` = 再加 post 降级 provider**（发行版仍可 skip） | `NARRATIVE_HINT_CONTRACT.md` · `complex_emotion.rs` · `post/post_llm.rs` · `complex_emotion_store.rs` |
| **2** | 专家模型 | 条件 → 专家子流程；`slot.lora.apply` 选择预声明的 directory LLM adapter | `expert_routing.json` · `dual_core` · `post::run_main_llm*` | **可选启用；默认关** | TECHNICAL_DEBT §2 |
| **3** | 立绘 | 封闭 catalog → `visual_state_id` | `post_llm` · 表现导演 LLM | **平台默认关；角色包可 opt in** | RFC_PORTRAIT |
| **4** | 视觉表现 | `visual_state_id` → `performance_directive` | 宿主 UI 帧循环 · **无** AI 选图 | **平台默认关；角色包可 opt in** | RFC_VISUAL_PRESENTATION |

**禁止**：上述任一写入 `plugin_backends` 六键或蓝图六键别名。

---

## 11. 独立通道能力增强（注册表 · 非六槽）

| `id` | 职责 | 锚点 | 进 `process_message`？ |
|------|------|------|------------------------|
| `event_ring` | 通用内核单次 dispatch 外环与信封/路由权威：模块通过 `EventModuleDeclaration` 只声明订阅、允许发射事件与优先级，并只提交 `EventDraft`；可信注册调用通过独立 `EventModuleRegistryPolicy` 分配基础影响权重与 `fail_fast` / `isolate` 故障边界，模块不能自报权重或故障策略。Ring 签发来源/权重/顺序/因果链后按 `(priority, module_id)` 路由；隔离型模块的处理错误或非法输出会被原子拒绝并隔离，诊断只记录状态和失败次数。主动输入链为来源绑定 emitter 提交 `kernel.proactive.turn.proposed`，`builtin.proactive_turn_decision` 按注册权重、置信度与紧迫度生成带因果链的 `kernel.proactive.turn.authorized`；宿主只为该权威事件签发一次性 `ProactiveTurnPermit`，Ring dispatch 返回后由 `process_proactive_turn` 消耗 Permit 并以独立 `TurnInput::ExternalObservation` 进入 origin-aware 共景 Prompt，`user_message` 保持为空且不写用户聊天状态。目录插件以独立 `eventRing` manifest 建议 + `event_ring.handle` RPC 接入：宿主限制精确安全订阅、插件自有发射命名空间、权重上限与超时，并在首次扫描/重扫时同步；当前目录插件不能读写 proactive 内核事件，也拿不到主动 emitter/Permit，且不复用前端 `bridge.events`。`EventRingDiagnostics` 不包含 payload、metadata 值或 stream key。每个 `AppState` 独立、内存有界、无数据库写者，权重不控制执行顺序；该“权威”不包含提案采纳或角色状态提交 | `oclive_kernel_types::EventDraft` / `EventEnvelope` / `EventModuleRegistryPolicy` / `ProactiveTurnProposal` / `EventRingDiagnostics` · `oclive_kernel_contracts::EventEmitter` / `EventModule` / `EventModuleRegistrar` · `domain/event_ring/` · `domain/chat_engine::process_proactive_turn` · `infrastructure/directory_plugins/event_ring.rs` | **分路径**：legacy `event.impact` 与 memory recollection 在 `process_message`；主动链经 Ring dispatch 后走受 Permit 限制的 `process_proactive_turn`；目录模块只随获准 Ring dispatch 被调用。后两者不构成第二套 `process_message` |
| `user_identity` | 用户是谁 | `user_identities/` · pre | **是**（pre 段落） |
| `reply_post_process` | 回复润色/改写 | `config.json` · `post_llm.rs` 内 semantic/state 消费后、聊天写入前 | **是**（post） |
| `reply_mode` | 回复分段与展示节奏 | `config.json` · display 阶段（`reply_post_process` 之后、聊天写入前） | **是**（post） |
| `theater_director` | 剧场场景生成 | `POST /theater/scene` | **否**（圈外 API） |
| **`voice.asr`** | 麦克风 → 文本（ASR，基础）+ 可选情感 TTS（扩展 · 默认关） | 宿主 `chat_toolbar` + **`plugin_rpc_invoke`** → [`VOICE_ASR_SUBMIT_EVENT`](../distros/shared/src/lib/voiceAsrEvents.ts) → `send_message`；`message:sent` / 流式首句 → **`voice.speak`**（须 `tts_expansion_enabled`） | **否** |
| **`voice.director`** | 人设 → **`voice_directive`**（`rules-v1` · `emo_text` · `ref_map`） | 插件 RPC **`voice.build_directive`** | **否** |
| **`voice.synth`** | `reply` + directive → 音频（CosyVoice2 / cloud） | **`voice.speak`** · `voice.probe_tts` · `voice.warm` · 模型 DLC | **否** |

**`voice.asr` 插件 SSOT**：[`distros/chat-pro/plugins/com.oclive.voice.asr/`](../distros/chat-pro/plugins/com.oclive.voice.asr/) · **v0.5** · `provides: ["voice.asr"]` · RPC 见插件 README · 开发烟测 [`examples/voice-loop-minimal/`](../examples/voice-loop-minimal/)（Piper 仅 `--tts-sherpa` dev 路径）。导演 + 发声器已合入同插件，见 [`ARCHITECTURE_DECOUPLING_PANORAMA.md`](../human-docs/team/ARCHITECTURE_DECOUPLING_PANORAMA.md) §6–§7。

RFC：[`RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md`](../creator-docs/rfc/RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md) · 现行行为以源码与本注册表为准；Phase 2 过程记录已归档。

---

## 12. 编排行策略（非模块号 · 易与六槽混淆）

| 能力 | 归类 | 配置 | 代码 |
|------|------|------|------|
| **Turn Thinking** Fast/Deep/Auto | HostProfile 策略 | `[turn_thinking]` · `distro.oclive.toml` | `turn_thinking.rs` |
| **Turn Thinking 持久化分流** `fast_persistence` | HostProfile · `legacy` \| `strong_only` | `[turn_thinking].fast_persistence` | `turn_thinking.rs` · `co_present` / `post` · RFC [`RFC_TURN_THINKING_PERSISTENCE.md`](../creator-docs/rfc/RFC_TURN_THINKING_PERSISTENCE.md) |
| **Turn Thinking 包级路由** OR/AND · latch · ephemeral | 角色包 `config.json` → `turn_thinking` | 合并 Host OR + pack OR/AND | `turn_thinking.rs` · `035_turn_thinking_runtime.sql` · RFC §8–12 |
| **叙事连续性微状态机** | 场景内编排行状态（不属于人格档案或第七槽） | `scenes/{scene_id}/scene.json` → `continuity`；旧包缺省关闭 | `narrative_continuity.rs` · `036_narrative_continuity.sql` · `co_present` 动态 `extra_sections` → `post` 最终可见回复显式标记迁移 |
| **`ModelTier`** Small/Large | 编排行 · Ollama 模型启发式 | — | `model_tier.rs` |
| **`PersonaSource`** FullCore/PersonaCapsule | 编排行 · Small 模型 Tier0 选择 | 角色 `meta.deep_capsule_enabled` + `prompts/deep_capsule.txt` | `model_tier.rs` · `co_present` |
| **`event_impact_llm`** | HostProfile 开关 | `[host_flags]` | `event_impact_ai.rs` |
| **`prompt.profile` concise** | HostProfile overlay | `[prompt]` | `DISTRO_CONCISE_PROMPT_OVERLAY` |
| **PersonalityEngine / 好感（legacy 数值）** | 无编号设施 · **目标废弃** | 角色 `evolution` · `role_runtime` | `personality_engine.rs` · 见 [RFC_MODULE_MVL_AND_AFFECT_ARCHITECTURE.md](../creator-docs/rfc/RFC_MODULE_MVL_AND_AFFECT_ARCHITECTURE.md) §6 |
| **PluginHost** | 无编号设施 | — | `plugin_host.rs` |
| **remote_stub / remote_life** | 场景模式分支 | 场景 + `remote_presence` | `process_message` 分支 |

TTFT / Deep capsule：**设计** [`DEEP_PROMPT_DISTILLATION.md`](./DEEP_PROMPT_DISTILLATION.md) · **bench** [`TTFT_BENCHMARK.md`](./TTFT_BENCHMARK.md) — 不在此展开进度。

---

## 12.1 蓝图扩展、执行计划与资源协调

| 概念 | 职责 | 禁止 |
|------|------|------|
| 蓝图扩展外壳 | `capability`、可选 `provider`、`required`、外置 `config_ref` | 携带卸载进程、固定显存或任意脚本命令 |
| Capability Provider | 实现业务能力并声明权限/兼容性 | 仅有配置、没有生产者或消费者便宣称交付 |
| `ExecutionPlan` | 合并蓝图、`HostProfile`、用户设置、能力注册表与设备状态；进程内使用 | 落盘进角色包或允许第三方直接写 |
| Resource Coordinator | 集中预算、租约、优先级、压力和降级决策 | 传输 token/PCM/渲染帧或包含各模块业务逻辑 |
| Resource Adapter | LLM、语音、渲染等领域的探测与 start/suspend/unload/degrade 执行 | 自行绕过中央租约偷偷预热 |

```text
Blueprint + HostProfile + user/session + Capability Registry
                          ↓
                     Plan Compiler
                          ↓
                     ExecutionPlan
                          ↓
                 Resource Coordinator
                    ↓       ↓       ↓
               LLM adapter Voice adapter Render adapter
```

资源协调是**无编号控制面设施**，不是第七后端模块、不是独立通道注册表项，也不是 [`resolve_kernel_action`](./KERNEL_SCHEDULER_RESCOPE.md) 的进程 attach/replace 调度。新扩展首先实现 Capability Provider；只有占用共享 GPU/内存/受管进程时才增加 Resource Adapter。完整字段、缺失语义和实施顺序只维护于 [蓝图扩展与资源协调 RFC](../creator-docs/rfc/RFC_BLUEPRINT_EXTENSION_AND_RESOURCE_COORDINATION.md)。

**当前实现边界（2026-08-01）**：宿主已能从 v4 `extensions`、有效六槽、`HostProfile`、目录插件 manifest、插件启停状态、依赖与高危授权编译**只读、进程内** `ExecutionPlan`；计划不会启动 Provider，也不会写回角色包。只有宿主登记了真实消费者的 capability 才可进入 ready，插件单独声明任意 `provides` 不构成可执行能力。首个登记项为 Chat Pro 的 `voice.asr`；其它发行版会对同一声明给出 `capability_consumer_unavailable`。

Resource Coordinator 已落地 NVIDIA 多设备、系统 RAM 与 CPU snapshot，原子 admission/lease/priority/pressure，以及带超时、取消清理和老化防饥饿的公平准入队列。宿主 Resource Adapter Registry 登记 managed llama-server、observe-only Ollama、performance 活动观察器和官方 bundled CosyVoice2 的控制权、运行档位、驻留能力与真实生命周期动作；资源诊断 v5 同时返回注册来源、队列和租约状态。`HostProfile` 的 `require` / `residency` / `coexist` / `exclusive` / `yield_then_run` / `fallback` 仍先经注册表静态校验，再由实时租约、GPU/RAM/CPU 容量和真实控制器编译只读 `candidate_plan`；查看诊断不会执行。Performance llama 的 `gpu_full`、`gpu_balanced`、`cpu_compatibility` 三档会改变实际 `--n-gpu-layers` 并在准入拒绝后逐档回退。自动抢占只选择优先级更低、明确声明可逆动作且拥有精确 requester → target → operation 授权的 managed 适配器；失败逆序回滚，工作完成后逆序恢复，observe-only 外部进程不会被假装卸载。第三方扩展可经所有者命名空间约束的进程内 `ResourceAdapterRegistrar` 登记 adapter facts 与单写者控制器，但目录插件 manifest 尚无资源声明/自动注册字段，登记也不会自动取得跨适配器控制权。通用契约已加入 `render` / `compute` / `hybrid` 资源域并覆盖渲染适配器容量与抢占恢复测试；Chat Pro 当前仍由 `Live2DStageAdapter` 明确回退 PNG，没有 bundled Live2D runtime，不能宣称 Live2D 已接管资源。bundled CosyVoice 仍采用“请求卸载 → 侧车确认 → 撤销租约”，未确认时保留状态；Performance LLM 暂停仍通过统一请求门排空在途 primary/fallback。桌面保持 thin shell，真实生命周期操作由内核单写者执行；纯计划编译/CLI 不碰硬件，保留 `not_evaluated`。本里程碑远端主 CI 与严格审计已在 PR #147 对应实现上通过；尚未落地的是外部批量计划 API、目录 manifest 资源声明、实际 bundled Live2D runtime、不可控外部进程完整故障矩阵与长时间真实进程/硬件 soak，状态见 `K-RESOURCE-COORD-01`。

实现锚点：[`execution_plan.rs`](../kernel/crates/oclive_kernel_host/src/domain/execution_plan.rs)（能力纯编译）· [`resource_plan.rs`](../kernel/crates/oclive_kernel_host/src/domain/resource_plan.rs)（资源候选计划纯编译）· [`capability_registry.rs`](../kernel/crates/oclive_kernel_host/src/infrastructure/capability_registry.rs)（能力适配）· [`resource_adapter_registry.rs`](../kernel/crates/oclive_kernel_host/src/domain/resource_adapter_registry.rs) / [`resource_coordinator/mod.rs`](../kernel/crates/oclive_kernel_host/src/domain/resource_coordinator/mod.rs)（资源目录、控制授权与批量执行基础）· [`resource_coordination.rs`](../kernel/crates/oclive_kernel_contracts/src/resource_coordination.rs)（快照/单写者控制器端口）· [`performance_request_gate.rs`](../kernel/crates/oclive_kernel_host/src/infrastructure/performance_request_gate.rs)（Performance LLM 请求准入/排空/恢复竞态）· [`service/resource_coordination.rs`](../kernel/crates/oclive_kernel_host/src/service/resource_coordination.rs)（Voice 准入/抢占/确认恢复）· [`service/execution_plan.rs`](../kernel/crates/oclive_kernel_host/src/service/execution_plan.rs)（激活门禁/只读查询）· [`models/execution_plan.rs`](../kernel/crates/oclive_kernel_types/src/models/execution_plan.rs) / [`models/resource_coordination.rs`](../kernel/crates/oclive_kernel_types/src/models/resource_coordination.rs)（公共诊断 DTO）。

---

## 12.5 前端 ↔ 内核契约边界（2026-07-13）

| 主题 | SSOT | 消费方 | 不变式 |
|------|------|--------|--------|
| **错误码** | [`AppError::code`](../kernel/crates/oclive_kernel_types/src/error.rs) + `http_chat_codes` | `distros/shared/src/api/generated/kernelErrorCodes.ts` · [`ERROR_CODES.md`](../creator-docs/getting-started/ERROR_CODES.md) · `scripts/check-error-codes-drift.mjs` | 前端 **禁止** 解析 `message` 文本分支（legacy `[CODE]` 除外）；用 `code` + 可选 `context.kind` |
| **错误 JSON** | [`KernelErrorBody`](../kernel/crates/oclive_kernel_types/src/error.rs) | Tauri `invoke` · HTTP `/chat` · `helpers.ts` | 字段名与形状内核权威；发行版只做 i18n 映射 |
| **热路径 DTO** | [`oclive_kernel_types::models::dto`](../kernel/crates/oclive_kernel_types/src/models/dto/mod.rs) | `distros/shared/src/api/*.ts`（过渡期为手写镜像） | 回复字段为 **`reply`**；六槽键为 `plugin_backends` / `slot_registry.type` |
| **六槽清单** | 内核 resolver + `PluginBackends` | `slotRegistry.ts`（待导出替换硬编码） | 前端不得假设 Chat Pro 为唯一宿主 |
| **invoke 矩阵** | [`INVOKE_HOTPATH_MATRIX.md`](./INVOKE_HOTPATH_MATRIX.md) | Tauri `api/*.rs` ↔ 前端 `invoke` | 命令签名变更须同步矩阵与契约测 |

**第一切片（已落地）**：错误码三方一致门禁（dimension5 **`kernel error codes drift`**）。**后续切片**：DTO `ts-rs`/`typeshare` 试点 · 六槽枚举导出 · invoke 签名 ratchet。

---

## 12.6 跨模块能力闭环与兼容边界

模块能力以**完整消费链**为单位演进，不能把 Chat Pro、共享前端、桌面宿主、内核和官方插件当作互不相关的版本孤岛。AI 改动纪律见 [`AI_CHANGE_BOUNDARIES.md`](./AI_CHANGE_BOUNDARIES.md) G17；跨产物版本规则见 [`COMPATIBILITY.md`](../creator-docs/COMPATIBILITY.md)。

| 层 | 兼容职责 | 不变式 |
|----|----------|--------|
| 角色包 / 编写器 | 产出 schema、可选能力与资源 | 旧包缺新键可加载或明确拒载；编写器新增导出须核对宿主 loader/validation |
| 内核 types/runtime/host | 定义 DTO、能力语义、编排与持久化 | 新字段优先可选/有默认；`api_version`/schema/Breaking 变更显式迁移 |
| Desktop Tauri | 命令封装、ACL、插件 Bridge、资源协议 | snake_case ↔ camelCase、invoke 注册、Bridge 分发与权限白名单同步 |
| `distros/shared` | API 镜像、store/composable、通用插槽与状态归属 | 不假设 Chat Pro 是唯一消费者；异步结果须绑定 role/scene/generation |
| Chat Pro / Theater | 壳、页面挂载、发行版策略 | 明确能力适用发行版；Chat Pro 的 Fluent/Tool 不得只更新一壳 |
| 目录插件 | manifest、Vue 入口、iframe 回退、RPC/事件声明 | `entry` 与 `vueComponent` 均可解析；Bridge/RPC 声明与宿主实现对齐；失败可降级且可诊断 |
| Chat Pro staged beat | `types::dto` → `domain/adult_stage.rs` → `db/adult_stage.rs` → HTTP/Tauri → `adultBeatQueue.ts` | stage 只生成并持久化结构化文本，不得触发正式 turn 的聊天/记忆/关系/事件/人格写入；commit 按 generation+sequence 有序且幂等，cancel 删除未提交拍；其他发行版无需实现此 Chat Pro 扩展 |

**仓内结构门禁**：`npm run check:module-compat` 对拍内核/前端 10 个嵌入插槽、官方插件 manifest、Vue/iframe 资源、RPC timeout 声明与插件索引版本。它证明结构兼容，**不替代**行为集成测或跨版本能力协商。

---

## 12.7 CI 影响元数据与脚手架边界

领域感知 CI 是**开发控制面设施**，不是运行时模块、第七槽、蓝图步骤或 Resource Coordinator。它复用模块边界来选择验证，但不改变生产编排。详细契约与阶段计划只维护于 [`SOMEDAY_TOOLCHAIN_CI.md`](../creator-docs/roadmap/SOMEDAY_TOOLCHAIN_CI.md)。

| 元数据 | 谁拥有 | 语义边界 |
|--------|--------|----------|
| 路径绑定 | 主仓中央影响图 | 只把 changed path 定位到直接模块；未知路径 fail-safe 全量 |
| `runtime_requires` | 模块描述 | 运行所需逻辑能力/服务；不是物理资源预算 |
| `resource_claims` | 模块描述，运行时 schema 另有 SSOT | 声明 GPU/RAM/CPU/渲染等需求；CI 不据此直接调度生产资源 |
| `declared_affects` | 模块维护者 | 可增加潜在下游；不能覆盖中央强制影响边 |
| `validation_profiles` | 模块描述引用，主仓验证目录定义 | 只引用受信坐标；模块不得携带命令、runner、secret 或工作流编排 |
| `extensions` | 命名空间所有者 | required 未支持时失败并全量回退；optional 保留并告警 |

最终受影响集合为直接模块经“中央强制边 ∪ 合法声明边”计算的确定性闭包；中央高风险规则可强制附加 profile 或全量。规划结果只描述“为何选中”，实际门禁强度、命令和执行环境由主仓验证目录与工作流决定。

脚手架只负责生成/校验标准结构、展示可选项并调用既有解析器预检；它不生成主仓编排权，不执行任意第三方脚本，也不维护第二套影响算法。Scaffold Package 的项目/用户/官方发现、来源锁定、命令命名空间和兼容边界见 [`RFC_SCAFFOLD_PACKAGE_V1.md`](../creator-docs/rfc/RFC_SCAFFOLD_PACKAGE_V1.md)；该契约不得反向取得 CI 控制权。

---

## 13. 一轮 co-present · 模块调用关系

```mermaid
flowchart TB
  PM["process_message"]
  AG{"⑥ agent handled?"}
  MIN["minimal_response<br/>短路 Stable pipeline"]
  CO["co_present"]
  PRE["pre"]
  MID["co_present middle"]
  RULE["规则 EventEstimate"]
  TT["TurnThinkingRouter"]
  USE{"event LLM?"}
  ER["Event Ring"]
  RC["memory.recall.candidate"]
  RA["memory.recollection.activated"]
  PST["post_llm"]

  PM --> AG
  AG -->|handled| MIN
  AG -->|continue| CO --> PRE --> MID

  PRE --> M1["① memory"] & M2["② emotion"]
  MID --> RULE --> TT
  TT --> USE
  USE -->|yes| M3["③ event.impact（legacy 子槽）"] --> ER
  USE -->|no，沿用规则估计| ER
  M1 --> RC --> ER --> RA --> M4
  ER -.-> EM["声明式事件模块"]
  ER -.-> DP["目录事件模块 event_ring.handle"]
  TT --> M4["④ prompt.build"]
  M4 --> M5["⑤ llm.generate"] --> PST
  PRE -.-> F1["设施① complex_emotion"]
  F1 -.上一轮去内容信号.-> M4
  PST -.本轮解析与持久化.-> F1
  PST -.-> SC["独立通道 reply_post_process"]
  PST -.-> F3["设施③ portrait"]
```

图中的箭头表示当前普通用户 co-present 回合的数据依赖，不表示六槽按编号机械串行。Agent 短路在 `pre` 之前；异地 stub / RemoteLife 是并列分支，见 `process_message.rs`。

---

## 13.1 Chat Pro 壳与互动模式 IA

| 主题 | SSOT / 行为 |
|------|-------------|
| **默认壳** | [`resolveOcliveShell()`](../distros/shared/src/composables/useOcliveShell.ts) fallback **`fluent`**；`VITE_OCLIVE_SHELL=tool` → ToolShell；`theater` 走剧场发行版。 |
| **用户入口** | **Settings → General**（[`SettingsGeneralTab.vue`](../distros/shared/src/components/settings/SettingsGeneralTab.vue)）+ **FluentShell** 输入区上方 [`InteractionModeBar.vue`](../distros/shared/src/components/onboarding/InteractionModeBar.vue)（经 `MAIN_SHELL_KEY.onInteractionModeChange`）。 |
| **键位绑定** | **Settings → General → Advanced** · [`keybindings.ts`](../distros/shared/src/lib/keybindings.ts)（动作目录 SSOT）· [`KeybindingsSettingsSection.vue`](../distros/shared/src/components/hotkey/KeybindingsSettingsSection.vue)；全局 OS 快捷键仍经 `save_hotkey_bindings`；`voice.holdToTalk`（默认 **V**）→ `hostEventBus` → VoiceToolbar。 |
| **发现 / 编程入口** | 日常聊解锁条 [`ImmersiveUnlockBanner`](../distros/shared/src/components/onboarding/ImmersiveUnlockBanner.vue) · 首次剧情引导 [`ImmersiveModeIntro`](../distros/shared/src/components/onboarding/ImmersiveModeIntro.vue) · 插件总线 `com.oclive.mumu.settings-panel:set_interaction_mode`（[`usePluginEvents.ts`](../distros/shared/src/composables/usePluginEvents.ts)）— **非**并列用户 IA。 |

### 13.2 Chat Pro 外观正交轴 `data-skin`

| 轴 | 属性 / 存储 | SSOT |
|----|-------------|------|
| 明暗 | `html[data-theme]` · `oclive-runtime-theme` | [`useOcliveAppearance.ts`](../distros/shared/src/composables/useOcliveAppearance.ts) |
| 壳 | `html[data-shell]` · `VITE_OCLIVE_SHELL` | [`useOcliveShell.ts`](../distros/shared/src/composables/useOcliveShell.ts) · [`chat-pro/index.html`](../distros/chat-pro/index.html) 早启动 IIFE |
| 缩放 | `--oclive-ui-scale` · `oclive-runtime-ui-scale` | `useOcliveAppearance` |
| **皮肤** | `html[data-skin]` · `oclive-runtime-skin`（`default` / `win98`） | [`useEasterEggSkin.ts`](../distros/shared/src/composables/useEasterEggSkin.ts) · [`win98/tokens.css`](../distros/shared/src/styles/win98/tokens.css) + [`win98/primitives.css`](../distros/shared/src/styles/win98/primitives.css) |
| **CSP `connect-src`** | CosyVoice2 侧车默认 `http://127.0.0.1:50000` · `ws://127.0.0.1:50000`（与插件 `local_synth_endpoint` 默认一致） | [`tauri.conf.json`](../distros/desktop-tauri/tauri.conf.json) `security.csp` |

- **范围**：chat-pro **Fluent + Tool**；theater 不纳入。
- **解锁**：Konami 序列 → `oclive-easteregg-unlocked=1` → 自动启用 Win98；设置 → 常规外观区开关（`v-if` 已解锁）。
- **正交**：皮肤只覆盖 CSS 变量与少量 chrome 类；不改 shell 布局或六槽逻辑。`appearance:changed` 事件 payload 可含 `skin`。壳 / 面板 Win98 覆写 **co-locate 于 SFC unscoped `@import`**，避免与 scoped 样式抢同一属性。
- **Authentic chrome（Win98 窗口框）**：[`Win98TitleBar.vue`](../distros/shared/src/components/win98/Win98TitleBar.vue) 挂载于 FluentShell `.app-frame` / ToolShell `.tool-body__main` 首子节点；启用皮肤时 Tauri `setDecorations(false)` 隐藏原生标题栏，合成栏经 `data-tauri-drag-region` + `allowlist.window`（`minimize` / `maximize` / `unmaximize` / `close` / `startDragging` / `setDecorations`）驱动 ─ □ ✕；关闭皮肤或退出即恢复原生装饰与边缘缩放。对话框 / 侧栏 / 气泡等 Win98 覆写见下表（✕ 仍关对话框，非 OS 窗）。

**Win98 样式依赖表**（`distros/shared/src/styles/win98/`；规则均以 `html[data-skin="win98"]` 为前缀，`default` 零泄漏）：

| CSS 文件 | 引入方 | 层级 |
|----------|--------|------|
| `win98/tokens.css` | [`chat-pro/main.ts`](../distros/chat-pro/src/main.ts) | L0 |
| `win98/primitives.css` | `chat-pro/main.ts` | L1 |
| `win98/shell-fluent.css` | [`FluentShell.vue`](../distros/chat-pro/src/shells/fluent/FluentShell.vue) | L2 |
| `win98/shell-tool.css` | [`ToolShell.vue`](../distros/chat-pro/src/shells/tool/ToolShell.vue) | L2 |
| `win98/titlebar.css` | [`Win98TitleBar.vue`](../distros/shared/src/components/win98/Win98TitleBar.vue) | L4 |
| `win98/panel-settings.css` | [`SettingsView.vue`](../distros/chat-pro/src/views/SettingsView.vue) | L3 |
| `win98/panel-market.css` | [`MarketView.vue`](../distros/chat-pro/src/views/MarketView.vue) | L3 |
| `win98/panel-model.css` | [`ModelManagerPanel.vue`](../distros/chat-pro/src/views/ModelManagerPanel.vue) | L3 |
| `win98/panel-plugins.css` | [`SimplePluginManagerPanel.vue`](../distros/chat-pro/src/views/SimplePluginManagerPanel.vue) | L3 |
| `win98/component-side-panel.css` | [`UiSidePanel.vue`](../distros/shared/src/components/ui/UiSidePanel.vue) | L3 |
| `win98/dialogs-shared.css` | [`ShortcutHelp.vue`](../distros/shared/src/components/ShortcutHelp.vue) · [`HotkeyHost.vue`](../distros/shared/src/components/hotkey/HotkeyHost.vue) · [`PluginUiSlotSelectorDialog.vue`](../distros/shared/src/components/PluginUiSlotSelectorDialog.vue) · [`ImmersiveModeIntro.vue`](../distros/shared/src/components/onboarding/ImmersiveModeIntro.vue) · [`TopBarSceneModeDialog.vue`](../distros/shared/src/components/scene/TopBarSceneModeDialog.vue) · [`PresetRolePicker.vue`](../distros/shared/src/components/onboarding/PresetRolePicker.vue) | L3 |
| `win98/component-chat.css` | [`ChatMessage.vue`](../distros/shared/src/components/chat/ChatMessage.vue) | L3 |
| `win98/component-top-bar.css` | [`TopBarMorePanel.vue`](../distros/shared/src/components/TopBarMorePanel.vue) | L3 |
| `win98/component-plugin-toolbar.css` | [`ChatPluginToolbarSlots.vue`](../distros/shared/src/components/ChatPluginToolbarSlots.vue) · [`com.oclive.voice.asr` VoiceToolbar](../distros/chat-pro/plugins/com.oclive.voice.asr/slots/VoiceToolbar.vue) | L3 |
| `win98/component-voice-settings.css` | [`PluginSettingsPanelSlots.vue`](../distros/shared/src/components/PluginSettingsPanelSlots.vue)（`com.oclive.voice.asr` VoiceSettings 插槽） | L3 |

---

## 14. 配置四层（谁可改什么）

| 层 | 典型内容 | 谁改 | AI 任务边界 |
|----|----------|------|-------------|
| 角色内容层 | `core_personality.txt` · scenes · prompts | 创作者 | **不改** slot_registry |
| 包内蓝图配置层 | `slot_registry` · `runtime_config` · 目标 `extensions` 外壳 | 管理员 / 集成方 | 须 validation；扩展载荷由对应作者维护 |
| 发行版 | `distro.oclive.toml` → HostProfile | 产品 | **不改**角色人设任务 |
| 会话配置覆盖 | `SessionCache` · slot override | 运行时 | **只在进程内存，不写包、不进 SQLite** |

`role_runtime`、关系、记忆等角色状态有各自的持久化契约，不属于上表的槽位配置覆盖层；不要因为它们共享 `session_id` 就把两者合并成“会话 DB 配置”。

---

## 15. 改动约束速查（与 AI 边界对齐）

| 任务类型 | 可动模块 | 必读 |
|----------|----------|------|
| 只改 mumu 人设 | 角色包 §4–§7 不管 | ROLE_PACK_BOUNDARY · G1 |
| 换 memory 后端 | 第 1 模块 + 蓝图 | PLUGIN_V1 · SLOT_BACKEND matrix |
| 改 Prompt 段落 | 第 4 模块 + 角色锚点 | prompt_builder · G7 `reply` |
| 改发行版延迟 | HostProfile · Turn Thinking | DISTRO_CAPABILITY_PROFILE |
| 新设施子模块 | RFC + 本文 §10 登记 | 禁止 silent 第七槽 |
| 新蓝图扩展 / GPU 能力 | RFC + 本文 §12.1 + 完整 G17 链 | 蓝图只声明；资源敏感项必须接统一协调 |
| 新 handoff 文档 | **关键决策 / RFC 仅** | [`AI_CHANGE_BOUNDARIES.md`](./AI_CHANGE_BOUNDARIES.md) G10–G12 |
| 改 Chat Pro / 插件 / 角色能力 | §12.5–§12.6 全链核对 | G17 · `npm run check:module-compat` · 行为集成测 |

---

## 16. 维护

- **只改本文**：模块定义、槽位关系、编排行能力归类、术语对照。
- **不改本文**：版本号、Wave Done 列表、CVE 日期、invoke 条数 — 各走专属 SSOT。
- **新增模块**：先更新 §2–§12，再 **一行链接** 更新 OCLIVE_ARCHITECTURE（对外），**禁止**三处粘贴同一段落。
- **动本文前**：读 [`handoff/README.md`](./README.md) §文档分责 · [`AI_CHANGE_BOUNDARIES.md`](./AI_CHANGE_BOUNDARIES.md) G13–G16。

## 17. 脉络全景（插件清单 · 正交轴 · 核心术语 · 非定义 SSOT）

**模块定义仍只维护于本文 §0–§16**。**六槽 / 独立通道 / 正交 含义** · bundled 插件全表 · 解耦形式 A–I · 调用图 → [`human-docs/team/ARCHITECTURE_DECOUPLING_PANORAMA.md`](../human-docs/team/ARCHITECTURE_DECOUPLING_PANORAMA.md) **§1** 起（2026-07-05 起）。

*2026-06-25 v2：收敛为模块注册表 SSOT；进度迁至 TECHNICAL_DEBT / PROJECT_STATUS。*
