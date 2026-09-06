# A.I.Live · Minimal tool kernel and current embedded-runtime boundary

This page separates two meanings that older documents called “pure kernel”: the **minimal conceptual core** and today's embeddable **complete reference runtime** are not the same layer. Module taxonomy: [OCLIVE_ARCHITECTURE_OVERVIEW.md](OCLIVE_ARCHITECTURE_OVERVIEW.md). The unique responsibility and integration SSOT is [MODULE_MAP_AND_HANDOFF.md](../../handoff/MODULE_MAP_AND_HANDOFF.md). Diagram: [KERNEL_AND_MODULES_ARCHITECTURE.md](KERNEL_AND_MODULES_ARCHITECTURE.md).

**Last updated:** 2026-09-06.

[中文](../../creator-docs/getting-started/PURE_KERNEL_BOUNDARY.md)

---

## 1. Responsibility target and current implementation

### 1.1 Confirmed minimal-core responsibilities

The responsibility boundary confirmed for this slice is:

```text
one turn/lifecycle orchestration path
  + capability-call and merge rules
  + state-commit validity constraints, errors, and failure isolation
  + six stable ports: memory / emotion / event / prompt / llm / agent
```

The six ports are capability interfaces, not six peer decision kernels. The Core preserves public turn semantics, capability-result and domain-commit validity constraints, error boundaries, and failure isolation; the current reference implementation's trusted Rust Host performs the actual product-state commits and resource operations within those constraints. Responsibility/success terminology must distinguish generation success, domain-commit success, and external-delivery success, but this page does not make delivery a future Core output or add a “commit before streaming” ordering rule. Concrete memory/affect algorithms, Prompt templates, model vendors, Event Ring, SQLite, HTTP, Tauri, and role-pack tooling compose around this core. The current co-present health path requires `prompt + llm`; other slots may become thinner through their defined `none` / Noop semantics.

“Confirmed” here means that the responsibility wording is settled. Kernel v0 contract semantics are settled; its concrete API, fields, and physical extraction remain undecided. The minimal role content and loading preparation likewise do not automatically define the Core's invocation input; whether persona is a hard Core input remains open to the relevant contract.

### Kernel v0 public contract (semantics settled, API undecided)

For each turn, the Host submits one normalized call with the admitted context, capability declarations, and resource scope for that call. The Kernel validates the call, inputs, capability use, dependencies, result ownership, and terminal-state legality, and permits legal scheduling; it does not perform Host scheduling or prescribe one unique chain or fixed concurrency. The six ports consume only inputs declared and admitted in the current call and return capability result, not provided, skipped, or failed; these states do not convert automatically. A port has no domain-state commit authority, permission-grant authority, or second turn pipeline.

The necessary partial order expresses only execution eligibility and result visibility: inputs must be available to, valid for, and owned by the current call. It does not freeze a dependency graph, stages, or fixed order. Prompt runs after its dependency inputs are ready; the main LLM runs after the required Prompt result is valid. `Agent handled` only terminates the unfinished ordinary main chain of the current call; it grants no global scheduling, commit, or exactly-once authority. The terminal state is unique, and later actions cannot validly change it or the main result. The Kernel returns the current-call terminal state, termination path, an optional main result, or a failure reason.

Streaming fragments are incremental process output; they do not prove generation success, domain-commit success, or external-delivery success, which remain independent. After termination, fragments, results, and commits are invalid. This adds only minimal call-association/source semantics, not new fields or a complete provenance model. The Host performs domain commits and resource operations; the Kernel does not write SQLite, Memory, Event, or UI directly, and instead preserves invariants such as current-call ownership, admitted-capability origin, capability validity, no duplicate terminal-state changes, and no permission/resource-scope expansion. Role, Memory store, Event Ring, Stream, SQLite, HTTP, UI, distribution policy, RuntimeSnapshot, relations/favorability, Agent exactly-once, factual truth, and process sandbox are outside this contract. See [MODULE_MAP_AND_HANDOFF.md](../../handoff/MODULE_MAP_AND_HANDOFF.md) for the responsibility boundary.

### 1.2 Current complete embedded runtime

`oclive_kernel_host::OcliveKernel` is the supported Rust source-level **complete reference-runtime facade**, and the current implementation of the trusted Host boundary. It reuses the single `process_message` path and exposes role loading, complete/streaming turns, SQLite, plugins, Event Ring, and shutdown lifecycle. It is UI/BSP/vendor independent enough to embed, but the host crate still contains HTTP and many default facilities. Do not call the whole host crate—or all five kernel crates—the physical size of the minimal core. The Host is not an IPC bridge, Docker container, or security sandbox, and is not inherently one Host per distro.

