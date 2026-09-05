# A.I.Live · Minimal tool kernel and current embedded-runtime boundary

This page separates two meanings that older documents called “pure kernel”: the **minimal conceptual core** and today's embeddable **complete reference runtime** are not the same layer. Module taxonomy: [OCLIVE_ARCHITECTURE_OVERVIEW.md](OCLIVE_ARCHITECTURE_OVERVIEW.md). Diagram: [KERNEL_AND_MODULES_ARCHITECTURE.md](KERNEL_AND_MODULES_ARCHITECTURE.md).

[中文](../../creator-docs/getting-started/PURE_KERNEL_BOUNDARY.md)

---

## 1. Two boundaries

### 1.1 Minimal conceptual core

```text
one turn/lifecycle orchestration path
  + capability-call and merge rules
  + authoritative state commits, errors, and failure isolation
  + six stable ports: memory / emotion / event / prompt / llm / agent
```

The ports are capability interfaces, not six peer decision kernels. Concrete memory/affect algorithms, Prompt templates, model vendors, Event Ring, SQLite, HTTP, Tauri, and role-pack tooling compose around this core. The current co-present health path requires `prompt + llm`; other slots may become thinner through their defined `none` / Noop semantics.

### 1.2 Current complete embedded runtime

`oclive_kernel_host::OcliveKernel` is the supported Rust source-level facade. It reuses the single `process_message` path and exposes role loading, complete/streaming turns, SQLite, plugins, Event Ring, and shutdown lifecycle. It is UI/BSP/vendor independent enough to embed, but the host crate still contains HTTP and many default facilities. Do not call the whole host crate—or all five kernel crates—the physical size of the minimal core.

| Layer | Current anchor | Boundary |
|-------|----------------|----------|
| **Minimal-core skeleton** | six traits in `oclive_kernel_contracts`, turn pipeline, core DTO/errors | Conceptually settled; not yet a separately compiled crate |
| **Complete embedded-runtime facade** | `oclive_kernel_host::OcliveKernel` | Available; includes persistence, Event Ring, HTTP dependencies, and defaults |
| **Transport/UI hosts** | kernel server, Tauri, Vue, VS Code | Outside the essence; must delegate to the same turn entry |

Physical extraction is tracked by `K-CORE-BOUNDARY-01` in [TECHNICAL_DEBT_INVENTORY.md](../../handoff/TECHNICAL_DEBT_INVENTORY.md).

---

## 2. What the minimal tool kernel explicitly excludes

- **Vue frontend**, Tauri `invoke`, windows, and themes.
- **Vendor-specific LLM SDKs** (belong in the `llm` slot: ollama / remote / directory).
- **Board BSP** (mic drivers, motors, RTOS); integrate via **directory plugins / sidecars / MCP**; the kernel consumes contract-shaped results only.
- **Creator doc UI**, plugin market site, launcher install UX.
- **Prompt body language** (pack/model content language); separate from **UI i18n**.
- **Event Ring execution, SQLite repositories, resource coordination, and concrete slot implementations**; these belong to the reference assembly and collaborate through ports or controlled anchors.

---

## 3. Role-pack delivery unit

Role data and slot policy are both portable inputs, but they are not the same contract layer. A distro may ship them in one product package; its adapter must separate minimal role data from host capability bindings before entering the kernel.

| Part | Description |
|------|-------------|
| **Kernel minimal role definition (logical/local-file/optional PNG checks implemented; runtime integration pending)** | Minimum authored content remains persona and one visual asset; the logical contract expresses it as persona prompt plus visual asset references. Logical validation, local-file snapshots and optional static-PNG decoding are independent. PNG is not mandatory, unsupported format does not mean invalid role, and budgets belong to the caller. Seven-image, relation-model and capability boundaries: [ROLE_PACK_BOUNDARY.md](../../handoff/ROLE_PACK_BOUNDARY.md) §0.1–0.4 |
| **Reference-host combined directory (current)** | **`pipeline.ocblueprint`** v2/v3/v4 combines `meta` with `slot_registry` / `runtime_config`, then the loader consumes persona, scene, knowledge, and product-extension files from the same directory. This is a reference implementation input, not the kernel canonical schema ([ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md)) |
| **Effective backends** | Host blueprint `slot_registry` fold + **`set_session_slot_override`** + environment; outside the minimal role definition ([SETTINGS_REFERENCE.md](../cli/SETTINGS_REFERENCE.md)) |
| **Current reference-runtime relation & memory** | `role_runtime` and long-term memory use Repository access; their capability implementations own policy. They do not define minimum role content |

**Robot scenario**: swap role data while the device host independently selects its six-port assembly. A minimal role still carries at least one visual asset; a device without a display may leave it unrendered. Desktop seven-image directories, models, and blueprint policy are not required.

