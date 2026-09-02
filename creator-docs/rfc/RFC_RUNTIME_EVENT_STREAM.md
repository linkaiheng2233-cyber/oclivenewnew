# RFC：Runtime Event Stream（角色运行事件流）

**SSOT 范围**：本文只定义未来 Runtime Event Stream 的分层、权力边界、事件分型、投递/恢复语义与分阶段准入条件；现有 Event Ring wire、注册策略和主动 Permit 仍以 [`EVENT_RING.md`](../plugin-and-architecture/EVENT_RING.md) 为准，实施进度只在 [`K-EVENT-STREAM-01`](../../handoff/TECHNICAL_DEBT_INVENTORY.md) 维护。
**最后更新**：2026-09-02。
**状态**：**草案 v0.11 · Stage A.1、A.2.1、A.2.2.1 与 A.2.2.2-R0 受控实机探针已落地 · 尚无真实 OneBot/QQ 执行证据 · B0 Trace-only 的 S0/S1 合成证据阶段已收口 · Production Stream 未实现**。
**读者**：内核维护者、输入/输出适配器作者、Event/记忆/Agent 模块作者与多通道集成方。

---

## 0. 决策摘要

“角色运行河流”正式使用工作名 **Runtime Event Stream**。它是未来位于 Event Ring 之外、跨回合与跨重启保存角色运行事件及消费进度的持久时间线，**不是当前 Event Ring 改名，也不是让模块按圆圈顺序轮询运行的调度器**。

本 RFC 固定以下不可越过的边界：

| 决策 | 约束 |
|------|------|
| 唯一回合编排 | 用户回合继续进入现有 `process_message`；非用户回合继续先经 Event 决策与一次性 Permit，再进入 `process_proactive_turn`。不得复制 pipeline 或建立第二套内核 |
| Event Ring 保留 | Ring 继续负责来源绑定、信封身份、注册基础影响权重和单次 dispatch 的有界路由；Stream 不接管这些权力 |
| 状态单写者 | Session、Stream、消费者、插件和 LLM 都不能直接提交人格、关系、记忆或其它权威状态；Rust 编排仍是应用与提交边界 |
| 决策不等于提交 | `Decision` 表示提案已被领域决策接受/拒绝；只有提交成功后才能产生 `State` 事实 |
| 注册表不重复 | 基础影响权重继续只由现有 `EventModuleRegistryPolicy` 分配；Stream 的消费者登记只管理订阅、游标、读取权限和背压，不产生第二套影响权重 |
| 模型档位不扩权 | 大模型可以观察更多、查询更多、提出更丰富的提案；小模型使用模块筛选和 Prompt 编译后的有限上下文。任何模型都没有事实伪造或状态提交权 |
| 当前能力声明 | 可选 B0 Trace-only 影子只旁路记录成功 Ring dispatch 的脱敏事实头；Stage A.1 只有可编译 DTO 与状态机测试模型，A.2.1 只有对现有代码路径做校验的版本化设计夹具，A.2.2.1 只有钉住 OneBot v11 文档版本的协议响应/治理夹具；A.2.2.2-R0 只有默认拒绝、须人工显式确认的独立发送/撤回探针，本轮未连接真实实现或账号。S0/S1 命令只生成合成、忽略提交的 Trace 证据。持久 Stream、outbox/inbox 表、生产 QQ 适配器、消费者游标存储、读取循环、重放、多 IO 调度或产品化主动 Bot 仍未交付 |

---

## 1. 问题、目标与非目标

### 1.1 当前缺口

当前 Event Ring 是每个 `AppState` 内的有界内存外环：一次 dispatch 结束后只保留有限诊断历史，不负责跨回合、跨 QQ/直播/游戏/传感器通道或跨重启延续事件，也没有通用消费者游标、确认、重试、幂等、背压、保留期与恢复契约。

### 1.2 目标

- 为一个角色运行 Session 建立可恢复的事件时间线与明确的来源/因果关系。
- 让记忆、环境感知、Agent、主动回合调度和输出适配器按声明观察或派生事件。
- 保留现有 Event Ring、六槽与 Stable 回合管线，不为新能力复制权威边界。
- 让消费者崩溃、重复投递、宿主重启和部分通道故障具有可测试的恢复语义。
- 让 Trace/Replay 可独立启停，并在只读模式下不改变角色行为。

### 1.3 非目标

- 不把 token、音频 PCM、视频帧、Live2D 参数等高吞吐数据塞进通用事件流。
- 不让所有模块每轮固定执行，也不把 Stream 变成新的蓝图 `steps[]` 调度器。
- 不新增第七槽，不改变 legacy `event` 只负责 `event.impact` 的事实。
- 不允许 LLM、Session 或目录插件构造“已提交状态”。
- 不在本 RFC 冻结 Rust trait、数据库表、外部 broker 或公开网络协议。
- 不以“有持久日志”为理由自动启用主动回复或扩大当前非用户回合的持久化副作用。

---

## 2. 五层边界与辅助参与者

| 层 | 唯一职责 | 可以做 | 不能做 |
|----|----------|--------|--------|
| **Session** | 一个角色运行实例的身份、隔离与生命周期边界 | 绑定角色、通道、访问主体和 Stream 分区 | 采纳提案、修改人格、直接提交状态或把用户输入改成系统来源 |
| **Runtime Event Stream** | 持久追加、分区顺序、消费者进度、保留与恢复 | 保存已获准事件记录；按读取策略投递；记录 checkpoint | 判断角色该说什么、给提案分配影响权重、执行回合或写角色状态 |
| **Event Ring** | 可信信封、来源绑定、注册策略与单次 dispatch 的有界路由 | 将注册模块的 `EventDraft` 签成 `EventEnvelope`；运行获准处理器 | 充当持久队列、最终采纳所有领域提案或提交状态 |
| **Event 决策模块** | 在自己的事件族内采纳、拒绝或派生提案 | 依据证据、注册权重和当前只读状态形成 `Decision` | 伪造来源/权重、直接写数据库、签发未经宿主校验的能力 |
| **Rust 编排** | 调度、权限复核、状态应用与唯一提交边界 | 调用现有回合入口；CAS/事务提交；提交成功后发布 `State`/`Output` 事实 | 将业务判断全部塞进 Session/Stream，或接受 LLM 直接写状态 |

辅助参与者的权力：

- **输入适配器**只把外部输入规范化为提案/观察，并通过来源绑定入口提交；不能自报可信来源或基础影响权重。
- **消费者模块**可以观察、查询、提出或派生新事件；不能把读取到的历史信封重新冒充为新事实。
- **Prompt 层**只组装已经通过模块筛选和决策的上下文，不直接遍历整条 Stream。
- **LLM**是推理工具；输出默认属于候选内容或 `Proposal`，不能直接成为 `Fact`、`Decision` 或 `State`。
- **输出适配器**负责真实投递及结果回报；“生成了回复”不等于“已发送到 QQ/直播/设备”。

术语必须保持：本文的 **Event 决策模块**不是第 3 个 legacy `event` 槽。后者只估计对话 `event.impact`；领域提案的采纳权属于对应决策模块，最终状态提交权属于 Rust 编排。

---

## 3. 与现有主链和 Event Ring 的关系

### 3.1 当前主链保持不变

```text
用户输入
  → process_message（唯一用户回合入口）
  → turn_pipeline
  → memory / legacy event.impact 等产生证据或提案
  → Event Ring 单次 dispatch
  → Event 决策结果
  → Prompt → LLM → post
  → Rust 编排提交状态
```

主动链继续保持：

```text
可信非用户来源
  → source-bound EventEmitter
  → kernel.proactive.turn.proposed
  → Event 决策
  → kernel.proactive.turn.authorized
  → one-shot ProactiveTurnPermit
  → process_proactive_turn
```

Runtime Event Stream 的加入不得改变上述权威顺序。

### 3.2 未来持久流通关系

