# oclive 愿景落实 · 按月计划

本文把「开放平台 + 双软件 + 角色包 + 可替换记忆/情感 + 可选多语言插件」拆成**按月可交付**的里程碑。顺序可随人力微调，但**契约先于实现、默认实现先于真插件**的原则不变。

**产品首发（P0）**：桌面宿主当前执行视图见 **[`handoff/PRODUCT_LINE_TASK_BUCKETS.md`](../../handoff/PRODUCT_LINE_TASK_BUCKETS.md)**、**[`handoff/TECHNICAL_DEBT_INVENTORY.md`](../../handoff/TECHNICAL_DEBT_INVENTORY.md)**；发版按 **[CONTRIBUTING](../../CONTRIBUTING.md)** 与 CI 勾选。

---

## 愿景支柱（对照表）

| 支柱 | 含义 | 计划中对应项 |
|------|------|----------------|
| 开放 | 不追单点 SOTA，追**可替换、可文档化、可版本化** | 契约文档、trait 边界、开源准备 |
| 双软件 | **运行时（玩家）** 与 **创作者工具** 分离，**角色包**为唯一纽带 | 包规范强化、编写器、README 分工说明 |
| 角色即工作流 | 每个角色包是一套可声明的配置 + 可选后端 | manifest 扩展、`min_runtime`、后端枚举 |
| 记忆 / 情感可换 | 七维等只是**当前默认模块**，非平台上限 | Memory/Emotion 门面、第二套实现、远期侧车/WASM |
| **灵魂权重层** | 口癖、节奏、直播态等可沉淀为 **LoRA/SFT adapter**，与 prompt / 记忆并列；运行时由 **专家模型设施子模块** 按条件切换（`slot.lora.apply`），而非再做一个封闭「性格引擎」 | 微调工坊（独立创作者工具）、角色包 adapter 卫星文件、`expert_routing.json`、directory 推理插件 |
| **TTFT · 双档思考** | 闲聊 **Fast**（规则 event · 裁剪上下文）保首字；高价值轮 **Deep** 保质量；Deep 侧用 **离线 persona capsule + 稳定前缀 KV 延续** 压 prefill | Turn Thinking · `handoff/TTFT_BENCHMARK.md` · `handoff/DEEP_PROMPT_DISTILLATION.md` |
| **具身互动 · 性格驱动的「手脚」** | 角色不只会说：按人设 **被动调工具** 与 **idle 自发动作**；沙盒 playroom + 用户授权 + 频率/撤销策略；聊天仍走 co-present | 第 6 槽 agent/MCP · 拟 **独立通道** 行为导演 · [APPLICATION_SCENARIOS.md](APPLICATION_SCENARIOS.md) **S12** |

---

## 第 1 月：契约与代码边界（地基）— **已对齐当前实现**

**目标**：不动产品行为的前提下，把「能换什么」说清楚、接稳。

| 交付物 | 说明 |
|--------|------|
| `creator-docs/plugin-and-architecture/PLUGIN_V1.md` | 各子系统 DTO、`settings.json` 枚举；**已补充**「`send_message` 编排顺序」与 `chat_engine` / `PluginHost` 对照。 |
| `creator-docs/role-pack/PACK_VERSIONING.md` | 包版本、`schema_version`、`min_runtime_version`（预留）、未知字段策略；**已补充**第 1 月与 `plugin_backends` 的对照。 |
| Rust 门面 | 以 **[`PluginHost`](../../kernel/crates/oclive_kernel_host/src/domain/ports/plugin_host.rs)** 为宿主：[`MemoryRetrieval`](../../kernel/crates/oclive_kernel_runtime/src/domain/memory_retrieval.rs)、[`UserEmotionAnalyzer`](../../kernel/crates/oclive_kernel_runtime/src/domain/user_emotion_analyzer.rs)、[`EventEstimator`](../../kernel/crates/oclive_kernel_host/src/domain/event_estimator.rs)、[`PromptAssembler`](../../kernel/crates/oclive_kernel_runtime/src/domain/prompt_assembler.rs)、[`LlmClient`](../../kernel/crates/oclive_kernel_host/src/infrastructure/llm/mod.rs)；主流程只做编排。 |
| `settings.json` | 使用嵌套对象 **`plugin_backends`**（`memory` / `emotion` / `event` / `prompt` / `llm`），见 [`plugin_backends.rs`](../../kernel/crates/oclive_kernel_types/src/models/plugin_backends.rs)。**不再**使用独立字段名 `memory_backend` / `affect_backend`（早期愿景草案）；情感分析对应键 **`emotion`**。`builtin` / `remote`（及 `llm`: `ollama` / `remote`）为已实现枚举；`builtin_v2` 仅为读兼容 alias（等同 `builtin`）；`remote` 需环境变量时，加载角色时会 **记警告日志**（仍回退内置，与既有行为一致）。 |