**RobotSoulPack** is the existing reference-host robot/embedded profile, validated by **`oclive pack validate --profile robot-soul`**. Passing it does not establish conformance to the new minimum; fields and sample: [ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md), [examples/robot-soul-minimal](../../examples/robot-soul-minimal/README.md).

---

## 4. Where companion emotion sits

The default companion assembly uses backend modules plus facilities, not one black-box “emotion module”:

- **emotion backend module** + **complex-emotion facility submodule** (facility submodule 1): user affect and cross-turn `narrative_hint`.
- **expert-model facility submodule** (facility submodule 2): conditional expert sub-pipeline (expert routing); **sibling** of complex-emotion—see [OCLIVE_ARCHITECTURE_OVERVIEW.md](OCLIVE_ARCHITECTURE_OVERVIEW.md).
- **memory / event**: relationship and event impact on later turns.
- **prompt / llm**: language and persona injection.
- **agent** (optional): tools and external world (MCP, directory plugins).

The kernel guarantees call order, ports, errors, and state-commit boundaries. Quality comes from the model, slot implementations, facilities, and role content. Strong-model assemblies may omit some explicit assistance; small-model assemblies may use thicker candidate generation and Prompt compilation without making helper output the unique semantic truth.

---

## 5. Deployment shapes of the complete reference runtime

| Shape | Use | Monolith | Notes |
|-------|-----|----------|-------|
| **Desktop host** | Players / creators | Optional (separate project) | Tauri + Vue + same domain |
| **Headless HTTP** | Gateway, robot brain, CI | **Monolith only** for **kernel_server** projects from `oclive-cli` | Workspace **`oclive-kernel-server`** and **`oclivenewnew-tauri --api`** are equivalent (`http_api`); default port **8420** (`OCLIVE_API_PORT`) |
| **Embedded `library`** | In-process embed with your own `main` | **Not applicable** | Link host + contracts/runtime/types. **`OcliveKernel`** exposes role loading, complete and streaming turns, Event Ring, and persistence; generate it with `oclive-cli init --project-type library --kernel-source` ([KERNEL_PLATFORM_DEVELOPER_PATH.md](KERNEL_PLATFORM_DEVELOPER_PATH.md) §5) |
| **HTTP `--api`** | Dev, CI, editor try-chat | N/A | Transition — [headless-kernel-minimal](../../examples/headless-kernel-minimal/README.md) |

**Detachable or welded**: dev-time swappable slots; production optional Monolith weld into one binary—orthogonal to role runtime configuration (current blueprint `slot_registry` / `runtime_config`, plus legacy `settings.json`).

“Stable” here means the supported **Rust source-level facade**, not a stable C ABI. `AppState`, HTTP routes, and Tauri commands remain composition/transport details; integrators should not assemble a second turn pipeline around them. The facade currently lives in `oclive_kernel_host`: consumers do not start HTTP or Tauri, though that crate still contains the HTTP implementation and its dependencies and can be slimmed further.

---

## 6. Honest embedded scope

### In scope (current architecture target)

- Linux user space, devices/gateways with **hundreds of MB RAM** and up.
- **Rust async**, HTTP/JSON-RPC, directory plugin subprocesses, SQLite persistence.
- Different distros adapt to the same minimal role definition and bind providers separately. The complete reference runtime still accepts the combined blueprint and folds it into `PluginBackends`; that is a transition implementation, not a cross-distro format requirement.
- On the current desktop development target, an in-process integration test covers file SQLite, role loading, ordinary/streaming turns, Event Ring registration, and an authorized proactive turn.
- Sidecar LLM (`remote`), local Ollama (`ollama`), hardware via directory plugins.

### Explicitly out of scope (do not over-promise)

- **Hard real-time**, **MCU / KB-scale RAM**, bare-metal without OS.
- **Built-in A/V codec stack** in kernel (use plugins or device services).
- Multi-tenant cloud **isolation and billing** as first-class kernel features (phase B2 if needed).
- This code-level proof is not yet Linux/ARM hardware validation, a long hardware soak, or a measured resource-budget proof; see `V-EMBED-01`.

---

## 7. Related links

- Implementation plan: [KERNEL_IMPLEMENTATION_PLAN.md](KERNEL_IMPLEMENTATION_PLAN.md)
- Kernel integrator path: [KERNEL_PLATFORM_DEVELOPER_PATH.md](KERNEL_PLATFORM_DEVELOPER_PATH.md)
- Current engineering debt: [TECHNICAL_DEBT_INVENTORY.md](../../handoff/TECHNICAL_DEBT_INVENTORY.md) and [PRODUCT_LINE_TASK_BUCKETS.md](../../handoff/PRODUCT_LINE_TASK_BUCKETS.md)
- Doll / hardware delivery pack: **oclive doll core** sibling directory (settings templates, hardware examples); contracts authoritative in this repo.
- Monolith RFC: [RFC_OCLIVE_MONOLITH_MODE.md](../rfc/RFC_OCLIVE_MONOLITH_MODE.md)