```text
外部输入 / 内核结果
        ↓
来源绑定入口 → Event Ring dispatch → 已签发 EventEnvelope
                                      ↓ append
                           Runtime Event Stream
                                      ↓ read
                           声明式消费者 / 调度器
                                      ↓ 新 EventDraft（带因果引用）
                              Event Ring 再次裁决
                                      ↓
                  现有 process_message / process_proactive_turn
                                      ↓
                      Rust 提交成功 → host-bound emitter → Ring
                                      └→ State / Output 事实 → Stream
```

关键规则：

1. 外部生产者不能绕过来源绑定入口直接向 Stream 写任意 `EventEnvelope`。
2. Stream 保存的是已经过入口校验的记录；持久化本身不增加采纳权或状态权。
3. 消费者读取旧事件后如需产生影响，必须提交一个**新的** `EventDraft`，并以旧事件作为 causation；不得重用旧 `event_id` 冒充新发生。
4. Event handler 内不得递归启动回合。只有 dispatch 返回后，Rust 调度边界才能消费 Permit 或其它类型化结果。
5. 用户消息的低延迟路径仍可直接进入 `process_message`；Trace-only 记录不得成为用户主链的必经依赖。未来若启用 Stream-first 多通道 ingress，也只能把合格用户输入路由到同一个 `process_message`。

### 3.3 Trace 与生产 Stream 分离

- **Trace-only**：旁路记录输入、决策、提交和输出结果；可独立关闭，关闭后角色行为必须相同。
- **Production Stream**：消费者确实依赖其游标和恢复语义；不可用时必须显式降级或阻塞该消费者，不能静默改成直接写状态。
- 两者可以共享事件身份和底层存储实现，但启停、读取权限、保留期和故障语义必须独立。

#### 当前 B0 Trace-only 第一切片（已实现）

`d2320596` 只落地了可独立回滚的影子观察器，不是 Production Stream：

- `OcliveKernelConfig` 默认不配置 Trace，也不创建 Trace 文件；显式开启时使用独立 SQLite，拒绝与内核主库同址；
- Ring 仅在一次 dispatch 全部成功并写入有界内存历史后，把最终信封头交给内部观察口；失败 dispatch 不记录；
- 热路径只执行有界队列 `try_send`，SQLite 打开、schema、队列或写入失败只累计脱敏诊断/丢弃计数，不改变 `EventDispatchResult`；
- 记录有存储自增 `position`，并保存 `event_id`、kind、source、注册权重、correlation、causation、Ring `sequence`/depth 与发生/接收时间；`position` 不冒充 Session 消费游标，Ring `sequence` 也不冒充持久游标；
- 表禁止原地 update/delete，不保存 payload、metadata 或 `stream_key`；当前没有读取 API、消费者、checkpoint、Replay、保留期执行器，也不观察 Rust 状态提交或输出投递；
- `runtime_event_trace_diagnostics()` 只暴露配置/工作状态、队列与写入计数、最后位置和错误种类，不暴露路径或事件正文。

所以 B0 只证明“成功 Ring 事实可被行为中性地旁路持久化”。它不证明 Session Runtime Event Stream、语义事件分型、至少一次消费或恢复闭环已经完成。

#### B0 合成样本采集（已实现，仍只作为证据）

`6bb3e1f7` 增加了显式开发者命令 `npm run event:trace-shadow-samples`，用于在不接触真实用户数据的前提下积累可复现结构样本：

- 版本化场景合同固定为 5 个场景、7 条记录，覆盖单根事件、带 causation 的派生链、主动提案获准/拒绝，以及同一 Trace SQLite 上的内核重启；
- 采集过程只注册合成 Event Ring 模块，注入的 payload、metadata、observation、角色/Session 标识均为合成值；不会调用模型回复，也不会把 Permit 送入主动回合；
- 证据 JSON 只保留场景内索引、kind、source、注册权重、持久位置、Ring sequence、因果索引和 depth；采集器会递归拒绝 payload、metadata、`stream_key`、事件 ID、时间戳、observation 与用户消息字段；
- SQLite、JSON 与 Markdown 只生成在被 Git 忽略的 `target/oclive-event/trace-shadow-samples/`，并记录源提交、工作树状态和场景合同 SHA-256；原始 SQLite 默认不得提交；
- 当前样本证明存储位置在两次内核启动间连续为 1–7，而进程内 Ring sequence 在第二次启动重置为 1。这个观测只验证 B0 结构，不是 Session 游标、消费恢复或 Production Stream 证据。

后续样本必须按以下阶梯准入，不能因为“需要更多数据”就直接收集角色对话：

| 等级 | 范围 | 准入边界 | 当前状态 |
|------|------|----------|----------|
| **S0 · 固定合成结构样本** | 仓库内版本化合同；成功/派生/决策/重启结构 | 显式本地命令；只写忽略目录；不驱动行为、不进入训练 | **已实现** |
| **S1 · 扩展合成故障矩阵** | 队列满、写入失败、重复事件、并发和持续运行等可复现场景 | 仍不使用真实用户、角色记忆或模型正文；每个新场景先冻结预期与隐私字段 | **已收口：S1.1 故障 + S1.2 并发/短时 soak + S1.3 recorder 重复头 + S1.4 固定十分钟耐久** |
| **S2 · 明示同意的本地运行聚合** | 计数、延迟、丢弃率、事件种类分布等最小聚合 | 必须先完成开关、告知/同意、保留期、删除、脱敏、预算和导出审查；默认不采正文，不自动上传 | 未获准实现 |

`d03655b5` 实现 S1.1 命令 `npm run event:trace-shadow-fault-samples`，但没有修改生产 Trace、Ring 或公开 API：

- `queue_saturation_fail_open` 在临时 Trace SQLite 上持有写锁，再提交合同固定的 640 次合成 Ring dispatch；验收只冻结“640 次 Ring 全成功、入队与丢弃计数守恒、至少出现一次 `queue_full`、已入队项最终排空、主库健康”，不把线程调度决定的入队/丢弃精确分割写成产品合同；
- `post_start_write_failure_fail_open` 只破坏临时 Trace 表，再提交 1 次合成 dispatch；要求 Ring 成功、诊断出现 `write_failed`、无 dispatch 因 Trace 被拒、主库仍健康；
- JSON/Markdown 只写 Git 忽略的 `target/oclive-event/trace-shadow-fault-samples/`，不导出故障 SQLite；校验器拒绝正文、身份、事件 ID、correlation、运行路径和事件时间字段；
- 重复事件、并发与持续运行在 S1.1 时仍属于后续切片。重复事件若需要新增内部重放入口，必须先评审其是否会成为绕过来源绑定的测试后门。

`e17038a4` 实现 S1.2 命令 `npm run event:trace-shadow-load-samples`，同样没有修改生产 Trace、Ring 或公开 API：

- `concurrent_burst_fail_open` 由 8 个并发 worker 各提交 64 次来源绑定的合成 dispatch；合同只要求 512 次 Ring 全成功、Trace 入队/丢弃守恒、已入队项最终排空、Ring 历史保持有界，并只允许无错误或 `queue_full`；
- `bounded_short_soak_fail_open` 以 4 个 worker、24 轮、每 worker 每轮 8 次 dispatch 和轮间 50 ms 停顿形成 768 次有界短时运行；完整场景受 60 秒上限保护，验证 worker/轮次完整、主库健康和 Trace 排空；
- 重复本地样本中，并发突发的精确入队/丢弃分割会随调度变化，而短时 soak 可完整排空；耗时与精确比例只保存在忽略提交的本机证据中，不是性能 SLA 或产品合同；
- JSON/Markdown 只写 `target/oclive-event/trace-shadow-load-samples/`；共用校验器拒绝正文、身份、事件/关联 ID、路径和事件时间字段，并拒绝目录中出现 SQLite 或其它额外文件；
- 本切片没有添加 Replay、重复注入、消费者、checkpoint 或行为反馈入口。消费者重复投递仍属于阶段 C；生产时长与目标硬件 soak 不属于 S1.2 或后续 S1.4 的能力声明。

