# A.I.Live · 最小工具内核与当前嵌入运行时边界

本文消除“纯净内核”的两种旧含义：**最小概念核心**与当前已经可嵌入的**完整参考运行时**不是同一层。模块分层见 [OCLIVE_ARCHITECTURE_OVERVIEW.md](OCLIVE_ARCHITECTURE_OVERVIEW.md)；总览图见 [KERNEL_AND_MODULES_ARCHITECTURE.md](KERNEL_AND_MODULES_ARCHITECTURE.md)；实施阶段见 [KERNEL_IMPLEMENTATION_PLAN.md](KERNEL_IMPLEMENTATION_PLAN.md)。

[English](../../creator-docs-en/getting-started/PURE_KERNEL_BOUNDARY.md)

---

## 1. 两层边界

### 1.1 最小概念核心

OCLive 真正要长期守住的最小核心是：

```text
唯一回合/生命周期编排
  + 能力调用与合并规则
  + 权威状态提交、错误语义与故障隔离
  + memory / emotion / event / prompt / llm / agent 六个稳定端口
```

六槽是能力接口，不是六个平级决策内核。具体记忆算法、情绪模型、Prompt 模板、LLM 厂商、Event Ring、SQLite、HTTP、Tauri 和角色包工具都可以围绕它装配，但不决定 OCLive 是否成立。当前共景运行路径仍要求 `prompt + llm` 通过健康检查；其余槽位可按已定义的 `none` / Noop 语义变薄。

### 1.2 当前完整嵌入运行时

主仓今天交付的 `oclive_kernel_host::OcliveKernel` 是**受支持的 Rust 源码级门面**，它复用唯一 `process_message`，并提供角色加载、完整/流式回合、SQLite、插件、Event Ring 与关闭生命周期。它与 UI、具体硬件 BSP 和单一模型品牌解耦，因此可以被嵌入；但物理代码仍包含 HTTP 实现及大量默认设施，不能把整个 host crate 或五个 kernel crate 的总行数称为“最小内核大小”。

| 层 | 当前代码锚点 | 边界 |
|----|--------------|------|
| **最小核心骨架** | `oclive_kernel_contracts` 六个 trait、`process_message` / turn pipeline、核心 DTO/错误 | 概念已收敛；尚未独立成可单独编译的 crate |
| **完整嵌入运行时门面** | `oclive_kernel_host::OcliveKernel` · `role_kernel.rs` | 已可用；包含持久化、Event Ring、HTTP 依赖和默认装配 |
| **传输与 UI 宿主** | `oclive-kernel-server`、Tauri、Vue、VS Code | 不属于内核本体；必须委托同一回合入口 |

物理拆薄由 [`K-CORE-BOUNDARY-01`](../../handoff/TECHNICAL_DEBT_INVENTORY.md) 跟踪。在它完成前，“最小内核”是职责边界，不是已经存在的独立发布包。

---

## 2. 最小工具内核明确不包含什么

- **Vue 前端**、Tauri `invoke`、窗口与主题。
- **具体 LLM 厂商 SDK**（应落在 `llm` 槽：ollama / remote / directory）。
- **板级 BSP**（麦克风驱动、电机、RTOS）；通过 **目录插件 / 侧车 / MCP** 接入，内核只消费契约化结果。
- **创作者文档 UI**、插件市场站点、启动器安装体验。
- **Prompt 正文语言**（角色包与模型侧内容语言）；与**界面 i18n** 分离。
- **Event Ring 执行设施、SQLite Repository、资源协调和具体六槽实现**；它们属于参考运行时装配，通过端口或受控锚点协作。

---

## 3. 角色包交付单元

角色数据与槽位策略都是可携带输入，但它们不是同一层 contract。发行版可以把两者放在同一个产品包中；进入内核前必须由适配层拆成最小角色定义与宿主能力绑定。