**验收**：全量 `cargo test`、`npm run build`；对话与好感等行为与本月前**无回归**（或仅有可说明的显式变更）。

---

## 第 2 月：角色包编写器 MVP

**目标**：创作者**不靠手写 JSON** 也能产出可被运行时加载的包。

| 交付物 | 说明 |
|--------|------|
| 编写器形态 | 独立应用或 oclive 内「创作者模式」二选一；优先**独立**，避免与玩家端耦合过重。 |
| 功能范围 | `manifest.json` 门面字段、`settings.json` 基础段、**与后端同一套校验**（或调用/复用校验逻辑）。 |
| 导出 | 生成 `distros/chat-pro/roles/{id}/` 目录或 zip，结构与 [distros/chat-pro/roles/README_MANIFEST.md](../../distros/chat-pro/roles/README_MANIFEST.md) 一致。 |
| 文档 | 创作者路径：`creator-docs/getting-started/` 等 |

**验收**：用编写器新建/编辑一个包，**零手写 JSON** 可被 oclive 加载并正常对话。

---

## 第 3 月：证明「可替换」——第二套内置实现

**目标**：用**最小第二实现**验证 trait/配置链，而非追求更强效果。

| 交付物 | 说明 |
|--------|------|
| 第二套 Memory 或 Affect | 例如：记忆检索改为「简化 FIFO / 标签过滤」或情感侧「直通占位」；**行为可简单，接口要真走枚举**。 |
| 编写器 | 可选到第二套 backend（若该实现面向创作者开放）。 |
| 回归 | 默认 backend 仍为线上默认；切换路径有测试覆盖。 |

**验收**：同一角色包仅改 `*_backend` 字段，可观察到**可测差异**（日志或固定用例）。

---

## 第 4 月：外接插件协议草案 + 工程化

**目标**：为「多语言插件」留**正式插口**，先实现**一种**宿主侧调用方式。

| 交付物 | 说明 |
|--------|------|
| 协议草案 | 推荐 **子进程 + JSON-RPC（stdin/stdout 或本地端口）** 或 **gRPC**；文档写清版本、超时、错误码。 |
| 试点 | **记忆侧车**优先（重 IO、适合进程隔离）；情感管线可仍内置。 |
| 安全 | 不默认任意执行；manifest 声明可执行路径或 URL，用户确认策略写进文档。 |
| CI / 开源准备 | `LICENSE`、根 `README` 项目化、`.gitignore` 与密钥扫描；可选 GitHub Actions：`cargo test` + `npm run build`。**本仓库已加** `LICENSE`（Apache-2.0）、`NOTICE`、重写 `README`、`CONTRIBUTING` / `SECURITY`、`.github/workflows/ci.yml`。 |

**验收**：一个**最小外部 demo 插件**（任意语言）可被 oclive 调通一轮「检索/写入」mock。

---

## 第 5 月：包内「知识载体」与检索钩子

**目标**：自媒体/创作者**预写答案**随包分发、可版本更新。

| 交付物 | 说明 |
|--------|------|
| 包结构 | 如 `knowledge/`（Markdown 分块或 JSON FAQ）+ manifest 引用。 |
| 运行时 | 对话前 **检索/注入**（关键词或向量二选一先做轻量）；与现有 prompt 管线衔接。 |
| 编写器 | 知识块编辑与版本展示；与包版本联动。 |

**验收**：换包版本后，同一问题能反映**新预写内容**（在「以包为准」策略下）。

---

## 第 6 月：双软件叙事落地 + 可选启动器雏形

**目标**：对外说法与仓库结构一致；降低非开发者上手成本。

