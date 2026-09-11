# 01 · 简架构

> **最后更新**：2026-09-11
> **读者**：已跑通主仓、要理解「一条消息怎么走」的工程师。  
> **读完能做什么**：画出用户回合与主动回合主路径；说清六槽、Event Ring、上下文来源和权力边界。
> **耗时**：约 **45 分钟**（含下面扩展节）。  
> **下一篇**：[03 术语表](03_GLOSSARY.md) · 逐槽细节 → [MODULE_MAP §4–§12](../handoff/MODULE_MAP_AND_HANDOFF.md)。

配套学习图：[OCLive 架构学习长图（SVG，可无限放大）](assets/oclive-architecture-learning-map.svg) · [PNG](assets/oclive-architecture-learning-map.png)。这两份图保留为**完整参考 Host 的学习快照**，不是小 Kernel 边界图；旧称“内核”不可据此把编排、SQLite、记忆或 Event Ring 纳入小核心。当前分责以本文与 MODULE_MAP 为准，不声称静态资产已随本轮更新。

先记住两层：**最小概念核心**只保留六槽契约与必要公共合法性边界；下面的大图讲的是**当前完整参考运行时**，所以还会出现 Event Ring、SQLite、设施和宿主。它们很重要，但不等于全都属于最小 core。

这里的 Host 不只是通信桥：小 Kernel 负责六槽契约、必要公共合法性和错误边界，当前参考实现的可信 Rust Host 在约束内执行产品状态提交与资源操作；Tauri API 和 HTTP 负责传输适配。当前 `oclive_kernel_host::OcliveKernel` 是完整参考运行时门面，物理上仍装配状态、SQLite、插件与设施，不能据此声称小 core 已拆成独立 crate。权责唯一 SSOT 是 [MODULE_MAP_AND_HANDOFF.md](../handoff/MODULE_MAP_AND_HANDOFF.md)。

以 Chat Pro 为例：Tauri API/HTTP 是传输面，runtime 负责装配 `AppState`、状态与权限；`distros/desktop-tauri/src/api/chat_backend.rs` 明确 loopback kernel 是 single authoritative writer。这个例子不表示每个 Distro 都必须一对一配置一个 Host。