`20f44f47` 实现 S1.3 命令 `npm run event:trace-shadow-duplicate-samples`；它只编译并运行 B0 recorder 的内部测试，不增加生产 Trace、Ring 或公开 API：

- 版本化合同只包含 `duplicate_header_idempotency`：测试在 `#[cfg(test)]` 内把同一个合成 Trace 头交给私有 recorder 两次，不建立生产 Replay 或重复注入入口；
- 验收固定为 2 次内部记录批次均入队并处理、SQLite 只保留 1 行、`duplicate_events = 1`、零丢弃、零 Trace 故障，且 worker 在显式 shutdown 后停止；这不是两次真实 Ring dispatch 的生产链路测试；
- Rust 测试只打印规范化计数与布尔标记；采集器再递归拒绝正文、metadata、`stream_key`、事件/关联 ID、路径和事件时间字段，并只把 JSON/Markdown 写入 `target/oclive-event/trace-shadow-duplicate-samples/`，不导出 SQLite；
- 这只证明 B0 recorder 对同一事件身份的幂等落库与诊断计数。它**不证明**消费者重复投递、至少一次消费、端到端幂等、checkpoint 恢复或 Production Stream；这些仍属于阶段 C 及后续工作。

`993fb041` 在不增加读取或行为入口的前提下加固了 B0 生命周期，作为 S1 收口前置条件：

- 诊断 schema v2 以 serde 默认值兼容读取 v1，并区分队列拒绝与“已入队但终端写失败”；
- 终端写失败会停止接收、排空队列并把未持久批次计入失败，不让 worker 伪装成仍可用；
- 并发或重复 shutdown 只执行一次有序排空；同一事件身份但事实头冲突时拒绝覆盖原记录并进入终端失败；
- 这些变化只影响 Trace 健康度与关闭语义，Ring dispatch 仍保持 fail-open，且没有 consumer、checkpoint 或 Replay 入口。

`b79c7912` 实现 S1.4 命令 `npm run event:trace-shadow-soak-samples`，用于固定而非产品化的持续运行证据：

- 版本化合同固定十分钟、每秒一轮、2 个 worker 每轮各 2 次来源绑定 dispatch，共 600 轮、1,200 个 worker-run 与 2,400 次 Ring dispatch；完整场景有 660 秒硬上限；
- 每 30 轮检查一次主库健康，共 20 次；同时采样进程 RSS 起点/峰值/终点，以 128 MiB 峰值增长上限作为本机回归门槛，并验证 Ring 诊断历史保持有界；
- 显式 shutdown 后从临时 SQLite 核对 2,400 行与最终位置，再用同一 Trace 路径重启并追加 1 条，验证启动位置连续、写入位置为 2,401 且主库仍健康；
- JSON/Markdown 只写 `target/oclive-event/trace-shadow-soak-samples/`；不导出 SQLite，校验器拒绝正文、身份、事件/关联 ID、路径和事件时间字段；该命令明确不进入常规 CI；
- 十分钟与 RSS 结果只是固定开发者回归证据，**不是**生产时长、目标硬件、吞吐或 SLA 声明，也不证明 consumer、至少一次、checkpoint、Replay、保留期或恢复闭环。

#### S1 退出与下一阶段入口

S1 只在以下条件同时满足时视为“Trace-only 合成证据收口”：S0、S1.1、S1.2、S1.3 与 S1.4 的版本化合同通过；B0 生命周期、故障计数和 shutdown 测试通过；全部生成物留在忽略目录；生产代码仍没有读取/消费/Replay/Prompt/主动回复入口。这个“收口”只关闭合成样本缺口，不关闭阶段 B 的保留期、提交/输出覆盖，也不关闭 `K-EVENT-STREAM-01`。

下一项获准工作仍在阶段 A：A.1 已冻结纯类型和参考状态机；A.2.1 已冻结本地存储拓扑、transactional outbox/inbox 与隐私删除设计；A.2.2.1 已用钉住提交的 OneBot v11 规范冻结 QQ 文本输出的 ACK、不确定投递、撤回与多宿主否定证据；A.2.2.2-R0 已准备默认拒绝的实机探针。A.2.2.2 仍须由维护者接真实 OneBot 实现与专用测试账号，分步验证实际发送/撤回、可控超时、适配器私有 store 和单写者租约；它完成前不得直接实现生产读取 API、checkpoint 表、消费者循环或第二套回合入口，阶段 C 也不得与阶段 A 偷跑合并。

任一级样本都不是权威事件库、行为输入或训练授权。S2 之前不得增加真实运行采集接线；S2 之后若要使用内容级数据，必须另立隐私与数据治理设计，不能沿用本命令扩权。

---

## 4. 事件分型

每种事件必须声明一种语义类型。类型描述的是**权力与生命周期**，事件的 dotted `kind` 仍描述具体领域。

| 类型 | 含义 | 允许生产者 | 对角色的影响 | 禁止 |
|------|------|------------|--------------|------|
| **Fact** | 某个可信来源对已发生/已知事项的签名陈述 | Rust 宿主、获准适配器、提交后的领域组件 | 可作为决策证据 | 把“来源可信”写成“内容绝对正确”；由普通 LLM直接生成 |
| **Observation** | 原始或规范化输入，例如传感器读数、直播弹幕、游戏状态变化 | 来源绑定输入适配器 | 可触发查询或提案 | 自动触发回复、直接修改人格/记忆 |
| **Proposal** | 模块建议采纳的上下文、动作或状态变化 | 注册模块、受约束 Agent/LLM 包装器 | 进入 Event 决策 | 自称已采纳、携带伪造权重或最终状态 |
| **Decision** | 决策模块对特定 Proposal 的接受、拒绝或派生结果 | 对应事件族的可信 Event 决策模块 | 允许 Rust 编排进入下一步 | 冒充数据库已提交；越过预期 revision |
| **State** | Rust 编排成功提交后的状态事实或可验证引用 | 宿主提交边界 / transactional outbox | 供消费者更新只读视图、索引或后续推理 | 用作“请修改状态”的命令；在提交前发布 |
| **Output** | 回复/动作已经生成、尝试投递或投递完成的结果事实 | Rust 输出边界、获准输出适配器 | 驱动 UI、审计、重试或通道状态 | 把生成成功等同投递成功；让模型自报外部发送结果 |

补充约束：

- 语义类型不能由普通 payload/metadata 自报；实施时必须由宿主维护的 event-kind registry 或类型化外层字段分配，并与模块允许发射范围一起校验。
- `Fact` 是**来源可追责的陈述**，不是全知真理；冲突 Fact 必须保留各自来源并交由决策策略处理。
- `Decision` 与 `State` 必须分开。决策可以因 revision 过期、权限变化或事务失败而未被应用。
- `Output` 至少区分 prepared / delivery attempted / delivered / failed 的语义，具体 wire 留待实施 RFC 冻结。
- Stream checkpoint、租约、背压等控制面记录不默认进入角色语义上下文，也不冒充上述六类领域事件。

---

## 5. 身份、顺序与持久记录

### 5.1 复用 `EventEnvelope`，不篡改当前字段语义

现有 `EventEnvelope` 继续承载 `event_id`、`source`、`source_weight_bps`、`stream_key`、`correlation_id`、`causation_id`、`sequence`、`depth`、`occurred_at`、payload 与 metadata。未来持久层应使用一个外层记录保存 Session 分区和持久位置，而不是把所有 Stream 语义硬塞进现有信封。

Stage A.1 已由提交 `3ee559a6` 在 [`runtime_event_stream.rs`](../../kernel/crates/oclive_kernel_types/src/runtime_event_stream.rs) 冻结以下 **v1 纯数据契约**。这些类型可编译、可序列化，但没有存储实现、读取 API、consumer loop、Replay 或回合接线：