| 交付物 | 说明 |
|--------|------|
| 根 README | **软件 A（运行时）** / **软件 B（编写器）** 分工、安装方式、角色包放置路径。 |
| 启动器（可选） | 检测 Ollama、设置 `OCLIVE_ROLES_DIR`、拉起运行时；**可与编写器分阶段**，不必同月完成。 |
| 扩展点索引 | `creator-docs/plugin-and-architecture/EXTENSION_POINTS.md`：列出稳定 trait、manifest 字段、外接协议版本。 |

**验收**：新用户仅读 README 能分清「玩」与「做包」两条路径。

---

## 第 7 月及以后（ backlog，按需排）

### TTFT 与 Deep 精炼（Wave A–D · 2026-06 起）

**已交付（Wave A–E）**：发行版 `event_impact_llm` · Turn Thinking Auto/Fast/Deep · co-present Fast 裁剪 · `scripts/measure-ttft.mjs` · Chat Pro co-present **p50 ~243ms**（bench profile `desktop-latency`，见 [`handoff/TTFT_BENCHMARK.md`](../../handoff/TTFT_BENCHMARK.md)）· **Wave C** 主 UI `/chat/stream` · **Wave D** Small+Deep **`prompts/deep_capsule.txt`** · **Wave E** `fast_persistence = strong_only`（RFC [`RFC_TURN_THINKING_PERSISTENCE.md`](../rfc/RFC_TURN_THINKING_PERSISTENCE.md)）。

**第 7 月+ backlog（Wave D-T3）**：

| Wave | 目标 | 说明 |
|------|------|------|
| **D-T3 / D+** | **上下文延续（Large + KV）** | Prompt 拆 **稳定前缀 / 可变后缀**；**Large 模型全文 + KV 延续** — 待 34B+ 默认用户或 bench 需求 |

**架构归类 SSOT**：规则 event = **第 3 模块** optional 子路径 + **HostProfile** 开关；Turn Thinking = **编排行无编号设施**；capsule = **角色包卫星 + 第 4 模块组装分支**。详见 **[`handoff/DEEP_PROMPT_DISTILLATION.md`](../../handoff/DEEP_PROMPT_DISTILLATION.md)**。

**纪律**：蒸馏 **仅离线/包内**；不新增 `process_message` LLM 压缩 stage；`KERNEL_DIALOGUE_GUARDRAILS` 不可被 capsule 替换。

| 方向 | 说明 |
|------|------|
| WASM 插件 | 在进程插件稳定后，对计算型扩展做沙箱化。 |
| 动态 `.dll`/`.so` | 仅在有强需求与 ABI 规范时考虑；默认不推荐。 |
| 奖杯 / 关系仪式、多模式（纯聊 / 沉浸）细化 | 与产品节奏对齐，可插入各月小迭代。 |
| 生态 | 示例包、模板仓库、贡献指南 `CONTRIBUTING.md`。 |

### 三发行版结项之后 · 微调工坊（创作者工具链第三阶段）

**定位**：在 **Chat Pro / VS Code Flash / AI Theater** 工程 smoke 结项、编写器简单创作闭环可用之后，补创作者工具链的 **权重层**——使「灵魂」不只有 prompt / 记忆 / 关系，还可把口癖、节奏、直播态等沉淀为 **可打包、可校验、可分发** 的小模型 adapter（LoRA / SFT 等）。

**动机（产品）**：垂直 AI 角色实践（如 AI 主播）表明，仅靠人设 prompt 难以长期锁死说话习惯；OClive 的差异化应落在 **微调产物是角色包模块 + 专家路由在运行时按需切换**，而不是与 EchoVessel 等拼「谁的记忆/情感引擎更强」。

**与现有架构的接点**（已实现或预埋，默认关 / 待产品化）：

| 项 | 说明 |
|----|------|
| **第 2 设施子模块** | **专家模型设施子模块** · `blueprint/includes/expert_routing.json` · 条件触发子流程 |
| **`slot.lora.apply`** | 专家步骤：会话标记 `plugin_id` 以切换 adapter（`dual_core` + Experimental 核；Stable 主链不接，直至解冻决策） |
| **第 5 模块 `llm`** | 主对话仍走通用 `plugin_backends.llm`；**默认**微调 adapter 仅在 expert 子流程切换，不强制替换主槽 |
| **编写器** | 导出 `.ocpak` / `distros/chat-pro/roles/`；工坊产出写入包内卫星文件（契约待 RFC） |