先读 [Kernel/Host 权责与候选边界摘要](../handoff/MODULE_MAP_AND_HANDOFF.md#kernel-responsibilities)，再用 [定点源码对照](../handoff/MODULE_MAP_AND_HANDOFF.md#kernel-source-map) 区分职责目标与当前实现；本页的流程只是学习当前 ChatPro Host。

---

## 一轮对话（当前参考 Host 主路径）

```mermaid
flowchart TB
  UI[Vue 前端\ninvoke 或 HTTP]
  API[desktop-tauri/api/*.rs\n或 http_api]
  PM[process_message.rs\n主编排]
  AG[agent\n可选短路]
  TP[turn_pipeline\npre → middle → LLM → post]
  ER[Event Ring\nevent.impact · 记忆提案]
  PH[PluginHost\n六槽]
  UI --> API --> PM
  PM --> AG
  AG -->|继续普通分支| TP
  TP <--> ER
  PH --> TP
```

**实现文件**：[`process_message.rs`](../kernel/crates/oclive_kernel_host/src/domain/chat_engine/process_message.rs)（经 `chat_engine/mod.rs` re-export）。

**分支（先知道有这三条）**

| 分支 | 何时 |
|------|------|
| **Agent 短路** | 当前 Host 在 `handled=true` 时转去构造最小响应，跳过普通主链；不据此断言整个公共 invocation 已结束 |
| **异地 / remote_life** | 用户与角色不在同场景 |
| **共景 co_present** | 默认 Chat Pro 主路径（本文以下默认此路径） |

**当前参考 Host 的运行口诀**：预检 → Agent 分支 → 情绪与记忆 → 事件与思考 → Prompt → LLM → 持久化与展示。顺序由当前 **Rust Host 代码**保证；蓝图 **`steps[]` 不参与首轮调度**。这不是所有 Host 的固定流水线。

---

## turn_pipeline（共景内）

| 阶段 | 文件 | 做什么 |
|------|------|--------|
| **pre** | `turn_pipeline/pre.rs` | 人格/关系/身份、用户情绪、STM/LTM 检索 |
| **middle** | `turn_pipeline/co_present/run_middle.rs` | Turn Thinking、Fast 情绪强度降级、知识、event 估计、Event Ring、关系预览、Prompt；Prompt 只接收上一轮情绪余韵的去内容信号 |
| **LLM** | `turn_pipeline/post.rs` + `llm` 槽 | 生成原始 `reply` 与可选 `[EMO]` |
| **post** | `turn_pipeline/post/post_llm.rs` · `persistence.rs` | 剥离 `[EMO]`、解析本轮角色回复情绪与复杂情感、保存下一轮 hint，再做策略与状态落地、立绘、回复后处理、聊天写入 |

入口：[`turn_pipeline/mod.rs`](../kernel/crates/oclive_kernel_host/src/domain/chat_engine/turn_pipeline/mod.rs) 的 `execute_turn`。

### 上下文从哪里来

| 来源 | 例子 | 谁处理 |
|------|------|--------|
| 当前输入 | 用户消息或外部观察 | `TurnInput` 保持来源边界 |
| 角色包静态内容 | 核心人格、世界观、场景、Prompt 扩展 | loader + prompt 槽 |
| 运行时状态 | 当前人格、关系、好感、情绪、位置、时间 | host 编排/Repository |
| 检索证据 | STM、LTM、近期对话、知识块 | memory/knowledge |
| 本轮派生结果 | 用户情绪、事件影响、复杂情感、激活回想 | emotion/event/设施/Event Ring |

第 4 模块 `prompt` 负责把**本轮获准进入模型的事实、候选和已提交状态**组装成模型输入；第 5 模块 `llm` 消费它。获准进入不等于语义必然正确：场景/身份/工具结果可以是事实，情绪/意图/回忆意义通常仍应是候选。回复后处理面对模型输出，不负责倒推或重做上下文决策。

复杂情感是一个容易画错的跨轮例子：本轮 Prompt 只能知道“上一轮存在情绪余韵”，看不到 hint 原文；本轮主模型回复后才解析 `[EMO]`，有效标记优先，remote/directory 只作兜底，解析出的 hint 留给下一轮。这样既保留模型判断权，也保留可追踪、可复用的状态契约。

### 谁能决定什么

| 组件 | 一句话权力 |
|------|------------|
| 当前 Rust Host 编排 | 决定阶段、分支、结果何时应用和持久化 |
| Event Ring（当前参考运行时设施） | 签发事件身份、来源、注册权重、顺序和因果链 |
| Event 决策模块 | 决定自己负责的一类提案是否采纳 |
| 六槽/设施 | 提供检索、分析、估计、组装或生成能力 |
| 后处理 | 修改最终展示文本，不重做事件与记忆决策 |

当前参考运行时的口诀：**六槽提供能力，Ring 管事件可信流通，决策模块管采纳，Rust Host 编排管领域应用。** 领域条件和提交责任属于 Host；公共调用约束不等于领域提交成功保证，传输和 UI 不取得领域权威。

### Turn Thinking（Fast / Deep · 编排行）

**不是第七槽**；由 `turn_thinking.rs` + `co_present` 内 `TurnThinkingRouter` 决定本回合档位。

| 层 | 谁配 | 作用 |
|----|------|------|
| **Wave E · 持久化** | 发行版 `distro.oclive.toml` → `[turn_thinking] fast_persistence` | `strong_only` 时 Fast 闲聊不写 LTM / 好感 / 性格演化；**Quarrel 等强事件仍写** |
| **Wave F · 路由** | 角色包 `config.json` → `turn_thinking` | OR/AND 规则、Deep latch（争吵→和解）、`ephemeral_archive` 局面摘要（TTL） |

**纪律**：聊天 turns **每轮仍写** UI 日志；Fast **不压缩**用户原句；**无**玩家 Fast/Deep 开关。人类开工包 → [modules/orchestration/turn-thinking.md](modules/orchestration/turn-thinking.md) · 深读：[RFC_TURN_THINKING_PERSISTENCE](../creator-docs/rfc/RFC_TURN_THINKING_PERSISTENCE.md) · [MODULE_MAP §12](../handoff/MODULE_MAP_AND_HANDOFF.md)。

---

## 记忆：三套存储（最易混 · 必背）

很多人说的「记忆有三层」，在本项目里指 **三套 SQLite/存储职责**，**不是** Prompt 的「系统/角色/用户」三层。

| 存储 | 表 / 组件 | 干什么 | 会进 Prompt 吗 |
|------|-----------|--------|----------------|
| **① 聊天日志** | `chat_sessions` / `chat_messages`（HybridConversationStore） | UI 聊天记录、导出、**记忆回放**数据源 | **否** |
| **② 短期记忆 STM** | `short_term_memory` | 近几轮缓冲 | **是**（memory 槽检索） |
| **③ 长期记忆 LTM** | `long_term_memory` | AI 归档、mention 衰减 | **是**（memory 槽检索） |

**三条纪律**

1. **删聊天记录 ≠ 清空 AI 记忆表**  
2. **MemoryEngine 不读** `{app_data}/chats/` 当真源  
3. 「记忆回放」是从 ① **合并写入** ③，不覆盖已有 LTM 全文  

深读：[CHAT_STORAGE_ARCHITECTURE](../handoff/CHAT_STORAGE_ARCHITECTURE.md) · 模块注册表：[MODULE_MAP §4](../handoff/MODULE_MAP_AND_HANDOFF.md#4-第-1-模块--memory)

---

## 六槽（第 1–6 后端模块）

| # | 键 | 职责（人话） |
|---|-----|--------------|
| 1 | `memory` | 检索 ②③ 注入 Prompt |
| 2 | `emotion` | 分析用户句情绪 |
| 3 | `event` | legacy 对话事件类型/影响估计；结果进入 Event Ring |
| 4 | `prompt` | 组装完整 prompt 字符串 |
| 5 | `llm` | 调用模型生成 **reply** |
| 6 | `agent` | 工具 / MCP；可短路主链 |

- **解析链**：蓝图 `slot_registry` → `PluginHost::resolve_for_role`  
- **多实例**：同 type 折叠为运行时 `PluginBackends`（memory 去重合并 · llm last-wins）  
- **backend 真值表**（24 格）：[`SLOT_BACKEND_REALITY_MATRIX`](../handoff/SLOT_BACKEND_REALITY_MATRIX.md)  

**逐槽定义、trait、禁止项** → [MODULE_MAP §4–§9](../handoff/MODULE_MAP_AND_HANDOFF.md)。

六槽是稳定能力分类，不表示当前每轮一定调用全部六个实现。共景健康门槛目前是 `prompt + llm`；memory、emotion、event、agent 可以按已定义的 `none` / Noop 语义关闭或变薄。普通扩展也不会自动成为“第七槽”；真的改变六槽分类属于 Breaking 核心契约修订。

---

## Event Ring：外环，不是第七槽

Event Ring 是当前完整参考运行时的一项可复用设施，不是最小核心成立的前提。它不按固定顺序强制所有模块运行。模块先注册自己订阅/允许发射的事件；可信宿主另行分配基础影响权重和失败策略。模块只能提交草案，Ring 负责签发真实来源、权重、顺序和因果链。

当前两条最重要的链：

```text
event 槽估计 → chat.event_impact.estimated → Ring → 人格/关系预览

memory 找到候选 → recall.candidate → event decision
  → recollection.activated（可能没有）→ 本轮回想 Prompt
```

权重只表示提案被判断时的基础影响力，不代表执行优先级，也不保证被采纳。完整契约见 [EVENT_RING](../creator-docs/plugin-and-architecture/EVENT_RING.md)。

> **不要把当前 Ring 画成永久事件河流**：这里的“外环”是单次 dispatch 路由边界，不是持续轮询模块的运行顺序。现在虽有一个默认关闭、只把成功 dispatch 的脱敏事实头写入独立 SQLite 的 Trace 影子，但它不能被读取来驱动行为。仓库已经为未来 Stream 写下纯类型、本地持久化设计，并按 OneBot v11 规范明确了 QQ 文本 ACK 与“不确定投递”边界；这些仍只是设计/协议夹具，没有持久 Stream、读取循环、真实 QQ 适配器或消费者。跨回合/跨通道/跨重启能力仍由 [`K-EVENT-STREAM-01`](../creator-docs/rfc/RFC_RUNTIME_EVENT_STREAM.md) 跟踪；即使以后实现，也不能替代 Event 决策、Rust 状态提交权或 Stable 回合管线。

### 没有用户消息时如何主动开口

```text
可信传感器/系统适配器提出事件
  → proactive_turn_decision
  → 权威授权事件
  → 一次性 Permit
  → process_proactive_turn
  → 以 ExternalObservation 进入共景回复管线
```

外部观察不会冒充用户消息；当前也不写用户聊天、STM/LTM、关系或人格。它仍缺去重、冷却、频率限制、用户输入抢占和通用输出端口，因此是可用地基，不是无限自动运行的 Bot。

---

## 不是六槽，但常一起问

| 类别 | 例子 | 占 `plugin_backends` 吗 |
|------|------|-------------------------|
| **第 N 设施子模块** | 复杂情感 hint、专家路由、立绘、视觉舞台 | **否**（编排行内） |
| **独立通道** | 用户身份、回复后处理、剧场 Scene Director API | **否** |
| **编排行策略** | Turn Thinking、发行版 `HostProfile` | **否** |

一张总表：[MODULE_MAP §2](../handoff/MODULE_MAP_AND_HANDOFF.md#2-模块四大类划分)。

---

## 配置谁说了算（四层）

```text
角色包内容（人设、场景）
  → 蓝图 slot_registry（六槽、引擎策略）
    → 发行版 distro.oclive.toml（HostProfile）
      → SessionCache 会话内存覆盖（临时、不写包、不进 SQLite）
```

这四层首先表示维护责任面，并非四份同名字段无条件互相覆盖；角色内容与包内蓝图在同一角色包中并列分责。发生重叠时才按有效配置解析链逐层合成，后层也不能突破宿主能力与安全上限。初级创作者只改角色内容层；蓝图由高级作者或宿主管理员维护。**不要**在「改 mumu 文案」任务里动 `slot_registry`。见 [ROLE_PACK_BOUNDARY](../handoff/ROLE_PACK_BOUNDARY.md)。

---

## Crate 五层（主依赖方向）

```mermaid
flowchart TB
  types[oclive_kernel_types\nDTO]
  contracts[oclive_kernel_contracts\ntrait]
  runtime[oclive_kernel_runtime\nPromptBuilder]
  host[oclive_kernel_host\nprocess_message]
  tauri[desktop-tauri\nIPC 薄壳]
  tauri -->|依赖| host -->|依赖| runtime -->|依赖| contracts -->|依赖| types
```

图只画当前参考运行时主链，省略 host / tauri 对 types、validation 等直接依赖；它不是未来小 Core 的物理拆分图或新调度契约。口诀：**Types 形状 · Contracts 接口 · Runtime 公式 · Host 流程 · Tauri 入口。**

---

## 验收

- [ ] 能指出 `process_message.rs` 与 `co_present` 的关系  
- [ ] 能区分 **聊天日志 / STM / LTM** 三者  
- [ ] 能列出六槽名称，并说出「复杂情感 **不是** 第七槽」  
- [ ] 能区分 legacy `event` 槽、Event Ring、Event 决策模块和 Rust 编排
- [ ] 能说出用户消息与 `ExternalObservation` 为什么必须分开
- [ ] 知道主编排 **不读** 蓝图 `steps[]` 当 DSL  

---

## 深度链接

- [MODULE_MAP_AND_HANDOFF](../handoff/MODULE_MAP_AND_HANDOFF.md) — **模块注册表**  
- [RFC_TURN_THINKING_PERSISTENCE](../creator-docs/rfc/RFC_TURN_THINKING_PERSISTENCE.md) — Fast/Deep · 持久化 · 包级路由  
- [OCLIVE_ARCHITECTURE_OVERVIEW](../creator-docs/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md) — 对外叙述  
- [BUS_FACTOR_NOTES §1](../handoff/BUS_FACTOR_NOTES.md#1-内核编排process_message)