| 组成部分 | 说明 |
|----------|------|
| **内核最小角色定义（逻辑校验与本地文件读取已实现，运行接入待做）** | 作者只需非空 persona prompt + 至少 1 个视觉资产。共享 DTO 只校验正文与资产引用非空；可选 native 适配入口另做本地路径/普通文件检查和有界字节读取，不证明媒体有效或角色可加载。预算属于调用方，不增加角色字段；七图情绪集、关系模型与技术封装边界见 [ROLE_PACK_BOUNDARY.md](../../handoff/ROLE_PACK_BOUNDARY.md) §0.1–0.3 |
| **参考宿主组合目录（当前实现）** | **`pipeline.ocblueprint`** v2/v3/v4 把 `meta` 与 `slot_registry` / `runtime_config` 放在同一文件，再从同目录加载 `core_personality.txt`、场景、知识及产品扩展；这是参考实现输入，不是 kernel canonical schema（见 [ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md)） |
| **有效后端** | 由宿主蓝图 `slot_registry`、**`set_session_slot_override`** 会话覆盖和环境变量合成；不属于最小角色定义（见 [SETTINGS_REFERENCE.md](../cli/SETTINGS_REFERENCE.md)） |
| **当前参考运行时的关系与记忆** | `role_runtime`、长期记忆等经 Repository 读写；具体策略由相应能力实现，不构成最小角色内容要求 |

**机器人场景**：设备可以只替换角色数据，由设备宿主独立选择六槽装配；最小角色仍携带至少一个视觉资产，无显示设备时可不渲染，不要求桌面版七图目录、模型或蓝图策略。

**RobotSoulPack** 是现有参考宿主的机器人/嵌入式 profile，对应 **`oclive pack validate --profile robot-soul`**，其校验通过不代表符合新最小 contract；字段与示例见 [ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md)、[examples/robot-soul-minimal](../../examples/robot-soul-minimal/README.md)。

---

## 4. 情感陪伴在架构中的位置

当前默认陪伴装配由**后端模块 + 设施模块**协作完成，而非单一“情感模块”黑盒（分层见 [OCLIVE_ARCHITECTURE_OVERVIEW.md](OCLIVE_ARCHITECTURE_OVERVIEW.md)）：

- **emotion 后端模块** + **复杂情感设施子模块**（第 1 设施子模块）：用户句情绪与跨回合叙事 `narrative_hint`。
- **专家模型设施子模块**（第 2 设施子模块）：条件触发的专家子流程（专家路由）；与复杂情感**并列**，见 [OCLIVE_ARCHITECTURE_OVERVIEW.md](OCLIVE_ARCHITECTURE_OVERVIEW.md)。
- **立绘设施子模块**（第 3 设施子模块 · 草案）：`portrait_catalog`、表现导演 AI 选 `visual_state_id`；见 [RFC_PORTRAIT_FACILITY.md](../rfc/RFC_PORTRAIT_FACILITY.md)。
- **视觉表现设施子模块**（第 4 设施子模块）：`performance_directive` 与发行版 gating 已交付，Live2D / 3D / 演算 adapter 部分交付；见 [RFC_VISUAL_PRESENTATION_FACILITY.md](../rfc/RFC_VISUAL_PRESENTATION_FACILITY.md)。
- **memory / event**：关系与事件对后续回合的影响。
- **prompt / llm**：语言表达与 persona 注入。
- **agent**（可选）：工具与外部世界（MCP、目录插件）。

内核保证 **调用顺序、端口、错误与状态提交边界**；陪伴“好不好”由模型、槽实现、设施和角色包内容共同决定。强模型装配可以省略部分显式辅助；小模型装配可以使用更厚的候选生成与 Prompt 编译，但辅助信号不得冒充唯一语义真值。

---

## 5. 当前完整运行时的部署形态

