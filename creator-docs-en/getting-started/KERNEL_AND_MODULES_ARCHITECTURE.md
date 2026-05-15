# Kernel-centric module architecture (overview diagram)

This page uses a **kernel-in-the-center** diagram to align with the current **main** branch. Authoritative detail remains in **[PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md)** (six-slot contract), **[EXTENSION_POINTS.md](../../creator-docs/plugin-and-architecture/EXTENSION_POINTS.md)** (traits & paths), and **[RFC_OCLIVE_MONOLITH_MODE.md](../../creator-docs/rfc/RFC_OCLIVE_MONOLITH_MODE.md)** (Monolith compile-time mode).

---

## 1. Overview diagram (Mermaid)

**How to read:** center = **dialogue kernel** (orchestration + resolution); top/bottom rows = **`plugin_backends` host slots** (facade traits); top band = **user & process boundary**; bottom = **persistence & external implementations**; lowest band = **scaffolding / compile-time** (orthogonal to runtime `load_role`).

Static asset (same structure as Mermaid, for slides/print):

![Kernel-centric overview](../../creator-docs/assets/oclive-kernel-centric-architecture.png)

```mermaid
flowchart TB
  subgraph boundary["User & process boundary"]
    direction LR
    UI["Vue frontend"]
    TAURI["Tauri invoke"]
    API["HTTP --api / kernel_server"]
    OOCP["OOCP suite · HTTP black-box tests"]
  end

  subgraph six_top["Swappable six slots · plugin_backends (top)"]
    direction LR
    M["memory<br/>builtin · v2 · remote · directory · local"]
    EM["emotion<br/>builtin · v2 · remote · directory"]
    EV["event<br/>builtin · v2 · remote · directory"]
  end

  K(("Dialogue kernel<br/>chat_engine · process_message<br/>PluginHost::resolve_for_role"))

  subgraph six_bot["Swappable six slots · plugin_backends (bottom)"]
    direction LR
    PR["prompt<br/>builtin · v2 · remote · directory"]
    LL["llm<br/>ollama · remote · directory"]
    AG["agent<br/>builtin ReAct · MCP · remote · directory"]
  end

  subgraph infra["Persistence & collaborators"]
    direction LR
    REPO["Repository / SQLite"]
    RMT["Remote sidecar<br/>JSON-RPC · OCLIVE_REMOTE_*"]
    DIR["Directory plugins<br/>plugins/ child processes"]
    MCP["MCP config<br/>app_data/mcp-servers/*.json"]
    SESS["Session backend overrides<br/>set_session_plugin_backend"]
  end

  subgraph toolchain["Scaffolding / compile-time (optional)"]
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
  SESS -.->|merged effective backend snapshot| K
  toolchain -.->|generated weld artifacts; not part of load_role| K
```

> **Note:** the desktop HTTP API used by the OOCP suite is **HTTP only** today; a future WebSocket OOCP surface would extend the host before the diagram label “WebSocket” becomes accurate.

---

## 2. Star diagram (six slots → kernel)

Use this with PLUGIN_V1’s **linear `send_message` sequence** (this diagram is **topology**, not per-turn order).

```mermaid
flowchart TB
  M[memory] --> K((Dialogue kernel))
  EM[emotion] --> K
  EV[event] --> K
  PR[prompt] --> K
  LL[llm] --> K
  AG[agent] --> K
```

---

## 3. Recently highlighted capabilities

| Capability | Notes |
|------------|------|
| **Sixth slot `agent`** | `plugin_backends.agent`; `BuiltinReActAgent`; MCP scan path — see root `AGENTS.md`. |
| **MCP** | Tool discovery / invocation on the agent path; config under app data `mcp-servers`. |
| **`memory = local`** | `_local_plugins` bridge — [LOCAL_PLUGIN_BRIDGE_SPEC.md](../../creator-docs/plugin-and-architecture/LOCAL_PLUGIN_BRIDGE_SPEC.md). |
| **Session overrides** | `set_session_plugin_backend`; `get_role_info` / `load_role` expose `plugin_backends_effective*` snapshots. |
| **`oclive-cli` + Monolith** | `init` / `build` / `bench`; `monolith.toml` is compile-time only; orthogonal to `settings.json`. |
| **Headless / CI** | `kernel_server`, `--api`, OOCP suite share domain contracts with the desktop build. |

If your fork differs, follow **that branch’s code and migrations**, then update this page.

---

## 4. Related links

- Six-slot pipeline (top-down): [PLUGIN_V1.md § Architecture & send_message order](../plugin-and-architecture/PLUGIN_V1.md)
- **Pure kernel boundary & embedded scope**: [PURE_KERNEL_BOUNDARY.md](PURE_KERNEL_BOUNDARY.md)
- Three extension styles for creators: [CREATOR_PLUGIN_ARCHITECTURE.md](../../creator-docs/plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md)
- CLI & Monolith: [OCLIVE_CLI_GUIDE.md](../../creator-docs/cli/OCLIVE_CLI_GUIDE.md) · [RFC_OCLIVE_MONOLITH_MODE.md](../../creator-docs/rfc/RFC_OCLIVE_MONOLITH_MODE.md)

---

[中文](../../creator-docs/getting-started/KERNEL_AND_MODULES_ARCHITECTURE.md)