```text
RuntimeEventRecord
├─ schema_version
├─ session_partition / session_binding_revision
├─ stream_position                  # 存储分配、分区内单调
├─ semantic_type                    # Fact / Observation / Proposal / Decision / State / Output
├─ payload_schema / privacy_class / retention_policy_id
├─ source_idempotency_key?          # 宿主批准、来源作用域、非秘密的 opaque key
├─ ingested_at                      # 宿主接收时间
└─ envelope: EventEnvelope          # 现有可信事件信封

RuntimeEventConsumerCheckpoint
├─ consumer_id
├─ session_partition
├─ next_position
├─ revision / lease_epoch
└─ state: ready | leased | retry_scheduled | blocked | disabled
```

`RuntimeEventRecord` 的外层治理字段由宿主 event-kind registry 分配，普通 payload、metadata、插件或 LLM 不能自报。`source_idempotency_key` 的唯一性作用域固定为 `(session_partition, source, key)`；同一作用域/键对应不同事件内容时必须拒绝，而不是覆盖旧记录。

### 5.1.1 追加快照规则

Production Stream 未来只允许追加以下两类不可变快照：

1. 完整 Ring dispatch 成功后，由宿主保留并追加的 **Ring 初始签发信封**；
2. dispatch 内显式发射、拥有自身来源与 `causation_id` 的子事件初始信封。

Ring handler 对原事件 payload/metadata 的原地替换仍只是 dispatch-local 兼容行为，不能被提升为来源无歧义的持久事实。若变换结果需要跨回合保留，处理器必须发射新事件。失败 dispatch 不进入 Production Stream；实施时需要让 Ring 返回或旁路保留初始信封，但本切片没有修改现有 `EventDispatchResult`。

必须区分：

| 字段/概念 | 权威语义 |
|-----------|----------|
| `event_id` | 事件身份与幂等关联；同 ID 不得对应不同内容 |
| `correlation_id` | 一轮任务/交互的关联，不代表存储顺序 |
| `causation_id` | 直接原因；派生事件必须形成可追踪链 |
| `EventEnvelope.sequence` | 当前 `AppState` 的 Ring 分配顺序，重启后不能当持久游标 |
| `stream_position` | 未来 Stream 在一个 Session 分区内的持久位置 |
| `session_binding_revision` | 追加时的 Session 映射 CAS 快照；防止端点/角色版本换绑后误归档 |
| `semantic_type` | 宿主按 event kind 分配的权力/生命周期类型，不来自 payload |
| `payload_schema` / `privacy_class` / `retention_policy_id` | 追加时冻结的治理策略引用；不表示保留期执行器已实现 |
| `occurred_at` | 来源事件时间，可能受设备时钟漂移影响 |
| `ingested_at` | 宿主接收时间；恢复与游标不能只依赖 `occurred_at` |
| `stream_key` | 当前 Ring 的路由/关联键；不能自动等同 Session 身份或访问控制凭证 |

### 5.2 Session 隔离

- Session 标识由宿主建立并绑定角色、授权主体和通道；外部 payload 不能自行切换到其它 Session。
- v1 使用 `RuntimeSessionPartitionBinding` 显式保存 `partition_id`、`runtime_session_id`、`role_id`、宿主计算的 `role_pack_revision`、`access_subject_id`、`legacy_srid`、`binding_revision` 与多个 `endpoint_id + adapter_id`。
- `partition_id` 与 `runtime_session_id` 都由宿主签发且 v1 一对一，但不要求字符串相等；现有请求 `session_id` 只作为可信 Session registry 的兼容查询输入，不能直接拼接或复制成分区 ID。
- 当前 `srid = conversation_state_role_id(mrid, session_id)` 是既有持久化命名空间；`legacy_srid` 只负责显式兼容映射，不升级为 Stream 身份或访问凭证。
- `access_subject_id` 表示授权/数据所有者，不等于角色扮演中的 `user_identity_id`；后者可随场景变化，不能因此切分或合并 Stream。
- `role_pack_revision` 只提供事件来源版本证据。角色包升级以 `binding_revision` CAS 更新并产生新事实，不自动创建或合并 Session。
- 一个 Session 可以绑定多个通道端点；端点离线不等于 Session 销毁。
- 跨 Session 查询、记忆共享或角色合并必须经过独立授权和显式策略，不能靠相同 `role_id` 自动开放。

### 5.3 Stage A.2.1 真实代码路径评审与首个存储选择

当前仓库能证明的是“多种集成表面共用一个内核”，不是已经存在多通道 Event Stream：

| 现有表面 | 真实路径 | A.2.1 结论 |
|----------|----------|-------------|
| Desktop Tauri | `kernel_attach/chat.rs` 转发 `/chat` / `/chat/stream` | 只是 HTTP 代理，不建立回合管线 |
| HTTP JSON / SSE | `http_api/chat.rs` 调用 `process_message` / `process_message_stream` | 两种传输共用现有 chat engine |
| Rust Library | `OcliveKernel` 委托同一 `AppState` 与 chat engine | 宿主门面，不是第二内核 |
| 主动输入 | 来源绑定 emitter → Ring → Permit → `process_proactive_turn` | 不能伪装用户消息，也不能绕过 Event 决策 |
| 目录 Event 插件 | `event_ring.handle` 观察获准事件并发射自身命名空间子事件 | 只有提案/派生权，没有状态提交权 |

仓库内仍没有 QQ、直播或硬件输出适配器提供可持久验证的“发送成功”回执；`SendMessageResponse` 和 SSE `done` 只证明内核生成完成，不等于外部送达。A.2.2.1 的 OneBot 规范评审只冻结未来适配器的判定边界，不是一次真实发送。

首个 Production Stream 存储冻结为本地嵌入式 **SQLite**，默认文件名 `runtime-event-stream.sqlite3`，且必须与主状态 `app.db`、Trace-only 的 `runtime-event-trace.sqlite3` 三址互异：

- `app.db` 继续保存权威角色状态与聊天 SQLite 数据，并在未来由同一事务写入 producer outbox；现有 chat transcript 虽共用同一个 `DbManager`，但当前仍在角色状态事务之后单独追加，不能被文档冒充一个原子提交。
- `runtime-event-stream.sqlite3` 未来只拥有 Session binding、不可变事件记录、consumer registry/checkpoint、正常保留期 tombstone 与删除工作项；本切片没有创建文件、迁移或连接池。
- Trace-only 数据库永不升级或复用为 Production Stream，仍默认关闭、不可读回行为。
- JSON chat mirror 继续是非权威副本；用户删除完成语义不能沿用“SQLite 已删、mirror 失败只告警”的 best-effort 口径。
- 两个 SQLite 文件之间不宣称原子事务，也不依赖 `ATTACH DATABASE` 制造跨库 exactly-once。主库 outbox 到 Stream 采用至少一次传输，Stream 以来源作用域幂等键拒绝冲突重复。

只有出现以下任一有证据的条件，才进入外部 broker 选型：同一 Session 需要在多个宿主同时活跃；远程消费者要求独立于本机内核的可用性；实测 WAL、容量或延迟预算无法满足目标。提前抽象可替换 store port 可以，但不得在没有证据时把 broker 变成运行前提。

### 5.4 持久化前必须补齐变换来源

当前 Ring 允许已注册处理器在一次 dispatch 内替换当前 payload 或合并 metadata，但主事件的 `source` 仍属于最初 emitter。这适合有界兼容链，却不足以证明一条持久 Fact 的每次语义变换来自谁。

因此生产 Stream 必须遵守：

- 改变事实/提案语义的模块应发射带自身来源和 `causation_id` 的子事件，不把语义变化永久写在原来源名下。
- 原地 payload/metadata 变换只可作为 dispatch-local 兼容行为；若 Trace 记录最终快照，必须同时保留处理器路由摘要，且不能把该快照提升为来源无歧义的持久 Fact。
- 实施前须明确“追加 dispatch 前快照、dispatch 后快照还是派生事件”的规则；同一事件不能在 Stream 中被覆盖更新。
- Stream 记录一经追加即不可原地改写；更正和撤销使用新事件及因果引用。