| Layer | Current anchor | Boundary |
|-------|----------------|----------|
| **Minimal-core responsibility target** | six traits in `oclive_kernel_contracts`, turn pipeline, core DTO/errors | Responsibilities confirmed; Kernel v0 semantics are settled, while its concrete API, fields, and physical extraction remain undecided |
| **Complete embedded-runtime facade** | `oclive_kernel_host::OcliveKernel` | Available; includes persistence, Event Ring, HTTP dependencies, defaults, and actual Host commit/resource operations |
| **Transport/UI adapters** | kernel server, Tauri, Vue, VS Code | Outside Core and Host domain authority; must delegate to the same turn entry |

Physical extraction is tracked by `K-CORE-BOUNDARY-01` in [TECHNICAL_DEBT_INVENTORY.md](../../handoff/TECHNICAL_DEBT_INVENTORY.md). This page does not settle the concrete Kernel v0 API, fields, or extraction stages.

---

## 2. What the minimal tool kernel explicitly excludes

- **Vue frontend**, Tauri `invoke`, windows, and themes.
- **Vendor-specific LLM SDKs** (belong in the `llm` slot: ollama / remote / directory).
- **Board BSP** (mic drivers, motors, RTOS); integrate via **directory plugins / sidecars / MCP**; the kernel consumes contract-shaped results only.
- **Creator doc UI**, plugin market site, launcher install UX.
- **Prompt body language** (pack/model content language); separate from **UI i18n**.
- **Event Ring execution, SQLite repositories, resource coordination, and concrete slot implementations**; these belong to the reference assembly and collaborate through ports or controlled anchors.

`RoleRuntime`, `MemorySystem`, `Relations`, `Favorability`, `Blueprint`, `runtimeState`, SQLite, PNG, Event Ring, and Runtime Event Stream are peripheral assembly, product capabilities, or later facilities; none is required for the Core to exist. A Host or authorized Adapter may provide them, but resource access is not domain-commit authority and an implementation may not use it to bypass error or authorization boundaries.

---

## 3. Role-pack delivery unit

Role data and slot policy are both portable inputs, but they are not the same contract layer. A distro may ship them in one product package; its adapter must separate minimal role data from host capability bindings before entering the kernel.

| Part | Description |
|------|-------------|
| **Confirmed minimal role-content boundary (logical/file-loading preparation/optional PNG checks implemented; runtime integration pending)** | Minimum authored content remains persona and one visual asset; the logical contract expresses it as persona prompt plus visual asset references. A caller-selected JSON file can prepare the definition and read-only asset snapshots without a canonical filename or role activation. PNG is an independent optional capability, unsupported format does not mean invalid role, and budgets belong to the caller; the existing full-`Role` lifecycle still needs separation/adaptation. Seven-image, relation-model and capability boundaries: [ROLE_PACK_BOUNDARY.md](../../handoff/ROLE_PACK_BOUNDARY.md) §0.1–0.5 |
| **Reference-host combined directory (current)** | **`pipeline.ocblueprint`** v2/v3/v4 combines `meta` with `slot_registry` / `runtime_config`, then the loader consumes persona, scene, knowledge, and product-extension files from the same directory. This is a reference implementation input, not the kernel canonical schema ([ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md)) |
| **Effective backends** | Host blueprint `slot_registry` fold + **`set_session_slot_override`** + environment; outside the minimal role definition ([SETTINGS_REFERENCE.md](../cli/SETTINGS_REFERENCE.md)) |
| **Current reference-runtime relation & memory** | `role_runtime` and long-term memory use Repository access; their capability implementations own policy. They do not define minimum role content |

This table confirms role-content and loading-preparation boundaries, not the final Core invocation input. In particular, “persona plus at least one visual reference” does not establish that the Core must directly receive persona in this slice. The public `PromptInput<'a>` still has `role: &'a Role`; that is known coupling in the reference runtime, to be addressed only when the relevant interface scope is decided.

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

The kernel constrains call order, ports, errors, and state-commit boundaries; the trusted Host performs actual product-state commits and resource operations. Quality comes from the model, slot implementations, facilities, and role content. Strong-model assemblies may omit some explicit assistance; small-model assemblies may use thicker candidate generation and Prompt compilation without making helper output the unique semantic truth.

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
