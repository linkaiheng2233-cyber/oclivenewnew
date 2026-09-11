# 03 · 术语表

> **读者**：读源码或 PR 前需要统一缩写的工程师。  
> **读完能做什么**：看到 `srid` / `mrid` / `pl` 等缩写知道含义；区分 `slot_registry` 与 `plugin_backends`。  
> **耗时**：约 20 分钟（可边查边用）。  
> **下一篇**：[04 工程约束](04_ENGINEERING_RULES.md)。

---

## 会话与角色 ID

下列缩写、字段和运行策略描述**当前参考 Host**，不是最小 Kernel 必须公开的字段。权责查 [MODULE_MAP](../handoff/MODULE_MAP_AND_HANDOFF.md#kernel-responsibilities)，名称查 [NAMING](../creator-docs/NAMING_CONVENTIONS.md)。

| 缩写 | 全称 | 含义 | 代码锚点 |
|------|------|------|----------|
| **`mrid`** | manifest role id（历史简称） | 当前参考 Host 的角色 ID；来源按所加载格式解析，不代表所有包都须有 manifest | `SendMessageRequest.role_id` |
| **`srid`** | session-scoped role id | SQLite / 缓存命名空间：默认等于 `mrid`；有 `session_id` 时变为 `{mrid}::{session_id}` | [`conversation_state_role_id`](../kernel/crates/oclive_kernel_host/src/domain/chat_engine/mod.rs) |
| **`pl`** | plugin layer / resolved plugins | 本回合解析后的 `ResolvedRolePlugins`（六槽 `Arc<dyn …>` 句柄集） | `process_message` 内 `pl` 变量 |

**示例**：HTTP 试用聊天带 `session_id` 时，记忆与 `role_runtime` 行按 **`srid`** 隔离，不与默认会话混用。

---

## 架构核心

| 术语 | 含义 |
|------|------|
| **最小工具内核** | 六槽能力契约与必要公共合法性/因果/错误边界；不拥有具体调度与领域提交，当前尚未物理抽成独立 crate |
| **发行版 Host** | 运行组合、有限自由的调度与领域应用层；不是前后端通信桥或 OS 进程的同义词 |
| **Adapter** | 协议转换或执行已授权操作；不自行取得领域决策权 |
| **完整参考运行时** | 当前 `oclive_kernel_host::OcliveKernel` 门面及其 SQLite、Event Ring、具体槽实现、设施和 HTTP 依赖；可嵌入，但不等于最小 core |
| **`PluginHost`** | 按角色包 + 会话覆盖解析六槽实现；入口 [`plugin_host/mod.rs`](../kernel/crates/oclive_kernel_host/src/domain/plugin_host/mod.rs) |
| **`slot_registry`** | 参考 Host 的 v2/v3/v4 蓝图多实例槽位表；该格式族新 Stable 包用 v4，不是最小角色必填项 |
| **`plugin_backends`** | legacy 六键磁盘结构 + Rust 运行时折叠类型名 `PluginBackends`；**不是**当前蓝图 UI 的写盘真源 |
| **`slot_registry.type`** | 与六槽键同义；**禁止**别名 `memory_backend` 等 |
| **`OOCP`** | OCLive Open Chat Protocol；现行 HTTP 黑盒范围见 [OOCP_TEST_SUITE](../creator-docs/testing/OOCP_TEST_SUITE.md) |
| **`co_present`** | 当前参考 Host 的 Stable 共景主路径实现模块 |
| **Event Ring / 事件外环** | 当前参考运行时中的有界事件路由与权威信封设施；不是最小核心必选项、第七槽或数据库总线 |
| **显式 / 隐式装配** | 两种可混合策略：显式模块共享契约化状态；隐式装配让模型从语境推断。OCLive 不指定唯一正统 |
| **状态候选** | 带来源、置信度、有效期与作用域的语义判断目标形态；当前并非所有 DTO 都已实现这些字段 |
| **[Runtime Event Stream / 角色运行事件流](../creator-docs/rfc/RFC_RUNTIME_EVENT_STREAM.md)** | 外围跨回合/跨通道事件流方向；trace/shadow 与后续实验不等于正式 Production Stream。逐阶段已实现范围、缺口和暂停项查 RFC 与技术债，不与当前有界 Ring 混称 |
| **Session（事件语境）** | 角色运行实例与隔离边界；可承载/投递属于该实例的事件，但不因此取得 Event 采纳或状态提交权 |
| **legacy `event` 槽** | 第 3 后端模块，只估计对话事件类型/影响；不是整个 Event Ring |
| **EventDraft** | 模块提出的事件草案；不含可信来源、注册权重和顺序 |
| **EventEnvelope** | Ring 签发身份、来源、权重、顺序和因果后的权威事件信封 |
| **EventEmitter** | 模块注册后取得的来源绑定发射器；只能提交声明范围内的草案 |
| **基础影响权重** | 决策模块评估提案时的基础影响力；不是优先级、频率或最终决定 |
| **Event 决策模块** | 对特定事件提案作采纳/拒绝并可派生事件；不是 Rust 总编排 |
| **TurnOrigin** | 回合来源：`user` / `sensor` / `system`；外部客户端不能自行选择低持久化来源 |
| **TurnInput** | 当前输入的类型化边界：`UserMessage` 或 `ExternalObservation` |
| **ProactiveTurnPermit** | Event 授权后由宿主签发的一次性主动回合许可；不能由插件构造 |
| **Turn Thinking** | 编排行 Fast/Deep（**非第七槽**）；Wave E 持久化分流 · Wave F 包级路由 / latch / ephemeral |
| **Fast / Deep** | 本回合思考档位：Fast 跳过部分 LLM 与（在 `strong_only` 下）长时巩固；Deep 全量 |
| **ephemeral_archive** | 规则写的临时局面摘要（TTL），Prompt 段 `【局面摘要】`；与 `mutable_personality` 独立 |
| **第 3 设施（立绘）** | `portrait_catalog` · AI **表现导演** 选 `visual_state_id`（RFC 草案；v0.3 仍为文件名 + 七 tag） |
| **第 4 设施（视觉表现）** | **角色舞台**：`visual_state_id` → `performance_directive` 与发行版 gating 已交付；Live2D / 3D / 演算 adapter 部分交付，默认关 |

---

## 磁盘与配置

| 术语 | 含义 |
|------|------|
| **角色包** | `distros/chat-pro/roles/{id}/`：身份、人格、`prompts/` |
| **蓝图文件** | `pipeline.ocblueprint`（文件名冻结）；含 `slot_registry`、`groups` |
| **`{app_data}`** | Tauri 应用数据目录；含 **`app.db`** |
| **`reply`** | AI 回复契约字段名；**不是** `response` |

---

## 与 NAMING 交叉引用

完整权威名、禁止别名、crate 边界：[creator-docs/NAMING_CONVENTIONS.md](../creator-docs/NAMING_CONVENTIONS.md)

- §1.3 六槽键名
- §4.2 Canonical import
- §5 `slot_registry` vs `plugin_backends`
- §6 禁止别名表

---

## 验收

- [ ] 能解释 `srid` 与 `mrid` 何时相同、何时不同
- [ ] 能区分 `slot_registry`（v2/v3/v4 蓝图磁盘配置）与 `PluginBackends`（运行时六槽折叠）
- [ ] 能区分 `event` 槽、Event Ring、Event 决策模块与 Rust 编排
- [ ] 能说明当前 Event Ring 为什么不等于持久化 Runtime Event Stream

---

## 深度链接

- [ROLE_PACK_BOUNDARY](../handoff/ROLE_PACK_BOUNDARY.md)
- [RFC_TURN_THINKING_PERSISTENCE](../creator-docs/rfc/RFC_TURN_THINKING_PERSISTENCE.md) — Fast/Deep · 持久化 · 包级路由
- [SETTINGS_REFERENCE](../creator-docs/cli/SETTINGS_REFERENCE.md)
- [EVENT_RING](../creator-docs/plugin-and-architecture/EVENT_RING.md)
