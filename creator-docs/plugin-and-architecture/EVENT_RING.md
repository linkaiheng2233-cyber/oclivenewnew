# Event Ring 与主动回合契约

**SSOT 范围**：Event Ring 的事件信封、注册策略、权威边界、现有事件种类与主动回合授权契约。模块编号与归类见 [MODULE_MAP](../../handoff/MODULE_MAP_AND_HANDOFF.md)，实施进度与缺口见 [TECHNICAL_DEBT_INVENTORY](../../handoff/TECHNICAL_DEBT_INVENTORY.md)。
**最后更新**：2026-09-01。
**读者**：内核集成方、Event 模块作者、目录插件作者与维护者。

---

## 1. 定位

Event Ring 是每个 `AppState` 独立、进程内、有界且不写数据库的事件外环。它允许已注册模块提出、补充和变换事件，并为事件签发可信的身份、来源、基础影响权重、顺序与因果链。

这里的“外环”只表示单次 dispatch 的路由边界，“权威”只覆盖信封身份与注册策略。Event Ring 不是持续轮询模块的调度循环，不判断外部事实绝对真伪，也不拥有提案采纳或角色状态提交权。

Event Ring **不是**：

- 第七个后端槽；
- 按固定回合依次调用所有模块的 pipeline；
- 聊天记录或数据库事件总线；
- 绕过 `process_message` 的第二套回复引擎；
- 允许任意插件自行触发主动回复的入口。