**分阶段交付（T0→T3，按需排期）**：

| 阶段 | 交付物 | 验收 |
|------|--------|------|
| **T0 · 契约** | RFC：语料来源与隐私、`lora_adapters`（或等价）卫星 schema、与 `expert_routing` / `slot.lora.apply` 引用关系、导出 profile（`desktop-full` / `vscode-lite` / `theater`） | 文档评审；`oclive_validation` 键表草案 |
| **T1 · 工坊 MVP** | **独立桌面/Tauri 工具**（推荐与 pack-editor 并列，避免 GPU/训练进程拖慢编写器）：导入对话/人设样本 → 单 base 模型 LoRA 或等价 → 导出 adapter 清单进角色包目录 | 产物经校验 crate 检查；零手写 JSON 可装入 `distros/chat-pro/roles/{id}/` |
| **T2 · 运行时** | directory 推理插件或 Ollama modelfile 等路径；`slot.lora.apply` **真加载** adapter 并参与 generate | 专家路由命中时，同一角色可观测 prompt-only vs adapter 差异（固定用例或日志） |
| **T3 · 评测** | 扩展 bench / OOCP / replay：**prompt-only · LoRA · LoRA+专家路由** 可复现对比（连贯 / 口癖 / 人设一致性 checklist） | 维护者可跑一轮对比报告 |

**纪律（与冻结期对齐）**：

- **训练、显存、数据集清洗** 放在工坊或 directory 侧车；**内核仍薄编排**，不新增 `process_message` stage 承载训练。
- **`expert_routing` / `dual_core` 产品冻结期内**：仅推进 T0 契约 + T1 工坊原型（可选 feature 分支），**不接 Stable 主链**；解冻条件见 [TECHNICAL_DEBT_INVENTORY.md](../../handoff/TECHNICAL_DEBT_INVENTORY.md) §冻结决定。
- **Theater v0 陌生人测试** 完成前，不把微调工坊标为 P0 阻塞项；与 [RECURRING_OPTIMIZATION_PLAYBOOK.md](../../handoff/RECURRING_OPTIMIZATION_PLAYBOOK.md) §9 元纪律一致——先让样板「发光」，再扩创作者楔子。

**体验向细化**见 [BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md](BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md) §五–§六；场景矩阵见 [APPLICATION_SCENARIOS.md](APPLICATION_SCENARIOS.md) **S11** · **S12**。

### 具身互动 · 性格驱动的「手脚」（Playroom · 2026-06 起 · backlog）

**定位**：让角色 **长出手脚**——在宿主上执行与性格一致的小动作（如高好奇心「小孩」在沙盒里建/删文件夹），同时 **共在聊天** 仍是 mumu，不被整包通用 Agent 短路。

**两种模式（均须人格约束，触发不同）**：

| 模式 | 触发 | 典型能力 | 现状 |
|------|------|----------|------|
| **被动** | 用户消息（「帮我在小房间里建个文件夹」） | 第 6 槽 **agent** + **MCP** 工具；`AgentRoleConstraints` 已带七维/关系/场景 | 骨架已有；Chat Pro 常 `skip_agent`；须 **playroom 沙盒** 工具契约 |
| **自发** | idle、虚拟时间、用户开启「允许自主探索」 | **行为导演**（拟独立通道，不进六槽主链）→ LLM 选意图 → 沙盒执行 → 记忆/UI 反馈 | **未产品化**；区别于 `autonomous_scene`（仅虚拟位移） |

**分阶段交付（P1→P3，按需排期）**：

