# Oclive 架构总览：工具内核、六槽与外围装配

**SSOT 范围**：对外架构叙述、模块编号与分层术语；模块定义见 MODULE_MAP，wire 契约见专题文档。
**最后更新**：2026-09-05。

本文是 **对外架构叙述** 与 **模块编号和分层术语** 的权威页：先定义最小工具内核，再说明当前完整运行时中的单核双态构建、**后端模块（第 1–6 模块）**、设施、独立通道和插件实现。

**模块定义 · 六槽/设施关系 · 改动约束（维护 SSOT）**：[`handoff/MODULE_MAP_AND_HANDOFF.md`](../../handoff/MODULE_MAP_AND_HANDOFF.md) — 本文侧重对外叙述与编号脚注，**不**与注册表双写长表。实现细节仍以 [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md)、[SETTINGS_REFERENCE.md](../cli/SETTINGS_REFERENCE.md)、[PURE_KERNEL_BOUNDARY.md](PURE_KERNEL_BOUNDARY.md)、[RFC_OCLIVE_MONOLITH_MODE.md](../rfc/RFC_OCLIVE_MONOLITH_MODE.md)、[RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md](../rfc/RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md) 与源码为准。

[English](../../creator-docs-en/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md)

---

## 架构简述

**Oclive** 的最小概念核心是一颗 **契约型工具内核**：它拥有唯一回合/生命周期编排、能力调用与合并规则、权威状态提交、错误语义和故障隔离；记忆、情感、事件、Prompt、LLM、Agent 通过 **PLUGIN_V1 六个稳定能力端口**接入。六槽提供能力、证据、候选、上下文或动作结果，不是六个平级权威。

当前 `oclive_kernel_host::OcliveKernel` 是可直接嵌入的**完整参考运行时门面**，仍物理装配会话状态、SQLite、Event Ring、HTTP 依赖、具体六槽实现和设施。它复用唯一 `process_message`，但不等于已经物理抽离的最小 core；拆薄状态见 `K-CORE-BOUNDARY-01`。**复杂情感设施子模块**、专家模型设施子模块等可以服务 Prompt，却不是第七槽，也不是 OCLive 成立的必要条件。

**Event Ring** 是当前参考运行时中的可复用事件设施：它形成进程内有界事件外环，不占六槽位置，也不替代 Stable 回合管线。legacy `event` 槽只估计对话事件影响；memory、传感器等来源可提出事件，由注册的决策模块采纳或拒绝，最终仍由 Rust 编排决定如何进入 Prompt、回复与持久化。没有 Event Ring 的更薄装配仍可以遵守六槽契约；公开契约见 [EVENT_RING.md](../plugin-and-architecture/EVENT_RING.md)。

OCLive 不规定情绪、关系等语义必须全部显式化或全部交给模型隐式推断。默认参考实现偏向本地小模型，使用较多显式辅助；强模型装配可以更薄。推荐边界是：**事实显式化，判断候选化，表达模型化**。当前 `EmotionResult` 仍主要是七维数值；`source / confidence / TTL / scope` 等候选元数据是目标原则与技术债，不能写成已完成契约。

在 **交付** 上借鉴 **发行版纪律**：通过稳定 HTTP / **OOCP** 黑盒契约、角色包规范与 **`oclive-cli` 内核工厂**，产出可独立部署的 **无头内核**（`--api` / `kernel_server`）或 **桌面宿主**（Tauri + Vue）。主仓 Chat Pro 的角色内容面是 `distros/chat-pro/roles/{角色id}/`；`oclive init` 生成的独立工程使用根级 `roles/{角色id}/`。内核集成方仍可直接使用 Rust 门面和六槽契约。

在 **构建** 上采用 **单核双态构建架构**：**同一套**编排语义与 DTO 契约（单核），构建期两档——**外核态**（低耦合、`PluginHost`）与 **宏核态**（Monolith 焊接）。二者经 `oclive init` 生成双 `[[bin]]`，**按构建产物选择**，非两套内核产品。

**运行时双核双态（Opt-in · 默认关）**：在**同一蓝图**内划分 **Stable 核**（固定六槽编排）与 **Experimental 核**（自定义 `pipeline.experimental`）。**机制已预埋，默认关闭**（`dual_core` Cargo feature；`dual_pipeline*` 默认不参与编译）。解冻条件见 [TECHNICAL_DEBT_INVENTORY.md](../../handoff/TECHNICAL_DEBT_INVENTORY.md) §冻结决定；现行设计见 [RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md](../rfc/RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md)，历史对齐记录不作为 truth。