legacy `event` 槽仍是第 3 后端模块，但只负责对话 `event.impact` 估计；其结果通过兼容桥进入 Event Ring。模块归类只在 [MODULE_MAP §6](../../handoff/MODULE_MAP_AND_HANDOFF.md#6-第-3-模块--event) 维护。

### 1.1 与未来 Runtime Event Stream 的边界

当前 Event Ring 处理一次 dispatch 内的可信事件流通，**不等于**角色跨回合、跨通道、跨重启持续运行的 Session 时间线。后续讨论中的“角色运行河流”暂称 **Runtime Event Stream**：它可能承载 QQ、直播、游戏、传感器、记忆与 Agent 等来源的连续事实及派生事件；边界已进入 Draft RFC，但 wire、存储和消费者接口尚未冻结或实现。

未来设计必须保持以下边界：

- Runtime Event Stream 不替代 Event Ring；前者管理持续时间线与消费进度，后者仍负责可信信封、注册策略与有界路由；
- Runtime Event Stream 不替代 Stable 回合管线，也不建立第二套 `process_message`；
- Session 是角色运行实例与隔离边界，不因负责投递而自动取得事件采纳、人格修改或状态提交权；
- 消费模块可观察、查询、提出和派生事件，但状态变化仍须经过 Event 决策与 Rust 编排；
- 大模型可以获得更广的观察权、查询权和提案权，但不能伪造事实或直接提交权威状态；小模型继续消费经模块筛选和 Prompt 编译的有限上下文；
- 持久轨迹、重放、消费游标、背压、幂等与隐私策略属于未来 Stream/Trace 契约，不能由当前有界诊断历史冒充。

边界草案见 [`RFC_RUNTIME_EVENT_STREAM`](../rfc/RFC_RUNTIME_EVENT_STREAM.md)，设计与实施状态只在 [TECHNICAL_DEBT_INVENTORY 的 `K-EVENT-STREAM-01`](../../handoff/TECHNICAL_DEBT_INVENTORY.md) 维护。在实现与恢复测试完成前，“河流”仍不是已交付能力。

---

## 2. 四层权威

| 层 | 拥有的权力 | 不拥有的权力 |
|----|------------|--------------|
| Rust 回合编排 | 决定何时派发事件、消费结果、运行 Prompt/LLM、提交状态 | 不替各能力槽实现检索或生成算法 |
| Event Ring | 签发事件身份、来源、注册权重、顺序、关联与因果；执行有界路由 | 不直接决定角色说什么，不把权重当执行顺序 |
| Event 决策模块 | 对自己订阅的提案作采纳、拒绝或派生事件 | 不能伪造来源、提高自身注册权重或签发宿主 Permit |
| 能力模块/槽 | 产生领域证据或提案，例如事件影响估计、记忆候选 | 不能自行宣称提案已被采纳，不能绕过回合编排写最终状态 |

本文所称“权威事件”表示来源、身份、因果链与对应决策均已通过当前边界校验，**不表示角色状态已经提交**。只有 Rust 编排成功应用结果后，状态变化才成立；后续若记录 `State` 事件，也必须是提交后的事实，而不是促成提交的命令。

基础影响权重使用定点比例：`10_000 == 1.0`。它只影响下游决策模块评估提案时的基础影响力，不控制模块优先级、调用频率或最终采纳结果。执行顺序由 `(priority, module_id)` 确定。

---

## 3. 数据与注册契约

```text
EventModuleDeclaration              EventModuleRegistryPolicy
（模块声明）                         （可信宿主分配）
├─ module_id                         ├─ influence_weight_bps
├─ subscriptions                    └─ failure_mode
├─ emissions                            ├─ fail_fast
└─ priority                             └─ isolate
           ↓
模块提交 EventDraft
           ↓
来源绑定 EventEmitter → Event Ring → EventEnvelope
```

模块只能提交 `EventDraft { kind, payload, metadata }`。Event Ring 为 `EventEnvelope` 补齐并保持以下字段权威：

- `event_id`、`source`、`source_weight_bps`；
- `stream_key`、`correlation_id`、`causation_id`；
- `sequence`、`depth`、`occurred_at`。

已注册模块可以按声明替换当前 payload、合并 metadata 或发射子事件，但不能修改上述权威身份字段。内置模块默认 `fail_fast`；不可信适配器应使用 `isolate`，其错误或越权输出会被原子拒绝并隔离。

公共 Rust 契约：

- 数据类型：`oclive_kernel_types::event_ring`；
- 端口：`oclive_kernel_contracts::event_ring`；
- 宿主实现：`oclive_kernel_host::domain::event_ring`。

---

## 4. 已接入的事件链

### 4.1 对话事件影响兼容桥

```text
legacy event 槽估计
  → kernel.chat.event_impact.estimated
  → Event Ring 模块可在契约内补充或变换
  → Rust 编排消费最终 EventImpactEstimate
  → 人格、好感与关系预览
```

未注册处理模块时，兼容桥保持估计结果逐字段不变。

### 4.2 记忆回想

```text
memory 检索候选
  → kernel.memory.recall.candidate
  → builtin.event_decision
  → kernel.memory.recollection.activated（可能无）
  → 本轮专用回想 Prompt 段
```

提案事件只携带记忆 ID、置信度和相关度，不复制记忆正文。只有被激活的记忆进入专用回复上下文；当前 TTL 固定为一轮。正文在 Ring 外按 ID 回取并经过安全证据提取。

### 4.3 目录插件

目录插件通过 manifest `eventRing` 与 JSON-RPC `event_ring.handle` 接入。manifest 内容是不可信建议，宿主负责订阅白名单、插件自有发射命名空间、权重上限、固定优先级、超时和 `isolate` 策略。详细 wire 只在 [DIRECTORY_PLUGINS §4.4](DIRECTORY_PLUGINS.md#44-内核-event-ring-适配与-bridgeevents-无关) 维护。

目录插件当前不能读写主动回合内核事件，也不能取得主动 emitter 或 `ProactiveTurnPermit`。

---

## 5. 经授权的主动回合

```text
可信输入适配器
  → kernel.proactive.turn.proposed
  → builtin.proactive_turn_decision
  → kernel.proactive.turn.authorized
  → 一次性 ProactiveTurnPermit
  → process_proactive_turn
  → origin-aware 共景回复
```

安全边界：

- 提案不能自报来源或基础权重；来源由注册所得 `EventEmitter` 绑定。
- `origin` 只能是 `sensor` 或 `system`，不能冒充 `user`。
- 只有来源、因果链和分数范围均合法的权威授权事件才能换取 Permit。
- Permit 不可公开构造、不可克隆、不可反序列化，只能消费一次。
- Ring dispatch 完成后宿主才调用 Turn Engine，Event handler 内不得递归发起回合。
- 外部观察使用 `TurnInput::ExternalObservation`；`SendMessageRequest.user_message` 保持为空。
- 当前非用户回合不调用用户情绪或 legacy event 路径，也不写用户聊天、STM/LTM、关系、人格、连续性、虚拟时间或旧 `events` 表。
- remote-life Prompt 尚未支持来源感知，非用户回合暂时强制走 co-present Prompt。

当前执行器是基础设施，不是产品化主动 Bot。去重、TTL、冷却、频率限制、用户输入抢占、主动输出端口和第三方根事件 ingress 的状态只在 [TECHNICAL_DEBT_INVENTORY 的 `K-PROACTIVE-01`](../../handoff/TECHNICAL_DEBT_INVENTORY.md) 维护。

---

## 6. 诊断与隐私

`EventRingDiagnostics` 提供确定性的注册表、模块运行状态、历史容量、序号以及事件路由/因果摘要。诊断默认不包含：

- payload 内容；
- metadata 值；
- `stream_key`。

它是只读、最终一致的内核诊断，不是事件持久化 API。

---

## 7. 改动检查表

- [ ] 新事件种类使用规范点分命名，并声明订阅/发射范围。
- [ ] 模块声明与可信注册策略保持分离。
- [ ] 提案、采纳、执行三个阶段使用不同事件或类型边界。
- [ ] 不把 influence weight 当优先级或最终决定。
- [ ] 外部观察不进入用户消息字段，不取得用户身份语义。
- [ ] 新主动入口不能绕过 Permit、持久化边界和后续调度状态机。
- [ ] 目录插件同时覆盖正常、越权、超时和隔离路径。
