# 以内核为中心的模块架构（总览图）

本文用 **一张「内核居中、模块环绕」的示意图** 对齐当前主仓能力；细节仍以 **[PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md)**（六槽契约）、**[EXTENSION_POINTS.md](../plugin-and-architecture/EXTENSION_POINTS.md)**（trait 与路径）、**[RFC_OCLIVE_MONOLITH_MODE.md](../rfc/RFC_OCLIVE_MONOLITH_MODE.md)**（编译期 Monolith）为准。

---

## 1. 总览图（Mermaid）

**读图约定**：中间为 **对话内核**（编排 + 解析）；上下两行为 **`plugin_backends` 六宿主槽**（门面 trait）；最上为 **用户与进程边界**；最下为 **持久化与外协实现**；最底为 **脚手架 / 编译期路径**（与角色包运行时正交）。

与下方 Mermaid 同结构的 **静态示意图**（便于打印或放进 PPT）：

![以内核为中心的总览：边界、六槽、内核、外协、脚手架](../assets/oclive-kernel-centric-architecture.png)

```mermaid
flowchart TB
  subgraph boundary["用户与进程边界"]
    direction LR
    UI["Vue 前端"]
    TAURI["Tauri invoke"]
    API["HTTP --api / kernel_server"]
    OOCP["OOCP 对照 · WebSocket"]
  end

  subgraph six_top["可替换六槽 · plugin_backends（上）"]
    direction LR
    M["memory<br/>builtin · v2 · remote · directory · local"]
    EM["emotion<br/>builtin · v2 · remote · directory"]
    EV["event<br/>builtin · v2 · remote · directory"]
  end

  K(("对话内核<br/>chat_engine · process_message<br/>PluginHost::resolve_for_role<br/>DTO: oclive_kernel_runtime"))

  subgraph six_bot["可替换六槽 · plugin_backends（下）"]
    direction LR
    PR["prompt<br/>builtin · v2 · remote · directory"]
    LL["llm<br/>ollama · remote · directory"]
    AG["agent<br/>builtin ReAct · MCP · remote · directory"]
  end

  subgraph infra["持久化与外协"]
    direction LR
    REPO["Repository / SQLite"]
    RMT["Remote 侧车<br/>JSON-RPC · OCLIVE_REMOTE_*"]
    DIR["Directory 插件<br/>plugins/ 子进程"]
    MCP["MCP 配置<br/>app_data/mcp-servers/*.json"]
    SESS["会话级后端覆盖<br/>set_session_plugin_backend"]
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

## 2. 星型简图（仅六槽与内核）

便于和 **PLUGIN_V1** 中的「线性数据流图」对照：下图强调 **六槽均汇入同一内核**，不表示单轮调用时序（时序见 PLUGIN_V1「`send_message` 编排顺序」）。

```mermaid
flowchart TB
  M[memory] --> K((对话内核))
  EM[emotion] --> K
  EV[event] --> K
  PR[prompt] --> K
  LL[llm] --> K
  AG[agent] --> K
```

---

## 3. 图中已标出的「近期更新 / 应对齐」能力

| 能力 | 说明 |
|------|------|
| **第六槽 `agent`** | `plugin_backends.agent`；`BuiltinReActAgent`；MCP 扫描目录见仓库根 `AGENTS.md`。 |
| **MCP** | Agent 路径上工具发现 / 调用；配置在应用数据目录下 `mcp-servers`。 |
| **`memory = local`** | `_local_plugins` 与桥接契约见 [LOCAL_PLUGIN_BRIDGE_SPEC.md](../plugin-and-architecture/LOCAL_PLUGIN_BRIDGE_SPEC.md)。 |
| **会话级后端覆盖** | `set_session_plugin_backend`；`get_role_info` / `load_role` 返回 `plugin_backends_effective*` 快照。 |
| **`oclive-cli` + Monolith** | `init` / `build` / `bench`；`monolith.toml` 仅编译期；与 `settings.json` 正交。 |
| **无头 / CI** | `kernel_server`、`--api`、OOCP 对照套件等与桌面共用 domain 契约。 |

若某能力未出现在你维护的 fork 上，以该分支 **实际代码与迁移** 为准，再回头改本节表格与 Mermaid 标签。

---

## 4. 相关链接

- 六槽数据流（自上而下）：[PLUGIN_V1.md § 架构图与 send_message 顺序](../plugin-and-architecture/PLUGIN_V1.md)
- 创作者向三种扩展方式：[CREATOR_PLUGIN_ARCHITECTURE.md](../plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md)
- 脚手架与 Monolith：[OCLIVE_CLI_GUIDE.md](../cli/OCLIVE_CLI_GUIDE.md) · [RFC_OCLIVE_MONOLITH_MODE.md](../rfc/RFC_OCLIVE_MONOLITH_MODE.md)