**开放实验**是这套稳定契约可以支持的一种用法，不是对所有发行版的强制产品主轴（见 [VISION_OPEN_LAB.md](../roadmap/VISION_OPEN_LAB.md)）。

**蓝图扩展与资源协调（分阶段实现）**：蓝图只保存最小、命名空间化的能力声明；宿主已实现 Capability Registry、只读 `ExecutionPlan`，以及统一 Resource Coordinator（GPU/RAM/CPU、有限调度意图、公平准入、可逆自动抢占、真实 llama-server 档位与 owner-scoped 第三方注册入口）。通用契约已覆盖 `render` / `compute`，但具体 Live2D/3D runtime 仍由相应 Provider/发行版交付。扩展外壳不是第五类模块，资源协调也不是第七槽。详见 [RFC_BLUEPRINT_EXTENSION_AND_RESOURCE_COORDINATION.md](../rfc/RFC_BLUEPRINT_EXTENSION_AND_RESOURCE_COORDINATION.md)。

---

## 设施模块命名规范（规定）

| 术语 | 含义 |
|------|------|
| **设施模块** | **统称**：编排行内、**不进入六槽 `PluginBackends` 折叠**的内核延伸能力（含无编号设施与已登记子模块）。设施可以有自己的蓝图声明；例如 `complex_emotion` 可作为 `slot_registry.type`，但不会因此成为第七槽。**不存在**「专家模型设施模块」等中间大类。 |
| **`{专名}设施子模块`** | 在设施模块中**登记编号**（**第 N 设施子模块**）的项；全名 = **`{专名}` + `设施子模块`**；各专名**独立**，不得把「专家模型」当作整族前缀套在其它专名上。 |
| **专家模型**（专名） | 仅指 **专家模型设施子模块** 及其蓝图/实验核配置（条件触发子流程）；**不**包含复杂情感。 |
| **专家路由** | **专家模型设施子模块** 的默认实现：`blueprint/includes/expert_routing.json`（**与 `dual_core` 同 feature，默认不编译**）。 |

**扩展规则（设施子模块）**：新增已登记设施时，依次占用 **第 3、第 4… 设施子模块**，全名遵循 **`{新专名}设施子模块`**（须 RFC + 文档登记），**不**复用「专家模型」专名。

---

## 模块编号约定（规定）

当前完整参考运行时的扩展能力划分为 **四大类**；**不要**把四类全部等同为最小工具内核，也不要与「内核工厂配方层·实现层·代码层」混淆（后者见 [KERNEL_FACTORY_VISION.md](KERNEL_FACTORY_VISION.md)）。