| 形态 | 用途 | Monolith | 说明 |
|------|------|----------|------|
| **桌面宿主** | 玩家 / 创作者 | 可选（独立工程） | Tauri + Vue + 同一 domain |
| **无头 HTTP** | 网关、机器人中控、CI 联调 | **Monolith 仅** `oclive-cli` 生成的 **kernel_server** 工程可选 | 主仓 **`oclive-kernel-server`** 与 **`oclivenewnew-tauri --api`** 等价（`http_api`）；默认端口 **8420**（`OCLIVE_API_PORT`） |
| **嵌入式 `library`** | 进程内嵌、自有 `main` | **不适用** Monolith | 链接 `oclive_kernel_host` + contracts/runtime/types，由 **`OcliveKernel`** 提供角色加载、完整回合、流式回复、Event Ring 与持久化；`oclive-cli init --project-type library --kernel-source` 可直接生成（见 [KERNEL_PLATFORM_DEVELOPER_PATH.md](KERNEL_PLATFORM_DEVELOPER_PATH.md) §5） |
| **HTTP `--api`** | 联调、CI、编写器试聊 | N/A | 当前主仓过渡方案，见 [headless-kernel-minimal](../../examples/headless-kernel-minimal/README.md) |

**可拆可焊**：开发期槽位可替换（松耦合）；量产可选 Monolith 将选定 builtin 焊进单一二进制（紧耦合）。二者与角色运行配置（当前蓝图 `slot_registry` / `runtime_config`，以及 legacy `settings.json`）**正交**。

这里的“稳定接口”指当前 Rust crate 的**受支持源码级门面**，不是稳定 C ABI。`AppState`、HTTP 路由和 Tauri command 都是内部装配/传输细节，集成方不应绕过 `OcliveKernel` 直接拼装第二条回合链。门面目前位于 `oclive_kernel_host`；调用时不需要启动 HTTP 或 Tauri，但该 crate 仍包含 HTTP 实现与相关依赖，后续可继续做依赖瘦身。

---

## 6. 嵌入式诚实范围表

### 在范围内（当前架构目标）

- Linux 用户态、**数百 MB 级 RAM** 以上的设备或网关。
- **Rust 异步**、HTTP/JSON-RPC、子进程目录插件、SQLite 持久化。
- 不同发行版通过适配层产出同一最小角色定义；六槽 provider 由各宿主另行绑定。当前完整参考运行时仍接受组合蓝图并折叠为 `PluginBackends`，这是过渡实现而非跨发行版格式要求。
- 当前桌面开发机已用真实文件 SQLite、角色加载、普通/流式回合、Event Ring 注册与主动回合完成进程内集成测试。
- 侧车 LLM（`remote`）、本机 Ollama（`ollama`）、目录插件扩展硬件。

### 明确不在范围内（勿过度承诺）

- **硬实时**、**MCU / KB 级 RAM**、无 OS 裸机。
- 内核内建**音视频编解码栈**（应走插件或设备侧服务）。
- 多租户云端**隔离与计费**（未作为内核一等公民；B2 可单独立项）。
- 尚不能把当前代码级验证等同于 Linux/ARM 真机、长期硬件 soak 或明确资源预算证明；状态见 `V-EMBED-01`。

---

## 7. 相关链接

- 实施计划：[KERNEL_IMPLEMENTATION_PLAN.md](KERNEL_IMPLEMENTATION_PLAN.md)
- 内核集成方单线：[KERNEL_PLATFORM_DEVELOPER_PATH.md](KERNEL_PLATFORM_DEVELOPER_PATH.md)
- 当前工程债务：[TECHNICAL_DEBT_INVENTORY.md](../../handoff/TECHNICAL_DEBT_INVENTORY.md) 与 [PRODUCT_LINE_TASK_BUCKETS.md](../../handoff/PRODUCT_LINE_TASK_BUCKETS.md)
- 校企玩偶交付：与主仓并列的 **oclive doll core** 目录（settings 模板、硬件插件示例、打包说明）；契约以本仓为准。
- Monolith RFC：[RFC_OCLIVE_MONOLITH_MODE.md](../rfc/RFC_OCLIVE_MONOLITH_MODE.md)
