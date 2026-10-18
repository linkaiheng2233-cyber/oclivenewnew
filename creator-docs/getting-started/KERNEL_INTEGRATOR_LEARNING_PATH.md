# 内核集成方学习路径

面向 **无头 HTTP、嵌入式、硬件侧二次开发**：在自有设备上拉起与 **oclive 内核** 同契约的运行时。边界必读 [PURE_KERNEL_BOUNDARY.md](PURE_KERNEL_BOUNDARY.md)；脚手架见 **`oclive-cli`**（`cargo run -p oclive-cli -- …`）。

---

## 从六槽 Base 出发（约 10 分钟）

先运行 `cargo run -p oclive_kernel_runtime --example base_first_host`，再看 [示例源码](../../kernel/crates/oclive_kernel_runtime/examples/base_first_host.rs) 和 [Base-only 夹具](../../kernel/crates/oclive_kernel_contracts/tests/base_only_fixture.rs)。示例由一个外部小 Host 按需选择 `MemoryBase`、`PromptBase`、`LlmBase`；Memory 可以省略，Emotion／Event／Agent 不参与这一回合。它还验证额外 Prompt 要求被判 `Unsupported` 时不会继续调用 LLM。示例使用两项有限的内存参考实现和一个本地 Echo LLM，不读取完整 `Role`、数据库或 ChatPro 配置，不启动网络／真实模型。单次 poll 只适用于这些立即完成的实现，真实 Host 须用自己的执行器驱动 `BaseCallFuture`。

**验收方向**：先证明 Base 请求、正常空值与错误可独立调用，再核具体实现是否兑现自己的承诺，最后才验证 Host／发行版的适配与产品回合。六槽是可选能力而非固定六阶段；本示例既不是通用 Host 编排器，也不证明物理上已有独立的小 Kernel 发布包或 Stable API。

在仓库根目录按层复验（已有本地依赖缓存时可使用 `--offline`）：

```sh
cargo test -p oclive_kernel_contracts --test base_only_fixture --locked --offline
cargo test -p oclive_kernel_runtime --test base_memory --test base_prompt --test base_emotion --test base_event --test base_agent --locked --offline
cargo test -p oclive_kernel_host --test base_llm_adapter --locked --offline
```

第一条只验六槽契约及无 Host 的 Base-only 实现；第二条验五种有限具体实现；第三条只验 LLM 适配器的构造边界，不发送真实模型请求。之后再根据所选 Host 与发行版另测其装配、资源、持久化和产品行为，不能用下游 smoke 代替前三层。

**仓库外接入探针（2026-09-27）**：另用临时独立 Cargo 项目，仅以源码路径依赖 `oclive_kernel_contracts` 和 `oclive_kernel_types`，自行实现 `MemoryBase` 并通过 `&dyn MemoryBase` 调用；`cargo run --offline` exit 0，`cargo tree --depth 1` 只列出这两个直接依赖。探针文件测试后已清理，仓库工作树未因此改变。这证明源码级外部消费者可编译和调用该契约，不证明已有独立发布包、Stable API 或完整 Host 行为。

**参考接入案例**：[ChatPro Host 分层验收检查单](../../handoff/CHATPRO_HOST_KERNEL_INTEGRATION_GATE.md) 展示如何沿 Base → 所选 Host 能力 → 桌面桥逐层取证，并按改动面选择复验。案例只完成限定接入，不代表参考 Host 全面合规或 ChatPro 发布验收。

---

## 两条行为底线（自建 Host 前必读）

六槽 Base **只规定形状**（例如 `MemoryBase::retrieve` 返回 `Vec<String>`），不规定你的 Host 怎样组织一轮对话。因此**形状正确的 Host 仍然可能慢慢变坏**——而且这类变坏通常不报错、不崩溃、只在几十轮之后表现为"角色不太对劲"。