| 大类 | 编号系列 | 与六槽折叠 `PluginBackends` 的关系 |
|------|----------|------------------------------------|
| **后端模块** | **第 1–6 模块**（固定，见下表） | 蓝图按六种稳定 `slot_registry.type` 声明，运行时折叠为六字段 |
| **设施模块** | **统称**；其中已登记项为 **第 N 设施子模块**（与 1–6 **独立序号**） | **不进入六槽折叠**；可有自有蓝图声明或由编排直接调用 |
| **独立通道能力增强模块** | **无模块号、无设施子模块号**；注册表 `id` 见 [RFC §2](../rfc/RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md#2-注册表-v1) | **不进入六槽折叠**；自有 Resolver + 锚点 / 独立 API |
| **后端模块插件模块** | **不使用「第 N 模块」编号** | 仅实现某 **第 K 后端模块**，沿用该槽的折叠字段 |

**扩展规则**

- 六个后端能力端口是当前稳定分类。新能力优先作为既有槽实现、设施、独立通道或宿主能力接入，**不**自动顺延为第 7、第 8 槽。改变六槽分类须有真实用例、RFC、Breaking 迁移与兼容窗口
- 新增 **`{专名}设施子模块`**（须 RFC + 文档登记）：第 3–4 已登记（立绘 · 视觉表现）；其后依次为 **第 5、第 6…**
- 新增 **独立通道能力增强模块**（须 RFC + 注册表）：登记 `id`、锚点或独立 API、可选 `provides`；**不** 占六槽、**不** 领设施子模块号（见 [RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md](../rfc/RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md)）
- 新增 **后端模块插件**（侧车 / 目录包）：写作 **「第 K 模块的 xxx 插件实现」**，**不** 占用第 7、第 8 模块号，也 **不** 占用设施子模块号或独立通道注册表位。

### 第 1–6 模块（后端模块，固定）

| 编号 | `slot_registry.type`（折叠后 `PluginBackends` 字段） | 职责 |
|------|---------------------------------------------------------|------|
| **第 1 模块** | `memory` | 记忆检索排序 |
| **第 2 模块** | `emotion` | 用户句情绪分析 |
| **第 3 模块** | `event` | 事件影响估计 |
| **第 4 模块** | `prompt` | Prompt 组装 |
| **第 5 模块** | `llm` | 主对话生成 |
| **第 6 模块** | `agent` | Agent / 工具编排 |

简称示例：**第 2 模块** = emotion 后端模块。内置 / ollama 实现仍属该模块的 **内置插头**，不是独立编号。

### 第 N 设施子模块（已登记 · 命名：`{专名}设施子模块`）

| 编号 | 规范全名 | 说明 |
|------|----------|------|
| **第 1 设施子模块** | **复杂情感设施子模块** | 回复后解析与跨轮 `narrative_hint`；第 2 模块输出仅是降级证据之一；详见 [§ 第 1 设施子模块](#第-1-设施子模块复杂情感设施子模块) |
| **第 2 设施子模块** | **专家模型设施子模块** | 条件触发专家子流程；默认实现为 **专家路由**；详见 [§ 第 2 设施子模块](#第-2-设施子模块专家模型设施子模块) |
| **第 3 设施子模块** | **立绘设施子模块** | `portrait_catalog` · **表现导演** AI 选 `visual_state_id`；详见 [§ 第 3 设施子模块](#第-3-设施子模块立绘设施子模块) |
| **第 4 设施子模块** | **视觉表现设施子模块** | `visual_state_id` → `performance_directive` · Live2D / 3D / 演算舞台；**无 AI 选图**；详见 [§ 第 4 设施子模块](#第-4-设施子模块视觉表现设施子模块) |

### 无编号设施模块（仍属设施模块统称）

`PluginHost`、`PersonalityEngine`、好感、`Repository`、`knowledge_index` 等：**设施模块**，**不**占用「第 N 设施子模块」序号；若未来需要可为编排/持久化类另立专名并登记编号。

### 后端模块插件模块（不编入第 N 模块）

**定义**：挂在 **某一第 K 模块（1≤K≤6）** 插座上的 **外挂实现**（Remote、directory、local 等）。**不与第 1–6 模块并列**，也 **不是** 第 7 模块。

| 说法示例 | 含义 |
|----------|------|
| 第 5 模块的目录插件实现 | `llm = directory`，`distros/chat-pro/plugins/<id>/` 子进程 |
| 第 2 模块的 Remote 侧车 | `emotion = remote`，共用 `OCLIVE_REMOTE_PLUGIN_URL` |
| ✗ 第 7 模块（目录插件） | **错误**——插件不单独占模块号 |

目录插件可选 **整壳 / ui_slots** UI，仍属 **该插件包**，不是新的「前端模块号」。

---

## 结构总图

```mermaid
flowchart TB
  ORCH["co_present 编排"]

  subgraph back["大类：后端模块（第 1–6 模块）"]
    M1["第1模块 memory"]
    M2["第2模块 emotion"]
    M3["第3模块 event"]
    M4["第4模块 prompt"]
    M5["第5模块 llm"]
    M6["第6模块 agent"]
  end

  subgraph plug["后端模块插件模块（无独立编号）"]
    P5["例：第5模块的 directory 插件"]
    P2["例：第2模块的 Remote 侧车"]
  end

  subgraph fac["大类：设施模块（统称）"]
    F0["无编号：PluginHost · 人格 · 好感 · DB …"]
    subgraph sub["第 N 设施子模块（{专名}设施子模块）"]
      F1["① 复杂情感设施子模块"]
      F2["② 专家模型设施子模块<br/>（专家路由）"]
      F3["③ 立绘设施子模块<br/>（表现导演）"]
      F4["④ 视觉表现设施子模块<br/>（角色舞台）"]
    end
  end

  ORCH --> M2
  M2 -.->|降级证据| F1
  M5 -->|post 解析 [EMO]| F1
  F1 -.->|持久化余韵供下一轮| M4
  ORCH -.->|experimental 且触发| F2
  F2 -.-> M4 & M5
  ORCH -.->|post_llm| F3
  F3 -.-> F4
  ORCH --> back
  M5 -.-> P5
  M2 -.-> P2
  ORCH --> F0
```

---

## 第 1 设施子模块（复杂情感设施子模块）

| 项 | 说明 |
|----|------|
| **职责** | 管理跨回合 `narrative_hint`：当前主 LLM 的有效 `[EMO]` 为权威，插件仅在标记缺失/无效时兜底；下一轮 Prompt 只接收“不含 hint 原文”的余韵连续性信号 |
| **编排位置** | `pre` 读取上一轮 hint → `middle` 仅为 Fast / 发行版 skip 计算本地确定性强度 → `post_llm` 解析并剥离 `[EMO]`、解析本轮结果并按契约持久化 |
| **与第 2 模块** | 第 2 模块分析**用户句**情绪，可作为降级证据；本设施处理**角色回复**的情绪标签与跨轮叙事余韵，不把用户情绪当成唯一结论 |
| **与专家模型** | **并列**的另一 `{专名}设施子模块`；**不**使用「专家模型」专名，**不**走 `expert_routing.json` |
| **现状** | 蓝图 `slot_registry` 可声明 `complex_emotion` 设施实例，经 `PluginHost` / `SlotRunner` last-wins 解析；`builtin` 启用跨轮 hint 读写并为 Fast 路径提供确定性强度，`remote` / `directory` 还可作 post 降级 provider；省略或 `none` 关闭 hint 读写 |
| **边界** | **`slot_registry` remote/directory 已可用**（`complex_emotion.resolve_turn`）；它不进入六键 `plugin_backends`，若未来改变六槽分类必须走 Breaking RFC |
| **Monolith** | 编译焊接键名 `complex_emotion`（**七焊接键**之一），≠ 宿主第六/第七槽 |

集成说明：[NARRATIVE_HINT_CONTRACT.md](../testing/NARRATIVE_HINT_CONTRACT.md)、[AGENTS.md](../../AGENTS.md)「复杂情感 `narrative_hint`」。

### 六宿主槽 vs Monolith 七焊接键

| 概念 | 个数 | 用途 |
|------|------|------|
| **后端模块（宿主槽）** | **6** | 蓝图六种稳定 `slot_registry.type` → 运行时 `PluginBackends` 折叠视图 → `PluginHost` |
| **Monolith `SLOT_IDS` 焊接键** | **7** | 编译期 `monolith.toml` / 演示管线；含 `complex_emotion` |
| **legacy 脚手架 `plugin_backends` 示例 JSON** | 6 + 扩展键 | `complex_emotion` 为 **旧工厂文档扩展键**，宿主的六槽 Serde 折叠会忽略；当前蓝图应声明独立 `complex_emotion` 实例 |

---

## 第 2 设施子模块（专家模型设施子模块）

| 项 | 说明 |
|----|------|
| **专名** | **专家模型**（仅指本子模块，见 [命名规范](#设施模块命名规范规定)） |
| **默认实现** | **专家路由**：`blueprint/includes/expert_routing.json`（`routes` · 触发条件 · `steps`） |
| **执行入口** | v3 蓝图 + **`dual_core`**：`pipeline.experimental` 中的 **`slot.expert.invoke`** → `execute_expert_route` |
| **步骤形态** | `slot.<registry_key>.<method>`（如 `slot.<llm>.generate`）及设施 action（`slot.personality.adjust`、`slot.prompt_enhance.apply`、`slot.memory.inject`、`slot.lora.apply` 等） |
| **与第 1 号** | 与 **复杂情感设施子模块** 同属 **设施模块** 下的并列子模块，**非** 包含关系 |
| **创作者 UI** | 插件工作台「专家模型设施」向导 / 架构图齿轮（产品简称；架构全名为 **专家模型设施子模块**） |

详见 [ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md) §2.6、[BLUEPRINT_FOLDER_LAYOUT.md](../../handoff/BLUEPRINT_FOLDER_LAYOUT.md)、[CREATOR_LEARNING_PATH.md](../role-pack/CREATOR_LEARNING_PATH.md) 高级配置。

---

## 第 3 设施子模块（立绘设施子模块）

| 项 | 说明 |
|----|------|
| **职责** | 从角色包 **`portrait_catalog`** 封闭列表选出 **`visual_state_id`**；默认 **表现导演**（AI + 复杂情感 hint） |
| **编排位置** | **post_llm**：合并/替换今日 `pick_portrait_emotion` 的「选状态」；**共景**可规则映射 `bot_emotion` |
| **与第 1 设施** | **消费** `narrative_hint`；**不**生成 hint |
| **与第 4 设施** | 输出 `visual_state_id`；**唯一**允许 LLM 选表现状态的设施 |
| **legacy** | 保留 `portrait_emotion` 七 tag；未启用 catalog 时行为与 v0.3 一致 |
| **RFC** | [RFC_PORTRAIT_FACILITY.md](../rfc/RFC_PORTRAIT_FACILITY.md) · [RFC_VISUAL_PRESENTATION_FACILITY.md](../rfc/RFC_VISUAL_PRESENTATION_FACILITY.md) |

---

## 第 4 设施子模块（视觉表现设施子模块）

| 项 | 说明 |
|----|------|
| **职责** | 将 **`visual_state_id`** 转为 **`performance_directive`**，供宿主渲染 adapter（PNG / Live2D / 3D /  procedural） |
| **编排位置** | post_llm 轻量 materialize；**帧循环 / GPU 在 UI**，不在 `process_message` |
| **AI** | **禁止**二次 LLM 选图 |
| **默认** | `config.json` → `visual_presentation.enabled: false` |
| **发行版** | `distro.oclive.toml` `[visual_presentation].mode` 已接入 HostProfile：VS Code off · Theater stage_full |
| **RFC** | [RFC_VISUAL_PRESENTATION_FACILITY.md](../rfc/RFC_VISUAL_PRESENTATION_FACILITY.md) |

---

## 单核双态构建架构

| 词 | 含义 |
|----|------|
| **单核** | 一套 `process_message` + PLUGIN_V1 契约；非 CPU 单核、非两套对话引擎 |
| **双态** | 外核态 / 宏核态 两档构建，长期并存 |
| **构建** | `oclive init` + `monolith.toml` + `cargo build`；双 `[[bin]]`；**非** 运行时热切换 |

| | **外核态** | **宏核态** |
|---|-----------|-----------|
| **实现名** | 低耦合、`PluginHost` | Monolith、`monolith.toml` |
| **六宿主槽** | v2/v3/v4 用蓝图 `slot_registry` 切换 backend；legacy `settings.json` 仅兼容/迁移 | 已焊槽静态调用；`weld_modules=[]` 且 `exclude=[]` → 六槽 + `complex_emotion` 焊接键全焊 |
| **桌面宿主默认** | **是** | 工厂脚手架；真 `process_message` 同构全焊热路径演进中（RFC §9） |

与内核工厂 **配方·实现·代码** 三层正交：双态只改变 **实现层解析方式**（动态 trait vs 静态焊），**代码层语义**不变。

---

## 共景主链（编号对照 · Stable 主路径）

Stable 主路径以 `process_message` 完成预取与可选 Agent 短路后，进入 `turn_pipeline` 的 **pre → co-present middle → 主 LLM → post**；**不是**按模块编号线性排列。编号仍对照 **第 1–6 模块** 与 **设施子模块**。

这张表描述的是**当前默认参考装配**，不是所有 OCLive 内核必须实现同样厚度的认知模型。当前代码要求共景健康路径至少有 `prompt + llm`；memory、emotion、event、agent 可以按各自 `none` / Noop 契约变薄。任何未来的强模型装配都应复用同一端口与权威边界，而不是复制第二套 `process_message`。

| 阶段 | 代码锚点 | 顺序 |
|------|----------|------|
| **预取** | `turn_prefetch.rs` | 用户身份、近期上下文 |
| **0 · Agent 短路**（可选） | `process_message.rs` | **LLM 之前** 可选短路（第 6 模块） |
| **pre_llm** | `turn_pipeline/pre.rs` | wave 1 并发取得上下文/人格、第 2 模块 `emotion.analyze`、有效模型、上一轮 hint、原始记忆与身份；随后时间/用户情绪/记忆强化 → **第 1 模块** `memory.rank_memories` → 关系快照/转移提示 |
| **co_present middle** | `turn_pipeline/co_present/run_middle.rs` | 规则事件初估 → Turn Thinking → Fast 本地情绪强度降级 / 知识 → 按策略可选调用第 3 模块 `event.estimate` 替换初估 → Event Ring 兼容桥与记忆提案 → 人格/关系预览 → 第 4 模块 `prompt.build`；Prompt 只消费上一轮 hint 的去内容连续性信号 |
| **主 LLM** | `turn_pipeline/post.rs` | 第 5 模块 `llm.generate` / stream → 原始 `reply` + 可选 `[EMO]` |
| **post_llm** | `turn_pipeline/post/post_llm.rs` | 剥离 `[EMO]` → 解析本轮角色回复情绪/复杂情感（有效主 LLM 标记权威，remote/directory 仅缺失/无效时兜底）→ 语义回复参与策略与情绪/关系/记忆/立绘状态计算和持久化 → 保存下一轮 hint → 单一回复后处理器 → 普通共景 `reply_mode` → 聊天写入与 DTO/视觉指令组装 |

**复杂情感跨轮不变量**：`pre` 只读取上一轮已存 hint；`build_prompt` 只据此输出去内容连续性信号，不注入原文，也不能看见本轮尚未产生的 hint。主 LLM 返回后，`post_llm` 才解析/剥离 `[EMO]`、按“有效标记 → remote/directory 兜底 → 保持/降级”解析本轮结果，并在设施启用时把 hint 持久化给下一轮。完整矩阵见 [NARRATIVE_HINT_CONTRACT.md](../testing/NARRATIVE_HINT_CONTRACT.md)。

**实验核（可选）**：匹配触发条件时，**第 2 设施子模块**（**专家模型设施子模块** / 专家路由）经 `slot.expert.invoke` 插入子步骤链，再汇合 Prompt / LLM 等（见 `dual_core` 文档）。Experimental 核 preview **尚未** 接线 relation transition / 复杂情感 hint。

---

## 独立通道能力增强模块（非六槽 · 非设施子模块编号）

与 [NAMING_CONVENTIONS.md](../NAMING_CONVENTIONS.md) §1.2、[RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md](../rfc/RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md) 对齐：**不进入六槽 `PluginBackends` 折叠**，**不**登记为「第 N 设施子模块」；经 **自有 Resolver** 接入 Stable 主链固定锚点，或经 **独立 API** 在 `process_message` 圈外运行。

**两类锚点**

| 模式 | 说明 | 注册表项 |
|------|------|----------|
| **主链侧钩（pre / post）** | 仍属对话回合，但不走六槽解析 | `user_identity`、`reply_post_process` |
| **圈外独立 API** | 单独 HTTP/Tauri 入口；消费 LLM 等设施，**不**插入 Stable 工序表 | `theater_director` |
| **输入侧宿主通道** | `send_message` **之前**；不进 `process_message` 钩子 | **`voice.asr`**（Windows 已交付 · Linux/macOS profile 占位） |

```mermaid
flowchart TB
  subgraph pre [turn_pipeline/pre · build_prompt 前]
    UI[user_identities/ · resolve_active_user_identity]
    PB[PromptBuilder.push_user_identity_section]
    UI --> PB
  end
  subgraph slots [第 1–6 模块 · process_message]
    LLM[llm.generate → raw reply]
  end
  subgraph builtin_post [post_llm · turn_pipeline/post/post_llm.rs]
    SEMANTIC[剥离协议标记 · semantic reply]
    STATE[情绪/记忆/好感/视觉等状态消费与持久化]
    SEMANTIC --> STATE
  end
  subgraph pp [reply_post_process]
    PROC[ReplyPostProcessor.process_reply]
    DISPLAY[reply_mode · display/transcript reply]
    CHAT[chat_storage append]
    OUT[SendMessageResponse.reply]
    PROC --> DISPLAY
    DISPLAY --> CHAT
    CHAT --> OUT
  end
  subgraph theater [theater_director · 圈外 API]
    API[generate_theater_scene / POST /theater/scene]
    SD[resolve_theater_director + 官方/目录插件]
    API --> SD
  end
  PB --> slots
  LLM --> SEMANTIC
  STATE --> PROC
```

### 注册表 v1（摘要）

| `id` | 规范名 | 配置落点 | 锚点 | 插件 `provides` |
|------|--------|----------|------|-----------------|
| **`user_identity`** | 用户身份 Prompt 模板 | 角色包 `user_identities/`；发行版 `[user_identity]` | pre → `build_prompt` | 无（角色包内容） |
| **`reply_post_process`** | 回复后处理 | 角色包 `config.json` → `reply_post_processor`；发行版 `[post_process].chain` | post_llm 内 semantic/state 消费后 → `process_reply` → chat append | `reply_post_process` |
| **`theater_director`** | 剧场场景导演 | `[theater].director_plugin`；fallback 内置 | `generate_theater_scene` | `theater_director`（**已交付**） |
| **`voice.asr`** | 语音识别输入 + 可选 TTS | 插件 `models/` + 设置；官方 `com.oclive.voice.asr` | 宿主 UI → `send_message` / `voice.speak` | `voice.asr`（**Windows 已交付**） |

**消歧**

- **用户身份** ≠ **角色身份**（`prompts/`、`core_personality.txt`）。
- **Reply Post-Processor** ≠ post-process chain profile 本身；≠ 第 4 模块 Prompt 槽；≠ Experimental 核 step。
- **Theater Scene Director** ≠ 第 4 模块 Prompt（`TheaterSceneRequest` ≠ `PromptInput`）；≠ 第 5 模块 LLM 插件（仅消费 `AppState::llm`）；默认 **不** 升格「第 5 设施子模块」。
- **Experimental 核**（`dual_core`）改的是 Stable **整圈工序**；独立通道是 Stable **固定钩子** 或 **圈外 API**。

**附录（宿主工具，非本注册表主表）**：编写器 `test_runner` 等见 RFC §2.1。

RFC 与验收：[RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md](../rfc/RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md) · 身份/后处理细节：[RFC_USER_IDENTITY_AND_REPLY_POST_PROCESSOR.md](../rfc/RFC_USER_IDENTITY_AND_REPLY_POST_PROCESSOR.md) · 剧场：[handoff/theater/DEVELOPMENT_ROADMAP.md](../../handoff/theater/DEVELOPMENT_ROADMAP.md)。

---

## 特点（摘要）

- **最小工具内核**：唯一编排与权威边界 + 六个稳定能力端口
- **参考运行时装配**：六槽实现 + 可选设施、Event Ring、持久化与宿主适配
- **后端模块插件模块**：按第 K 模块挂 Remote / 目录插件，**不占第 N 模块号**
- **发行版式交付**：OOCP、角色包、`oclive-cli` 工厂、Breaking 流程
- **单核双态**：标准二进制 + 可选 Monolith；`bench` 对比
- **权限**：目录插件 / MCP 高风险能力须用户授权
- **测试分层**：协议层（本仓 OOCP）、组件层（pack-editor）、插件层（编写器）

---

## 相关文档

| 主题 | 文档 |
|------|------|
| 六槽枚举与 JSON-RPC | [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md) |
| 蓝图 `slot_registry`、六槽折叠与 legacy `plugin_backends` | [SETTINGS_REFERENCE.md](../cli/SETTINGS_REFERENCE.md) |
| 专家路由文件与 includes | [ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md) · [BLUEPRINT_FOLDER_LAYOUT.md](../../handoff/BLUEPRINT_FOLDER_LAYOUT.md) |
| 插件扩展方式 | [CREATOR_PLUGIN_ARCHITECTURE.md](../plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md) |
| 蓝图扩展外壳 / Resource Coordinator | [RFC_BLUEPRINT_EXTENSION_AND_RESOURCE_COORDINATION.md](../rfc/RFC_BLUEPRINT_EXTENSION_AND_RESOURCE_COORDINATION.md) |
| 内核工厂与配方三层 | [KERNEL_FACTORY_VISION.md](KERNEL_FACTORY_VISION.md) |
| 总览图 | [KERNEL_AND_MODULES_ARCHITECTURE.md](KERNEL_AND_MODULES_ARCHITECTURE.md) |
| 纯净内核边界 | [PURE_KERNEL_BOUNDARY.md](PURE_KERNEL_BOUNDARY.md) |
| Monolith | [RFC_OCLIVE_MONOLITH_MODE.md](../rfc/RFC_OCLIVE_MONOLITH_MODE.md) |
| User Identity & Reply Post-Processor | [RFC_USER_IDENTITY_AND_REPLY_POST_PROCESSOR.md](../rfc/RFC_USER_IDENTITY_AND_REPLY_POST_PROCESSOR.md) · [ROLE_PACK_SPEC §1.1 / §9.7](../role-pack/ROLE_PACK_SPEC.md) |
