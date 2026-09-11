# A.I.Live · 最小工具内核与参考运行时总览图

**A.I.Live — 可插拔的角色动脉织机**（工程代号 **oclive**）。**内核集成方学习路径**：[KERNEL_INTEGRATOR_LEARNING_PATH.md](KERNEL_INTEGRATOR_LEARNING_PATH.md)

**架构叙述与编号**（第 1–6 模块、第 N 设施子模块、后端模块插件模块）：[OCLIVE_ARCHITECTURE_OVERVIEW.md](OCLIVE_ARCHITECTURE_OVERVIEW.md)（[English](../../creator-docs-en/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md)）。

本文分开表达**逻辑职责**与**当前参考运行时拓扑**。权责唯一入口为 [MODULE_MAP](../../handoff/MODULE_MAP_AND_HANDOFF.md#kernel-responsibilities)；现行 trait/DTO 和 richer 角色包分别见 [PLUGIN_V1](../plugin-and-architecture/PLUGIN_V1.md) 与 [ROLE_PACK_SPEC](../role-pack/ROLE_PACK_SPEC.md)，不自动等于未来最小 API。`pipeline.ocblueprint`、`slot_registry` 与 `groups` 是参考格式的装配配置，不是 persona + 视觉引用的最小角色条件。

---

## 1. 当前参考运行时总览图（Mermaid）

**读图约定**：这张图展示的是**完整参考运行时拓扑**，不是最小 core 的物理 crate 边界。中间为回合编排与解析；上下两行为六宿主槽门面；最上为用户与进程边界；最下为持久化与外协实现；最底为脚手架 / 编译期路径。**v1（已废弃）** 的 `settings.json` → `plugin_backends` 仅作迁移对照，见 [V1_TO_V2_MIGRATION.md](../role-pack/V1_TO_V2_MIGRATION.md)。

**历史静态示意图**（保留供参考运行时拓扑学习）：其中“内核”使用旧的完整运行时含义，不作为当前小 Kernel 边界或下方 Mermaid 逐字同步的证明。演示当前分责请使用 §2 与 MODULE_MAP。

![以内核为中心的总览：边界、六槽、内核、外协、脚手架](../assets/oclive-kernel-centric-architecture.png)

```mermaid
flowchart TB
  subgraph boundary["用户与进程边界"]
    direction LR
    UI["Vue 前端"]
    TAURI["Tauri invoke"]
    API["HTTP --api / kernel_server"]
    OOCP["OOCP 对照 · HTTP 黑盒"]
  end

  subgraph six_top["可替换六槽 · slot_registry 折叠（上）"]
    direction LR
    M["memory<br/>builtin · v2 · remote · directory · local"]
    EM["emotion<br/>builtin · v2 · remote · directory"]
    EV["event<br/>builtin · v2 · remote · directory"]
  end

  K(("完整参考运行时 / 参考 Host<br/>chat_engine · process_message<br/>PluginHost::resolve_for_role<br/>DTO: oclive_kernel_types"))

  subgraph six_bot["可替换六槽 · slot_registry 折叠（下）"]
    direction LR
    PR["prompt<br/>builtin · v2 · remote · directory"]
    LL["llm<br/>ollama · remote · directory"]
    AG["agent<br/>builtin ReAct · MCP · remote · directory"]
  end

  subgraph infra["持久化与外协"]
    direction LR
    REPO["Repository / SQLite"]
    RMT["Remote 侧车<br/>JSON-RPC · OCLIVE_REMOTE_*"]
    DIR["Directory 插件<br/>distros/chat-pro/plugins/ 子进程"]
    MCP["MCP 配置<br/>app_data/mcp-servers/*.json"]
    SESS["会话级槽位覆盖<br/>set_session_slot_override"]
  end

  subgraph toolchain["脚手架 / 编译期（可选）"]
    direction LR
    OCLI["oclive-cli init"]
    BUILD["oclive build / bench"]
    MONO["monolith.toml + feature monolith"]
  end

  boundary --> K
  six_top --> K
  six_bot --> K
  K --> REPO
  K --> RMT
  K --> DIR
  K --> MCP
  SESS -.->|合并有效后端快照| K
  toolchain -.->|生成焊接产物；不参与 load_role| K
```

---

## 2. 最小概念核心（六端口与内核）

下图只解释已确认的逻辑分责，不设计新接口或物理拆分。Host 负责具体运行；小 Kernel 定义六槽契约与必要合法性边界，不规定固定六阶段。框内槽名表示契约，不表示具体实现都驻留在小 Kernel；共景 `prompt + llm` 健康门槛是当前参考 Host 策略。

```mermaid
flowchart TB
  H["发行版 Host<br/>输入准备 / 能力绑定 / 调度 / 领域应用"]
  subgraph K["小 Kernel · 必要公共合法性边界"]
    C["六槽能力契约<br/>memory / emotion / event / prompt / llm / agent"]
  end
  H -->|按契约交互，不表示固定执行顺序| C
  C -->|契约范围内的执行结果，不代表产品整体成功| H
```

Event Ring、具体槽位实现、Repository / SQLite、资源协调、HTTP/Tauri 和发行版都在这张最小图之外；它们可以围绕端口与受控 hook 组成完整工具。当前源码尚未把该最小边界独立成一个可单独编译的 crate，见 `K-CORE-BOUNDARY-01`。

---

## 3. 图中已标出的「近期更新 / 应对齐」能力

| 能力 | 说明 |
|------|------|
| **第 6 模块 `agent`** | `slot_registry` 中 `type: agent` 实例；`BuiltinReActAgent`；MCP 见 `AGENTS.md`。 |
| **MCP** | Agent 路径上工具发现 / 调用；配置在应用数据目录下 `mcp-servers`。 |
| **`memory = local`** | `_local_plugins` 与桥接契约见 [LOCAL_PLUGIN_BRIDGE_SPEC.md](../plugin-and-architecture/LOCAL_PLUGIN_BRIDGE_SPEC.md)。 |
| **会话级槽位覆盖** | `set_session_slot_override`；`get_role_info` / `load_role` 返回有效后端快照。 |
| **`oclive-cli` + Monolith** | `init` / `build` / `bench`；`monolith.toml` 仅编译期；与角色包蓝图正交。 |
| **无头 / CI** | `kernel_server`、`--api`、OOCP 对照套件等与桌面共用 domain 契约。 |

若某能力未出现在你维护的 fork 上，以该分支 **实际代码与迁移** 为准，再回头改本节表格与 Mermaid 标签。

---

## 4. 相关链接

- 六槽数据流（自上而下）：[PLUGIN_V1.md § 架构图与 send_message 顺序](../plugin-and-architecture/PLUGIN_V1.md)
- **纯净内核边界与嵌入式范围**：[PURE_KERNEL_BOUNDARY.md](PURE_KERNEL_BOUNDARY.md)
- 创作者向三种扩展方式：[CREATOR_PLUGIN_ARCHITECTURE.md](../plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md)
- 脚手架与 Monolith：[OCLIVE_CLI_GUIDE.md](../cli/OCLIVE_CLI_GUIDE.md) · [RFC_OCLIVE_MONOLITH_MODE.md](../rfc/RFC_OCLIVE_MONOLITH_MODE.md)

---

[English](../../creator-docs-en/getting-started/KERNEL_AND_MODULES_ARCHITECTURE.md)