下列两条是**已被真实 Host 实际执行过**的底线。违反它们不违反类型签名，因此没有编译错误提醒你。完整条款见 [MODULE_MAP §0.5.1](../../handoff/MODULE_MAP_AND_HANDOFF.md#host-behavior-commitments)。

> **贯穿两条的原则**：内容必须带着它的来源进入下一层，不得在转运中丢失来源。

| # | 底线 | 自查方法 | 违反后（不报错，但会这样坏） |
|---|---|---|---|
| **H1** | **来源由 Host 决定，不由外部输入决定。** 接收外部内容的入口**不得有"来源"参数**；Host 自己产生的内容（传感器、定时器、内部事件）才用带来源的入口，且该入口不得对外暴露 | 找到接收外部内容的那个函数，看它有没有来源参数。有 → 追这个值从哪来；来自请求体/参数/插件载荷即违规 | 外部可把请求声明成低持久化来源，使整轮对话**静默地不写**好感、记忆与聊天。用户数周后才发现"角色不记得我" |
| **H2** | **由系统或观察材料发起的轮次，其内容不得最终成为用户发言**——不得出现在用户聊天记录、近期上下文、用户情绪分析或"用户说过"部分中。观察材料**可以**与用户发言共用同一传输字段，**前提是保证它被改道到观察通道并从用户发言中清除**；也可引用用户历史原话，但不得当作本轮用户发言 | 这一轮结束后，如果用户去看聊天记录，会不会看到自己没说过的话？ | 用户看到自己从未说过的话；该伪发言同时污染情绪分析、下一轮上下文与记忆提取 |

**H2 为什么按"结果"而不按"字段"写**：实践中把观察材料放进与用户发言相同的请求字段、再依赖编排把它改道到观察通道，是**正确**用法（真实硬件 Host 就是这样做的）。若按"字段必须为空"表述，这种正确用法会被误判为违规。

**「外部内容」怎么算**：按**值的来源**判，不按进程位置判——这个来源值是你**自己的代码算出来的**，还是**任何外部可写的输入算出来的**？同进程的第三方插件代码**算外部**：进程边界不等于信任边界。

**这两条不限制你的编排自由。** 你可以自定义阶段顺序、槽位调用次数、并发与缓存策略、是否复用参考编排——只要不违反 H1–H2。**它们约束的是副作用边界，不是执行结构。**

**当前无自动化门禁**：只能靠人工审阅。

### 另外四条：暂未列入

以下四项**已被识别、但尚无任何真实 Host 在实践中执行过**，因此没有进入公共条款，仅作为参考 Host 的不变量存在（[`BUS_FACTOR_NOTES` §11](../../handoff/BUS_FACTOR_NOTES.md)）：

| 候选 | 内容摘要 | 暂不列入的原因 |
|---|---|---|
| **程序化生成** | 后台/推测性生成默认不得写用户状态（防自我放大回路） | 尚无 Host 实现后台多拍生成 |
| **重试幂等** | 需要重试语义的调用须携带幂等标识 | 属通用工程常识；领域风险真实但无 Host 在其上执行 |
| **记忆来源** | 记忆进入上下文须标明来源，助手侧不得无标注冒充用户证据 | 无第二个 Host 验证该写法 |
| **表示层隔离** | 系统生成的协议标记与数值不得进入语义消费者 | 无 Host 实现分段/数值展示 |

**如果你正在实现上面某一项，请以实际形态为准**，并把你的用法反馈给维护者——条款应**按实现改写**，而不是让实现去适配未经检验的条款。H2 就是这样被修正的。

**如果你发现了这四条之外的新坑，欢迎提出**：判据见 [MODULE_MAP §0.5.1 归属判据](../../handoff/MODULE_MAP_AND_HANDOFF.md#host-behavior-commitments)。

---
## 想给 Base 加东西？先去 Extension

**默认路径：先做成独立增强（Extension），不要直接提议并入 Base。**

理由：Base 是**最低共同承诺**，不是能力上限；一旦某个增强被并入 Base，所有既有 Base-only 实现都会被迫多背一项义务，而"Base 不只是承诺"这件事一旦失守就再也收不回来。

**提议并入 Base 前，须先证明**：

1. 该能力**已作为 Extension 存在**，且至少有一个真实实现用过它（不是纸面设计）；
2. 缺少它时，合规 Host **无法**在不重复实现的前提下满足某类**公共**需求；
3. 该能力**不携带任何产品/发行版/领域词汇**（否则属 Extension，不是 Base）；
4. 加进 Base 后，既有 Base-only 实现仍**能编译**（在 Rust 中"加方法"通常是源代码 Breaking，见 [`COMPATIBILITY.md` §六槽 Base 兼容审阅范围](../COMPATIBILITY.md#六槽-base--最小逻辑角色的兼容审阅范围已采纳--2026-10-09)）。

四条不全满足时，正确结论是**继续留在 Extension**，而不是"先加进 Base 以后再收窄"。

**带默认实现的方法同样不构成理由。** 任何向 Base trait 增加方法（**含带默认实现的方法**）的变更，一律按 Breaking 审阅——默认实现会替实现者做它不知情的决定，并且声称了一个它没有的能力。**这类便利请写成扩展方法或独立 helper，不要加到 Base trait 上。**

**扩展压力通常不需要动 Base**，按成本从低到高有四级：

| 级 | 手段 | 代价 |
|---|---|---|
| 1 | 换某个槽的 backend（同一契约换实现） | 零 |
| 2 | 由一个实现承担多个槽（如把情绪逻辑并入记忆实现） | 放弃该槽的可替换性，由 Host 自行取舍 |
| 3 | 设施子模块（编排行内，不进六槽键） | 需登记 |
| 4 | 独立通道（自有 Resolver，不进主链） | 需登记 |
| — | **在槽外自成内核**（完全不同的领域，如硬件感知） | 完全自主，不属本契约范围 |

**真实需要新增槽位的情形，是上面四级都不能满足、且至少两个独立 Host 都提出同一需求时。**

---

## 参考 Host／CLI 入门（约 30 分钟）

| 步骤 | 做什么 | 读什么 |
|------|--------|--------|
| 1 | 理解「内核居中、六槽环绕」 | [KERNEL_AND_MODULES_ARCHITECTURE.md](KERNEL_AND_MODULES_ARCHITECTURE.md) |
| 2 | 理解「纯净内核」与 library / kernel_server 分工 | [PURE_KERNEL_BOUNDARY.md](PURE_KERNEL_BOUNDARY.md) |
| 3 | 生成最小无头/库骨架 | `cargo run -p oclive-cli -- init`（交互或 `--non-interactive`；见 [OCLIVE_CLI_GUIDE.md](../cli/OCLIVE_CLI_GUIDE.md)） |

**验收**：能在本机 `cargo build` 生成的骨架工程，并能区分生成工程的根级 `roles/` / `plugins/` 与主仓 `distros/chat-pro/roles/` / `plugins/`。当前非双核 `init` 示例仍是 legacy 参考宿主组合目录；现行参考宿主蓝图格式族的 Stable 样例以 v4 `pipeline.ocblueprint` 为 SSOT。两者都不是内核最小角色 contract；CLI 对该 contract 的生成/识别能力仍由 [TECHNICAL_DEBT_INVENTORY.md](../../handoff/TECHNICAL_DEBT_INVENTORY.md) 的 `D-CLI-BLUEPRINT-05` 跟踪。

---

## 进阶（约 1–2 小时）

| 主题 | 读什么 |
|------|--------|
| **`process_message` 编排** | 宿主参考实现 **`kernel/crates/oclive_kernel_host/src/domain/chat_engine/process_message.rs`** 与 **`turn_pipeline.rs`**（[handoff/BUS_FACTOR_NOTES.md](../../handoff/BUS_FACTOR_NOTES.md) 摘要） |
| **`PluginHost` 第 1–6 模块** | **`kernel/crates/oclive_kernel_host/src/domain/ports/plugin_host.rs`** · [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md) · [OCLIVE_ARCHITECTURE_OVERVIEW.md](OCLIVE_ARCHITECTURE_OVERVIEW.md) |
| **`slot_registry`、legacy `plugin_backends` 与兜底** | [SETTINGS_REFERENCE.md](../cli/SETTINGS_REFERENCE.md) · 远端失败回退相关设置见 [CONFIGURATION_FILES.md](../guides/CONFIGURATION_FILES.md) / 设置页「兜底」 |

**验收**：能描述一条 `send_message` 从入口到 LLM 再持久化的大致阶段名（便于对齐日志）。

---

## 高级（约半天）

| 主题 | 读什么 |
|------|--------|
| **OOCP / HTTP 与内核对话** | [OOCP_TEST_SUITE.md](../testing/OOCP_TEST_SUITE.md) · 示例 [`examples/oocp-test-suite/`](../../examples/oocp-test-suite/) · 无头最小示例 [headless-kernel-minimal](../../examples/headless-kernel-minimal/README.md) |
| **Monolith 使用场景** | 需要 **编译期七焊接键焊接**、减少动态解析时选用；[RFC_OCLIVE_MONOLITH_MODE.md](../rfc/RFC_OCLIVE_MONOLITH_MODE.md) · `oclive-cli init --monolith` + `build` / `bench` |
| **`--kernel-source`** | [OCLIVE_CLI_GUIDE.md](../cli/OCLIVE_CLI_GUIDE.md)（将脚手架依赖切到 path 的 `oclive_kernel_runtime` / `oclive_kernel_server`） |
| **进程内稳定门面** | `oclive_kernel_host::OcliveKernel`；生成 library 后按 [KERNEL_PLATFORM_DEVELOPER_PATH.md](KERNEL_PLATFORM_DEVELOPER_PATH.md) §5 完成角色加载、回合与关闭 |
| **平台单线文档** | [KERNEL_PLATFORM_DEVELOPER_PATH.md](KERNEL_PLATFORM_DEVELOPER_PATH.md) |

**验收**：能在目标设备上跑 **`--api` 健康检查**，或在自有进程中 `OcliveKernel::start → load_role → process_message → shutdown`；并完成一轮最小对话（可注入 mock `LlmClient`，或为 HTTP 开 mock LLM 环境变量）。

---

## 与主应用仓库的关系

- **契约**：DTO 与 `KernelErrorBody` 以 **`oclive_kernel_types`** 为准，端口 trait 以 **`oclive_kernel_contracts`** 为准；命名与载荷规则见 [KERNEL_ERROR_CODE_CONVENTION.md](KERNEL_ERROR_CODE_CONVENTION.md)。
- **完整宿主编排**只在 `kernel/crates/oclive_kernel_host` 维护一份，桌面薄壳位于 `distros/desktop-tauri`。可信 Rust 宿主应通过 **`OcliveKernel`** 使用它；`oclive-cli ... --project-type library --kernel-source` 已链接完整门面，而仅手工依赖 `oclive_kernel_runtime` 只提供参考引擎与辅助功能，不包含完整 Host 编排门面。
- 嵌入式可裁剪外围，但应保持 **错误 JSON 形状** 与 Event Ring 权威边界一致，便于共用 FAQ、编写器和诊断工具；不要直接依赖内部 `AppState` 拼第二条编排。
