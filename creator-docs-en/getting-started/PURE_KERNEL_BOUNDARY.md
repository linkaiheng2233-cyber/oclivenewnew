# Pure kernel: boundary, soul, and embedded scope

This page defines what oclive means by a **pure kernel**, and how it aligns with the desktop host, headless service, embedded library, and robot **“soul”** delivery. See [KERNEL_AND_MODULES_ARCHITECTURE.md](KERNEL_AND_MODULES_ARCHITECTURE.md) for the diagram and [KERNEL_IMPLEMENTATION_PLAN.md](KERNEL_IMPLEMENTATION_PLAN.md) for phases.

[中文](../../creator-docs/getting-started/PURE_KERNEL_BOUNDARY.md)

---

## 1. What the pure kernel is

The **pure kernel** is the runtime layer that is **independent of UI**, **independent of board BSP**, and **independent of any single proprietary model vendor**. It is responsible for:

| Responsibility | Anchor in main repo |
|----------------|---------------------|
| **Turn orchestration** | `src-tauri/src/domain/chat_engine/` · `process_message` |
| **Slot resolution** | `PluginHost::resolve_for_role` · `plugin_backends` |
| **Contracts & persistence shape** | `models/dto.rs` · `migrations/001_init.sql` · `oclive_validation` |
| **Headless entry (transition)** | `http_api` · `oclivenewnew-tauri --api` |

```text
User/device boundary   →  Vue / hardware drivers / sidecar processes (not “kernel”)
Pure kernel            →  process_message + PluginHost + Repository contracts
Slot implementations   →  builtin / remote / directory / local / ollama …
Soul data (customizable)→  role pack manifest + settings + personality/knowledge files
```

This is **not** the Linux kernel and **not** the full Tauri desktop app.

---

## 2. What the pure kernel explicitly excludes

- **Vue frontend**, Tauri `invoke`, windows, and themes.
- **Vendor-specific LLM SDKs** (belong in the `llm` slot: ollama / remote / directory).
- **Board BSP** (mic drivers, motors, RTOS); integrate via **directory plugins / sidecars / MCP**; the kernel consumes contract-shaped results only.
- **Creator doc UI**, plugin market site, launcher install UX.
- **Prompt body language** (pack/model content language); separate from **UI i18n**.

---

## 3. “Custom soul” delivery unit

Externally: **soul = versioned data + configurable slot policy**, loaded at runtime by the kernel—not hard-coded in orchestration.

| Part | Description |
|------|-------------|
| **Role pack** | `manifest.json` · `settings.json` · `core_personality.txt` · scenes/knowledge ([ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md)) |
| **Effective backends** | Pack default + session override + env merge ([SETTINGS_REFERENCE.md](../cli/SETTINGS_REFERENCE.md)) |
| **Relation & memory** | `role_runtime`, long-term memory via Repository; `memory` slot implements policy |

**Robot scenario**: swap soul pack and light `settings` tweaks without changing kernel version (within `min_runtime_version`).

Working name **RobotSoulPack** will align with `pack validate` profile in phase K3; until then use full role pack spec.

---

## 4. Where companion emotion sits

Companion behavior is **multi-slot collaboration**, not one black-box “emotion module”:

- **emotion / complex_emotion**: in-turn signals.
- **memory / event**: relationship and event impact on later turns.
- **prompt / llm**: language and persona injection.
- **agent** (optional): tools and external world (MCP, directory plugins).

The kernel guarantees **call order and DTOs**; quality comes from slots and pack content.

---

## 5. Deployment shapes and “one steel plate”

| Shape | Use | Monolith | Notes |
|-------|-----|----------|-------|
| **Desktop host** | Players / creators | Optional (separate project) | Tauri + Vue + same domain |
| **Headless `kernel_server`** | Gateway, robot brain process | **Supported** | `oclive-cli init` — [OCLIVE_CLI_GUIDE.md](../cli/OCLIVE_CLI_GUIDE.md) |
| **Embedded `library`** | In-process embed | **Not supported** | Static-link runtime lib (K2/K4) |
| **HTTP `--api`** | Dev, CI, editor try-chat | N/A | Transition — [headless-kernel-minimal](../../examples/headless-kernel-minimal/README.md) |

**Detachable or welded**: dev-time swappable slots; production optional Monolith weld into one binary—orthogonal to `settings.json`.

---

## 6. Honest embedded scope

### In scope (current architecture target)

- Linux user space, devices/gateways with **hundreds of MB RAM** and up.
- **Rust async**, HTTP/JSON-RPC, directory plugin subprocesses, SQLite persistence.
- **Same role packs** and `plugin_backends` shape as desktop.
- Sidecar LLM (`remote`), local Ollama (`ollama`), hardware via directory plugins.

### Explicitly out of scope (do not over-promise)

- **Hard real-time**, **MCU / KB-scale RAM**, bare-metal without OS.
- **Built-in A/V codec stack** in kernel (use plugins or device services).
- Multi-tenant cloud **isolation and billing** as first-class kernel features (phase B2 if needed).

---

## 7. Related links

- Implementation plan: [KERNEL_IMPLEMENTATION_PLAN.md](KERNEL_IMPLEMENTATION_PLAN.md)
- Gap checklist: [PRODUCT_AND_KERNEL_GAP_CHECKLIST.md](../../handoff/PRODUCT_AND_KERNEL_GAP_CHECKLIST.md) §B
- Doll / hardware delivery pack: **oclive doll core** sibling directory (settings templates, hardware examples); contracts authoritative in this repo.
- Monolith RFC: [RFC_OCLIVE_MONOLITH_MODE.md](../rfc/RFC_OCLIVE_MONOLITH_MODE.md)
