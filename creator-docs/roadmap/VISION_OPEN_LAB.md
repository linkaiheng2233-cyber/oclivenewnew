# 可选愿景：开放实验工具（摘要）

OCLive 的本体仍是 **最小工具内核 + 六个稳定能力端口**。在此基础上，参考运行时可以组成一个本地优先的开放实验工具：研究者只替换某个槽实现，复用同一角色、回合、错误边界与测试基线；角色创作者则只接触角色包内容。开放实验是内核支持的一种装配，不是所有下游必须采用的平台形态，也不把角色包变成唯一可能的集成面。架构见 [OCLIVE_ARCHITECTURE_OVERVIEW.md](../getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md)。

模块可切换（builtin / remote / directory / 各槽特有 backend），契约与 CI 守兼容边界，文档与示例降低接入成本。强模型装配可以更薄，小模型装配可以使用更多显式辅助；比较对象应是可复现实验结果，而不是谁的流程更复杂。

**运行时双核**（Stable / Experimental 编排 + 可降级试验场）见 [RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md](../rfc/RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md)（Opt-in Beta，默认关，不阻塞当前发布）。它是可选实验机制，不改变最小内核定义。

**模型适配层（路线图）**：创作者工具链可补微调工坊，把口癖/节奏等沉淀为可打包的 LoRA/SFT adapter，由专家模型设施子模块（`expert_routing` · `slot.lora.apply`）按条件切换。它是参考装配能力，不取代六槽契约，也不成为内核必要条件。详见 [BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md](BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md) §五。

**TTFT · 双档思考（路线图）**：共在 co-present 路径按轮选择 **Fast**（规则 event · 裁剪上下文，首字 ~243ms 量级）或 **Deep**（全 enrichment）。Deep 侧下一阶段用 **离线 persona capsule**（`prompts/deep_capsule.txt`）缩短 Tier0，并用 **稳定前缀 KV 延续** 压多轮 prefill——见 [VISION_ROADMAP_MONTHLY.md](VISION_ROADMAP_MONTHLY.md) §Wave A–D 与 [`handoff/DEEP_PROMPT_DISTILLATION.md`](../../handoff/DEEP_PROMPT_DISTILLATION.md)。

**具身互动 · 性格驱动的「手脚」（路线图）**：除对话外，角色按人设 **在宿主上动手**——分 **被动**（用户开口 → 第 6 槽 agent + MCP 调工具）与 **自发**（idle / 虚拟时间 → 行为导演按七维与包级策略选动作）。破坏性好奇（如「小孩」建删文件夹）须在 **playroom 沙盒** 与用户显式授权内完成；**不**用整包通用 Agent 替代 co-present 聊天。分阶段交付与场景见 [VISION_ROADMAP_MONTHLY.md](VISION_ROADMAP_MONTHLY.md)「具身互动」专节与 [APPLICATION_SCENARIOS.md](APPLICATION_SCENARIOS.md) **S12**。

**已对齐的落实点（随主仓演进）**：HTTP JSON-RPC Remote 宿主路径、`plugin_backends` 与扩展点文档、目录式插件与整壳桥接、开源与多平台 CI。详见仓库根目录 [README.md](../../README.md) 的「路线图状态」与 [DOCUMENTATION_INDEX.md](../getting-started/DOCUMENTATION_INDEX.md)。

**仍属路线图**：包内知识库深化、启动器/市场联动、社区站形态等，以对应 roadmap 文档为准。