| 阶段 | 交付物 | 验收 |
|------|--------|------|
| **P1 · 被动手脚** | playroom 目录 + 宿主 MCP（`mkdir` / `list` / 限频 `delete`）；用户显式请求时 agent 可调用；可选 UI 展示「角色小房间」 | 固定用例：授权后用户一句话可在 playroom 建文件夹；**不可**写用户真实桌面 |
| **P2 · 半主动** | 行为导演 + idle 调度；包级或七维策略（好奇心高 → 探索型动作）；通知 + 写入长期记忆；设置项「允许自主探索 playroom」 | idle 触发可观测沙盒动作 + in-character 反馈；可一键撤销/清空 playroom |
| **P3 ·  richer** | VS Code 等发行版：**虚拟/沙盒** 工作区互动（非真删仓库）；可选外部执行器（OpenClaw/Hermes 等）**仅**在 playroom 工具集内多步探索 | 与渗透插件模型对齐；外部引擎不接管 `process_message` |

**纪律**：

- **聊天**走 co-present；**动手**走 agent 或独立通道，失败不拖垮主链。
- 破坏性动作 **默认 playroom**；`high_risk_grants` 与 [`AGENT_REMOTE_PROTOCOL.md`](../plugin-and-architecture/AGENT_REMOTE_PROTOCOL.md) 边界不变。
- 不把 OpenClaw/Hermes **整包**塞进 `plugin_backends.agent` 当默认聊天大脑；至多作 P3 沙盒内 skill 执行器。
- 模块归类 SSOT：agent = **第 6 槽**；行为导演 = **独立通道**（见 [`handoff/MODULE_MAP_AND_HANDOFF.md`](../../handoff/MODULE_MAP_AND_HANDOFF.md) §11）；T0 契约 RFC **待开**。

**补充（体验向 backlog）**：编写器内试聊、启动器智能依赖、角色/插件市场与愿景对照的合并清单见 **[BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md](BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md)**（与本文并行维护，供排期引用）。

**中长期技术债务（第四批 · 2026-05）**：`library` 对称、`多模态/打断/多租户`、硬件靶子、边缘 OTA、市场 UGC 等 **延后**。**当前已为未来拓展预留空间，启动前请先阅读** **[handoff/TECHNICAL_DEBT_INVENTORY.md](../../handoff/TECHNICAL_DEBT_INVENTORY.md)** 中各项 **「预留设计」** 小节（预留原因 / 已有拓展基础 / 启动注意事项）。**A1.1c** 原生窗 WebDriver 烟测 **基础建设已启动**（非全屋 E2E）。**T05–T13** 编写器组件用例 **已全部覆盖**（见 [testing/OVERVIEW.md](../testing/OVERVIEW.md)）。

---

## 每月固定习惯（建议）

- **契约变更**走文档 + 版本号，避免静默改字段。  
- **默认路径永远可回退**：新后端挂了能切回 `default`。  
- **测试**：trait 切换与包加载至少有一层自动化覆盖。

---

## oclive-cli 脚手架（计划中）

以下方向**尚未**在 `oclive-cli` 实现；勿写入「十二阶段 / 已完成能力」总览。落地时优先保持 **A 级主轴** 简洁，避免重复 `market` / `template` 入口。

| 方向 | 说明 |
|------|------|
| `pack diff` / `pack update` | 角色包版本 diff 与依赖升级检查 |
| `oclive kernel update` | 生成工程的内核 path 依赖版本对齐主仓 |
| `dev --inject` | 热注入测试消息并观察步骤追踪 |
| `bench history clear` / `export` / `import` | 基准历史管理（当前仅有 `--save` / `--history` / `--compare`） |

---

## 文档索引

- 角色包契约：[distros/chat-pro/roles/README_MANIFEST.md](../../distros/chat-pro/roles/README_MANIFEST.md)  
- 创作者向：[../role-pack/CREATOR_ROLE_PACK_CUSTOMIZATION.md](../role-pack/CREATOR_ROLE_PACK_CUSTOMIZATION.md) 等  
- 体验差异化与愿景对照 backlog：[BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md](BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md)  
- 本月计划若与实现不一致，**以仓库代码与校验为准**，并回写本文。

---

*本文档随愿景迭代更新；重大方向变更时请改日期与版本说明。*

*2026-06-15：新增愿景支柱「灵魂权重层」与「三发行版后 · 微调工坊」专节（T0–T3）。*  
*2026-06-25：新增愿景支柱「TTFT · 双档思考」与 §Wave A–D（Deep persona capsule · 前缀 KV 延续）。*  
*2026-06-26：新增愿景支柱「具身互动 · 性格驱动的手脚」与 §Playroom P1–P3 backlog。*