---

## 6. 注册、订阅与权重

不得把“事件流消费者登记”发展成第二套 Event 权威注册表。

| 注册面 | 管理内容 | 不管理内容 |
|--------|----------|------------|
| `EventModuleRegistryPolicy`（现有） | 模块可信身份、基础影响权重、`fail_fast` / `isolate` | Stream cursor、保留期、消费并发 |
| Stream Consumer Registry（未来） | `consumer_id`、允许读取的事件族、Session 范围、checkpoint、最大并发/在途量、失败退避与隐私级别 | 提案基础影响权重、采纳权、状态提交权 |

Stage A.1 的 `RuntimeEventStreamConsumerRegistration` 固定保存：`consumer_id`、kind 订阅、允许语义类型、宿主 `session_scope_id`、隐私读取上限、跨 Session 分区最大并发、租约时长、最大尝试次数与退避上下限。v1 **同一消费者在单个 Session 分区内固定串行**，只允许跨分区并发，因此不会用并发完成顺序冒充连续 checkpoint。登记结构没有影响权重、提案采纳或状态提交字段。

消费者派生 `Proposal` 时，必须通过自己的来源绑定 Event emitter 回到 Ring；其基础影响力由现有可信注册策略分配。消费者不能因为“读到了更多历史”而提高自己的权重，也不能在 manifest 中自报最终权威。

---

## 7. 投递、幂等、顺序、冲突与背压

### 7.1 默认投递语义

- 首个实现以**至少一次（at-least-once）**为默认，不宣称端到端 exactly-once。
- 消费者只在副作用成功或安全持久化后推进 checkpoint；崩溃恢复可能再次收到同一事件。
- 每个消费者必须以 `event_id`、领域 idempotency key 或“原因事件 + 动作类型”实现幂等。
- 生产者重试需要来源绑定 idempotency key；同一 key 对应不同内容必须拒绝并记录诊断。
- 若派生事件和 checkpoint 必须原子一致，实施时应使用 transactional inbox/outbox 或等价机制，不能靠“先写一个再写另一个”冒充原子性。

### 7.1.1 v1 consumer/checkpoint 状态机

`next_position` 初始为 1。任何阶段都不能仅凭“处理函数返回了”推进它；必须先确认副作用已持久化，或通过幂等键确认该副作用此前已经完成。

| 当前状态 | 允许输入 | 下一状态 | checkpoint / lease 规则 |
|----------|----------|----------|-------------------------|
| `ready` | claim 精确的 `next_position` | `leased` | `next_position` 不变；`revision + 1`、`lease_epoch + 1` |
| `leased` | `applied` / `already_applied`，且 revision/epoch/position 全匹配 | `ready` | `next_position = delivered_position + 1`；`revision + 1` |
| `leased` | `retryable_failure` | `retry_scheduled` | 位置不变；`revision + 1`，按登记策略退避 |
| `leased` | lease 到期或宿主重启恢复到期 lease | `retry_scheduled` | 位置不变；旧 worker 的完成回报因 epoch 过期而拒绝 |
| `retry_scheduled` | 到达重试时间 | `leased` | 位置不变；`revision + 1`、`lease_epoch + 1` |
| `leased` | `terminal_failure` 或超过尝试上限 | `blocked` | 位置不变；不得自动跳过 |
| `blocked` | 宿主/运维显式 retry | `retry_scheduled` | 仍处理同一位置；不得伪造成功 |
| `ready` | 检测到保留期缺口 | `blocked` | 不自动越过缺失记录；须由删除/墓碑策略显式解决 |
| 非 `disabled` | consumer 登记撤销/禁用 | `disabled` | 保留 `next_position`，停止签发新 lease |

所有 checkpoint 改动使用 `revision` CAS。delivery 回报同时携带领取时的 `checkpoint_revision` 与 `lease_epoch`；任一不匹配都原样拒绝，不得“尽量合并”。`applied` 与 `already_applied` 都只对当前 lease 推进一次，后者用于至少一次投递中的幂等重复确认，不是 exactly-once 声明。

### 7.1.2 Stage A.2.1 事务交接矩阵

| 情况 | 必须处于同一事务 | 跨域交接与失败规则 |
|------|------------------|--------------------|
| Rust 权威状态提交 | `app.db` 中的领域状态 + producer outbox | 提交前失败两者一起回滚；提交后 Stream 不可用只重试 outbox，不能回滚已提交状态或发布假失败 State |
| 成功 Ring dispatch 的持久快照 | 宿主批准的初始信封 + `app.db` producer outbox | 完整 dispatch 后才写；需要持久保证的授权/接受结果必须等 outbox commit 后才能逸出，dispatch-local 替换不得进入 |
| 消费者副作用 | 消费者权威 store 中的 inbox 去重记录 + 副作用 + 可选派生事件 outbox | checkpoint 只在 `applied` / `already_applied` 证据后推进；重复投递复用 inbox 结果，不重复副作用 |
| 外部 Output 送达 | 输出适配器权威 store 中的 output outbox + 送达回执 | 只有 adapter ACK 后才能发 `delivered` Output；模型生成完成、HTTP 返回或 SSE `done` 不能冒充送达 |

跨 `app.db`、Stream SQLite 和消费者/适配器 store 的复制一律按至少一次处理。任一实现若没有“本域 inbox/outbox 原子写 + 跨域幂等接收”，不得宣称可靠发布、可靠消费或可靠输出。

`source_idempotency_key` v1 只接受 1–256 字节、无控制字符、明确为非凭据的 opaque UTF-8；不 trim、不大小写折叠。唯一性作用域为 `(session_partition, source, source_idempotency_key)`，同键同内容返回既有结果，同键不同内容必须拒绝并诊断。

### 7.1.3 Stage A.2.2.1 · OneBot v11 QQ 文本输出评审

首个外部输出协议选择 OneBot v11 的 HTTP JSON 同步接口，只作为通用 Output 契约的真实协议样本，不把内核绑定为 QQ 专用架构。评审来源固定到 OneBot v11 仓库提交 `d4456ee706f9ada9c2dfde56a2bcfc69752600e4` 的 [API 总则](https://github.com/botuniverse/onebot-11/blob/d4456ee706f9ada9c2dfde56a2bcfc69752600e4/api/README.md)、[公开消息 API](https://github.com/botuniverse/onebot-11/blob/d4456ee706f9ada9c2dfde56a2bcfc69752600e4/api/public.md)、[HTTP 响应](https://github.com/botuniverse/onebot-11/blob/d4456ee706f9ada9c2dfde56a2bcfc69752600e4/communication/http.md)、[Bearer 鉴权](https://github.com/botuniverse/onebot-11/blob/d4456ee706f9ada9c2dfde56a2bcfc69752600e4/communication/authorization.md) 与 [WebSocket `echo`](https://github.com/botuniverse/onebot-11/blob/d4456ee706f9ada9c2dfde56a2bcfc69752600e4/communication/ws.md)，避免浮动分支改变证据。

| 边界 | A.2.2.1 冻结结果 |
|------|------------------|
| 发送面 | 首切片只允许 `POST application/json` 的同步 `send_private_msg` / `send_group_msg`；纯文本默认 `auto_escape=true`。`_async` 与 `_rate_limited` 只返回“已提交异步处理”，不能用于 delivered 判定 |
| 唯一 ACK | 同时满足 HTTP `200`、`status="ok"`、`retcode=0` 且 `data.message_id` 存在，才能产生 `delivered` Output 并推进 checkpoint |
| 不确定投递 | 请求正文可能已被接受后的超时/断线、`status="async"`、畸形响应或缺少 `message_id` 的“成功”响应一律进入 `delivery_uncertain`；不推进 checkpoint，阻塞自动重试和换宿主接管，等待人工或适配器对账 |
| 明确拒绝 | HTTP `400/401/403/404/406` 或合法 `failed` 响应进入拒绝阻塞；默认不按实现方未标准化的文本错误消息猜测重试 |
| 幂等与多宿主 | OneBot v11 标准没有定义发送幂等键；WebSocket `echo` 只关联请求/响应，不提供去重。每个 output outbox item 必须只有一个带 lease epoch/fencing token 的发送所有者；标准也没有定义服务端 fencing，本地 SQLite 不是多宿主协调器 |
| 回执隐私 | Stream 只接收 `adapter_id`、opaque `receipt_ref`、outcome、attempt 与确认时间；access token、QQ/群号、正文和 provider `message_id` 留在加密的适配器私有存储，不进入通用事件回执 |
| 撤回与删除 | `delete_msg(message_id)` 是平台消息撤回，不是平台隐私擦除证明。Session 删除应先阻断新发送，尽力撤回仍有私有 locator 的消息，再硬删本地适配器正文/目标/locator 并验证；外部撤回失败不阻止本地擦除完成，但完成报告只能给出不可关联的聚合状态，不能声称平台已删除 |
| 网络安全 | 默认只连 loopback；远程端点须显式 `network:*` 授权与 TLS。access token 只走 `Authorization: Bearer` 请求头；凭据不得进入 Stream/Trace/日志，正文不得用 GET query 发送 |

版本化夹具 [`runtime_event_stream_stage_a2_onebot_review.v1.json`](../../kernel/crates/oclive_kernel_types/tests/fixtures/runtime_event_stream_stage_a2_onebot_review.v1.json) 固定 9 个响应/传输场景。这里新增的 `delivery_uncertain` 是 A.2.2 设计状态，不是已经发布的公共 Rust DTO；A.2.2.2 必须在真实适配器上冻结其持久状态与人工对账入口。

### 7.1.4 Stage A.2.2.2-R0 · 默认拒绝的实机探针

提交 `79d8aad7` 增加独立开发者命令 `npm run event:onebot-live-probe`。它不是生产适配器，只为维护者在专用 OneBot 实现和测试账号准备好后收集第一份可审计的同步发送/撤回证据：

- 未提供精确确认短语 `A2.2.2_TEST_ACCOUNT` 时，在任何网络请求前失败；私聊与群聊测试目标必须且只能选一个。
- access token 只从 `OCLIVE_ONEBOT_ACCESS_TOKEN` 环境变量读取并放入 Bearer 请求头；命令参数、控制台与证据均不输出 token。
- 默认只接受字面 `127.0.0.1` / `::1`；远程端点必须同时使用 HTTPS 和 `--allow-remote`。禁止 URL 内凭据、query、fragment 与重定向。
- 先用 `get_version_info` 确认 `protocol_version="v11"`，再发送固定无用户内容的探针文本并设置 `auto_escape=true`；只有完整同步 ACK 才调用一次 `delete_msg`。
- 发送/撤回均没有自动重试；提交后超时、畸形响应或缺失 `message_id` 继续按 `delivery_uncertain` 停止。
- 默认将 JSON 证据写入 Git 忽略的 `target/oclive-event/onebot-live-probe/`；证据不含端点、目标 ID、正文、token 或 provider `message_id`，且继续声明 Production runtime/read/consumer/store/lease 均未实现。

执行前由维护者在本机进程环境中设置 `OCLIVE_ONEBOT_BASE_URL`、`OCLIVE_ONEBOT_ACCESS_TOKEN`、`OCLIVE_ONEBOT_IMPLEMENTATION_LABEL`，以及 `OCLIVE_ONEBOT_TEST_USER_ID` 或 `OCLIVE_ONEBOT_TEST_GROUP_ID` 之一，再显式运行：

```powershell
npm run event:onebot-live-probe -- --confirm-live A2.2.2_TEST_ACCOUNT
```

真实运行应使用专用测试账号和已同意接收测试消息的目标。R0 成功也只关闭“真实同步发送 + 撤回”这一子证据；真实超时、适配器私有 store、跨进程/多宿主 owner lease 和人工对账仍须分开验证。

### 7.2 顺序与冲突

- 只保证单个 Session 分区内的 `stream_position` 顺序，不提供全局总序。
- 多来源并发不能依靠墙钟时间做 last-write-wins；`occurred_at` 只作证据。
- 状态变更由 Rust 编排携带 expected revision/CAS 或事务条件；过期 `Decision` 必须拒绝、重算或显式标记 superseded。
- 相互冲突的 Fact/Observation 保留来源与因果，不由 Stream 自行选择“真相”。

### 7.3 背压与丢弃

| 情况 | 最低要求 |
|------|----------|
| 消费者变慢 | 有界 in-flight、指数退避和可观察 lag；不能无限占用内存 |
| 高频 Observation | 只有来源策略明确允许时才能采样/合并，并记录丢弃计数与窗口 |
| 过期 Proposal | 可按 TTL 丢弃，但必须产生可诊断的 expired 结果，不能继续换取 Permit |
| Decision / State | 不得静默丢弃；存储失败须走 outbox/reconcile 或显式 degraded 状态 |
| 输出投递失败 | 记录 failed Output 结果并交给输出策略重试；不得重新生成假用户回合 |

---

## 8. 记忆、上下文与模型档位

Runtime Event Stream 可以让记忆模块观察更多时间线，但不会把“整条河流”直接塞进本轮 Prompt。

```text
Stream Observation / State
  → 记忆消费者更新索引或提出 recall/archive Proposal
  → Event 决策
  → Rust 编排按 ID 获取获准正文/摘要
  → Prompt 编译为本轮有限上下文
  → LLM
```

边界：

- 通用事件只携带最小 ID、分数、摘要引用和因果信息；长期记忆正文继续由记忆存储按权限读取。
- 记忆消费者可以提出“回想某条记忆”或“建议归档”，不能直接宣布已经回想、已经写入 LTM 或已经改变人格。
- 当前 `memory.recall.candidate → recollection.activated → one-turn weave` 链继续有效；未来 Stream 只扩展可观察来源与恢复，不绕开该采纳链。
- 大模型可以获得更宽的检索窗口和多步查询工具；小模型继续使用编译后的短上下文。档位只改变**能力预算**，不改变权力边界。
- Prompt 层不得把未经筛选的跨用户、跨 Session 或隐私敏感事件直接注入模型。

---

## 9. 主动调度与输出端口

Stream 中出现 Observation 或高权重 Proposal **不等于角色必须开口**。产品化主动能力仍依赖 `K-PROACTIVE-01` 的状态机：

1. 去重与 TTL；
2. Session/角色冷却和频率预算；
3. 用户输入抢占与在途回合互斥；
4. Event 决策与一次性 Permit；
5. dispatch 返回后的 `process_proactive_turn`；
6. 明确输出端口和 delivered/failed 回报；
7. 取消、超时、重试与恢复。

最低抢占原则：新用户输入优先于尚未执行的主动提案；已经消耗 Permit 的回合如何取消必须由回合状态机定义，不能由事件 handler 递归启动/终止另一个回合。

Runtime Event Stream 本身不扩大当前主动回合的持久化范围。聊天、STM/LTM、关系、人格等写入只有在独立产品决策、状态契约和测试完成后才能解冻。

---

## 10. Trace、Replay、隐私与保留期

### 10.1 Trace/Replay

- Trace 默认是行为中性的旁路消费者；关闭 Trace 后，同一输入的角色决策和回复路径应保持一致。
- 当前 B0 只记录成功 Ring dispatch 的脱敏信封头；它不提供 Replay，也不把 Trace 表作为事件权威库或行为输入。
- Replay 默认运行在隔离 Session / dry-run 中，输出端口关闭，状态提交替换为只读比较；不得重发 QQ 消息、直播动作或硬件指令。
- 生产恢复通过消费者 checkpoint 和幂等处理完成，不把“从头重放所有 Output”当恢复策略。
- 重放事件保留原事件引用，但新的派生结果使用新身份，并明确 `replay_of`/causation 关系；具体字段以后冻结。

### 10.2 隐私与保留

- 每类事件在进入生产 Stream 前必须由宿主 event-kind registry 声明语义类型、允许来源、payload schema/字节上限、隐私分级、可读取消费者、保留策略与删除行为；payload 不能自报这些字段。
- 凭据、完整系统 Prompt、原始长期记忆正文、未经同意的音视频和目录插件秘密不得进入通用 payload。
- 诊断默认只显示事件 kind、来源、因果、位置、状态与计数；payload/metadata 值继续最小披露。
- 普通过期不制造物理缺口：原事件内容替换为同一 `stream_position` 的最小 position tombstone，只保留 schema、分区、位置、retention policy、过期时间与原因码；不得保留 event/source/correlation/payload。消费者可以按宿主 tombstone 前进，未授权的物理缺口仍进入 `blocked`。
- `Decision` / `State` 等 required 类别在必需消费者处理完成前不得因普通保留期清除；具体最长 lag 预算仍由 event-kind policy 冻结。
- 用户请求删除整个 Session 时不保留分区 tombstone：先阻断新 ingress/lease，再依次删除主库状态与待发 outbox、Stream binding/事件/checkpoint、必需消费者派生数据、启用的 mirror/cache，最后验证所有必需域不存在。任一必需域失败，状态保持 `blocked`，不能返回“已完成”。
- 外部平台的“撤回”不等于隐私擦除。以 OneBot v11 为例，`delete_msg` 只有消息撤回语义；本地删除完成必须覆盖输出适配器私有 store，外部撤回只能以不可关联的聚合结果报告，不能升级为平台数据已删除的承诺。
- 删除流程状态固定为 `requested → quiescing → deleting → verifying → completed`，任一步可进入 `blocked` 并幂等重试。最小完成回执不得保存原始 Session/partition/访问主体/角色/event ID 或 payload。
- 现有角色删除会在主 SQLite 提交后对 JSON mirror 做 best-effort 删除并只记录警告；这对当前功能是已知兼容行为，但不满足未来 Session 隐私删除合同，本切片没有修改该路径。
- “正常运行 append-only”不覆盖法定/用户删除：Production Stream 必须把授权删除作为显式控制面例外，并留下不含可关联原始标识的最小完成证据。
- 观察/训练用途必须与运行用途分权；“可用于角色运行”不自动等于“可用于模型训练”。

---

## 11. 故障与降级语义

| 故障 | 必须行为 |
|------|----------|
| Trace-only 存储不可用 | 记录诊断并停用 Trace；不得改变当前用户回合结果 |
| Production Stream 不可用 | Stream 依赖消费者进入明确 degraded/blocked；用户 Stable 主链按未来宿主故障策略决定是否继续，不能静默绕过权威边界 |
| 消费者崩溃 | 不推进 checkpoint；隔离重试，不阻塞无关消费者 |
| 重复投递 | 幂等返回既有结果或安全跳过；不能重复提交状态/发送输出 |
| 过期 Decision | Rust revision 检查拒绝并记录 superseded/conflict |
| 状态提交成功但事件发布失败 | 由 transactional outbox/reconcile 补发；不得发布一个假的失败状态覆盖真实提交 |
| Event 模块越权/失败 | 继续遵守现有 `fail_fast` / `isolate`，Stream 不改变 Ring 原子拒绝语义 |

### 11.1 Stage A.1 失败/恢复测试模型

版本化夹具 [`runtime_event_stream_consumer_transitions.v1.json`](../../kernel/crates/oclive_kernel_types/tests/fixtures/runtime_event_stream_consumer_transitions.v1.json) 与纯测试参考模型目前固定 14 个场景：精确位置 claim、禁止越位、durable success、幂等重复成功、可重试失败、到期重试、重启恢复过期 lease、旧 worker 回报、checkpoint CAS 冲突、终止失败阻塞、显式解阻、保留期缺口、登记禁用与禁用后拒绝 claim。

这份模型只冻结状态转移与“不推进”的条件，不实现数据库、定时器、lease worker 或消费者副作用。阶段 C 实现必须用真实持久化 crash/restart 测试重新证明同一矩阵，不能把 Stage A 的内存参考模型当作生产恢复证据。

### 11.2 Stage A.2.1 代码证据与治理合同

提交 `c1673ec0` 的版本化夹具 [`runtime_event_stream_stage_a2_review.v1.json`](../../kernel/crates/oclive_kernel_types/tests/fixtures/runtime_event_stream_stage_a2_review.v1.json) 固定五类现有集成表面、三库分离、四类事务交接、来源幂等、普通保留 tombstone 与 Session 删除编排。测试会读取夹具列出的真实 Rust 路径并检查关键委托符号仍存在，防止后续把一个适配器漂移成第二条回合管线。

该夹具同时强制声明 `production_runtime_enabled = false`，并保留五个缺口：没有真实 QQ/直播/硬件 Output ACK、没有多宿主 Session lease 证据、没有 Production schema/migration、没有保留/删除执行器、没有 consumer/read API。它是设计评审证据，不是 SQLite、跨库恢复、删除完成或外部投递的运行证据。

### 11.3 Stage A.2.2.1 OneBot 协议证据

提交 `01f2ae63`、后续阻塞语义修正 `78a1e11a` 与协议/实现证据措辞修正 `66b8a9a8` 增加 OneBot v11 协议夹具和纯测试分类器：9 个场景覆盖同步 ACK、缺失 `message_id`、异步响应、明确拒绝、鉴权失败、畸形响应、请求前失败与提交后超时。测试同时锁住回执最小化、适配器私有加密字段、撤回非擦除、多宿主单所有者/fencing 要求，以及“不确定投递必须阻塞 checkpoint”的规则。

夹具显式声明 `live_adapter_tested=false`、`production_runtime_enabled=false` 与 `production_ready=false`。它没有启动 OneBot、登录 QQ、发送/撤回消息或实现适配器 store/owner lease，因此只关闭 A.2.2.1 协议映射缺口；A.2.2.2 真实适配器证据仍为 OPEN。

### 11.4 Stage A.2.2.2-R0 探针准备证据

提交 `79d8aad7` 增加受控实机探针及 6 条本地合成测试，覆盖默认拒绝、远程 HTTP 拒绝、目标互斥、路径型标签/运行 ID 拒绝、同步发送后撤回、缺失 `message_id` 不重试/不撤回、撤回不确定，以及落盘证据脱敏与本地互斥锁释放。测试只连接临时 loopback stub，`live_adapter_tested=false`；本轮未执行真实 OneBot/QQ 请求，因此 A.2.2.2 仍为 OPEN。

---

## 12. 分阶段实施准入

在写生产代码前，必须先用真实多 IO 消费方与参考项目核对 consumer、恢复和冲突语义；外部项目只作为证据输入，不能替代本 RFC 的权力边界。

| 阶段 | 范围 | 完成证据 | 明确不做 |
|------|------|----------|----------|
| **A · 契约原型（A.2.2.2-R0 已备探针）** | 冻结事件分型、Session 映射、外层记录、checkpoint、本地存储/事务/删除与首个真实协议映射 | A.1 可编译 DTO + 14 场景状态表；A.2.1 五表面/三库/四事务/删除夹具；A.2.2.1 OneBot v11 的 9 场景合同；A.2.2.2-R0 受控实机探针及合成自测；真实发送/撤回、超时、私有 store 与 owner lease 仍待验证 | 不接生产 IO，不改回复 |
| **B · Trace-only（B0 部分落地）** | 可关闭的持久记录器；B0 只观察成功 Ring dispatch 头，提交/输出摘要仍待后续合同 | B0 已有 disabled parity、重启续位、脱敏、append-only、坏库/同址 fail-open 与确定性 shutdown；S0/S1 合成证据已收口，含固定十分钟耐久和重启续位；保留期与提交/输出覆盖未完成 | 不驱动决策或主动回复 |
| **C · Consumer 基础** | 游标、至少一次、幂等、背压、隔离失败；先接无副作用测试消费者 | crash/restart、重复投递、lag、删除测试 | 不允许消费者直接写状态 |
| **D · 首个领域闭环** | 选择一个真实低风险消费者，经 Draft → Ring → Decision → Rust 应用闭环 | 正常、拒绝、重复、过期 revision、降级测试 | 不一次接入所有记忆/Agent/IO |
| **E · 主动与多通道** | 一个真实非用户输入适配器 + 调度状态机 + 输出端口 | TTL/冷却/抢占/取消/投递恢复与人工体验验收 | 不以单通道 demo 宣称通用 Bot 已完成 |

任一阶段都不能仅凭“表已建”“事件能写入”将 `K-EVENT-STREAM-01` 标记 Done。

---

## 13. 实施前仍需冻结的决策

- A.2.1 已选独立 `runtime-event-stream.sqlite3` 作为首个 Production 后端，并固定与 `app.db` / Trace 三址分离；具体 store port、schema、migration、WAL/容量预算和 broker 迁移实现仍未冻结。
- A.1 已冻结 Session 映射字段及 v1 一 Session/一分区关系；宿主如何签发/轮换 opaque ID、端点 ACL 与多宿主 lease 仍未冻结。删除传播的状态与必需域已冻结，但还没有执行器。
- B0 当前记录所有成功 dispatch 的最终脱敏信封头；哪些提交/输出摘要及哪些事件进入 Production Stream、payload 最小化和访问规则仍未冻结。
- A.2.1 已固定 1–256 字节 opaque key 规范、主库 producer outbox、消费者 inbox/outbox 与 adapter ACK 事务边界；具体表字段、加密/密钥轮换、reconcile 调度和故障预算仍未冻结。
- 多宿主同时运行同一 Session 时的租约、leader 或冲突策略；A.2.2.1 已确认 OneBot v11 标准未定义发送幂等或 fencing，不能依赖标准协议解决。
- 首个低风险真实消费者；首个 QQ 输出已完成 OneBot v11 协议映射和 A.2.2.2-R0 探针准备，但真实实现/账号运行、超时、适配器私有 store 与 owner lease 证据仍未完成。
- Replay 的隔离数据库、模型调用策略和隐私删除传播。

这些问题不阻塞本文作为边界草案，但在相应阶段编码前必须转成可测试的 accepted contract。

---

## 14. 验收清单

### 14.1 B0 Trace-only 已有证据

- [x] 默认关闭且不创建数据库；开启后使用独立 SQLite，并拒绝主库同址。
- [x] 只有成功 dispatch 进入观察口；失败 dispatch 与 Trace 故障都不改变 Ring 返回结果。
- [x] 热路径为有界非阻塞入队；重启后从存储自增位置继续追加。
- [x] schema 不含 payload、metadata、`stream_key`，并拒绝 update/delete。
- [x] S0 合成采集合同可复现单根、派生因果、主动采纳/拒绝与重启边界；证据只写 Git 忽略的 `target/`，并拒绝敏感字段。
- [x] S1.1 合成故障合同证明队列饱和与启动后写失败均不改变 Ring dispatch；只导出脱敏计数和布尔不变量，不导出故障数据库。
- [x] S1.2 合成负载合同证明 8 路并发突发与 24 轮短时 soak 不改变 Ring dispatch；验证计数守恒、排空、历史有界、主库健康和执行时限，不导出负载数据库。
- [x] S1.3 测试编译专用合同证明同一合成 Trace 头处理两次时只落 1 行并计 1 次重复；无生产 Replay/注入入口，不把 recorder 幂等冒充消费者至少一次投递或端到端幂等。
- [x] 生命周期测试区分队列丢弃与入队后写失败，覆盖冲突重复头、终端失败停机和并发幂等 shutdown，并保持 v1 诊断反序列化兼容。
- [x] S1.4 固定十分钟合同完成 2,400 次低速来源绑定 dispatch、20 次主库健康检查、RSS/有界历史检查、shutdown 排空与重启后第 2,401 位追加；只形成本机开发者证据，不冒充生产时长或硬件 soak。
- [x] S1 合成证据阶段已收口；S2 未获准。Stage A.1 已补齐纯类型，A.2.1 已补齐本地代码路径/存储/事务/删除设计合同，A.2.2.1 已补齐 OneBot v11 协议级输出评审，A.2.2.2-R0 已备受控探针；真实适配器证据仍未完成。
- [x] 当前没有读取/消费/Replay/Prompt/主动回复接线，`K-EVENT-STREAM-01` 保持 OPEN。

### 14.2 Production Stream 总体验收（未完成）

- [x] Stage A.1 纯类型契约没有第二套 `process_message`、复制 pipeline、Session 内核或任何 I/O 端口。
- [x] `RuntimeEventStreamConsumerRegistration` 没有影响权重、提案采纳或状态提交字段。
- [x] `RuntimeEventRecord.stream_position` 与 `EventEnvelope.sequence` 是独立字段，类型/测试未将后者用作消费游标。
- [x] Session v1 显式映射宿主分区、canonical Session、角色、角色包 revision、访问主体、legacy `srid` 和多个端点；角色扮演身份不参与分区。
- [x] 14 场景参考模型固定至少一次消费中的成功、重复、重试、过期 lease、CAS 冲突、阻塞、禁用和保留期缺口语义。
- [x] A.2.1 夹具对照现有五个集成表面，冻结独立 Stream SQLite、主库 producer outbox、消费者 inbox、adapter ACK、普通保留 tombstone 与 Session 删除顺序，且显式保持 runtime disabled。
- [x] A.2.2.1 夹具按钉住提交的 OneBot v11 规范冻结 9 个 ACK/重试/不确定投递场景、最小回执、撤回非擦除与多宿主否定证据，且显式保持 live adapter/runtime/production readiness 为 false。
- [x] A.2.2.2-R0 探针默认拒绝执行，须测试目标、Bearer token 与精确确认；只用固定文本做一次同步发送和一次撤回，不自动重试，证据脱敏且只写 `target/`。6 条合成自测未连接真实 OneBot/QQ。
- [ ] A.2.2.2 尚未在真实 OneBot 实现与 QQ 测试账号上验证发送、超时、撤回、适配器私有 store 和 owner lease；上述协议夹具不是 Production Stream、真实外部送达、消费者恢复、删除执行或持久 checkpoint 实现证据。
- [ ] 持久语义变换使用带自身来源的派生事件；没有把修改后的 payload 永久归因给原始 emitter。
- [ ] Fact / Observation / Proposal / Decision / State / Output 权力边界有类型或校验门禁。
- [ ] LLM 输出只能通过受约束包装器形成 Proposal，不能直接形成 Fact/State。
- [ ] 状态提交与 State 发布具备 outbox/reconcile 语义；重复投递不会重复写状态。
- [ ] Trace 关闭时行为等价，Replay 默认不写生产状态、不发送真实输出。
- [ ] 记忆正文、凭据和敏感 payload 没有进入通用事件记录。
- [ ] 主动链继续要求 Event 决策、一次性 Permit、用户输入抢占与输出结果回报。
- [ ] 技术债仍保持 OPEN/Partial，直到代码、恢复测试和目标 HEAD 证据满足仓库门禁。

---

## 相关文档

- [Event Ring 与主动回合契约](../plugin-and-architecture/EVENT_RING.md)
- [模块注册表](../../handoff/MODULE_MAP_AND_HANDOFF.md)
- [简架构](../../human-docs/01_ARCHITECTURE_SIMPLE.md)
- [技术债 `K-PROACTIVE-01` / `K-EVENT-STREAM-01`](../../handoff/TECHNICAL_DEBT_INVENTORY.md)
- [聊天与记忆存储边界](../../handoff/CHAT_STORAGE_ARCHITECTURE.md)
- [AI 改动边界](../../handoff/AI_CHANGE_BOUNDARIES.md)
