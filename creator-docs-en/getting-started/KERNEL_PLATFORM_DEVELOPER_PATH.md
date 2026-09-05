# Kernel integrator path: scaffold to deploy (single track)

One minimal path for **integrators / hardware / gateways**, aligned with [PURE_KERNEL_BOUNDARY.md](PURE_KERNEL_BOUNDARY.md) and [KERNEL_IMPLEMENTATION_PLAN.md](KERNEL_IMPLEMENTATION_PLAN.md).

This path integrates the current **complete reference-runtime facade**, `OcliveKernel`. The filename remains for link compatibility; “platform” is not the definition of OCLive's minimal tool kernel. Physical extraction is tracked by `K-CORE-BOUNDARY-01`.

[中文](../../creator-docs/getting-started/KERNEL_PLATFORM_DEVELOPER_PATH.md)

---

## 1. Prereqs

1. Clone **[oclivenewnew](https://github.com/linkaiheng2233-cyber/oclivenewnew)**.
2. **Rust** + **Node 22+** (for OOCP black-box).
3. Optional: **oclive doll core** pack next to this repo (school/industry doll delivery template); cross-linked from its `README.md`.

---

## 2. Single track

| Step | Action | Outcome |
|------|--------|---------|
| 1 | `cargo build -p oclive-cli` | CLI ready |
| 2 | `cargo run -p oclive-cli -- init --kernel-source <repo root> -o <proj> …` | **kernel_server** or **library** with path deps |
| 3 | Author a pack under the generated project's root-level **`roles/<id>/`** (`pack create` or copy [examples/robot-soul-minimal](../../examples/robot-soul-minimal/)) | `pack validate`; devices: **`--profile robot-soul`** ([ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md)) |
| 4 | Directory plugins / sidecars (optional) | [DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md), [REMOTE_PLUGIN_PROTOCOL.md](../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md) |
| 5 | `cargo run -p oclive-cli -- pack validate <role root> [--profile robot-soul]` | Current reference-host format + RobotSoulPack rules |
| 6 | Run | Cross-process: **`cargo run -p oclive_kernel_server -- --api`** or **`oclivenewnew-tauri --api`**; in-process: call **`OcliveKernel`** from the generated `library` |
| 7 | Ship | Binary + root-level `roles/` + `plugins/` (if directory) + env: `OCLIVE_ROLES_DIR`, `OCLIVE_API_PORT`, `OCLIVE_HTTP_API_MOCK_LLM` (bring-up), …; only built-in monorepo examples live under `distros/chat-pro/` |

These steps validate the existing reference host. The independent [kernel minimal role contract](../../handoff/ROLE_PACK_BOUNDARY.md) has a confirmed content boundary, but loader/CLI integration remains unimplemented; passing `robot-soul` does not replace that acceptance gate.

---

## 3. Headless & default port

- **Default HTTP port**: **8420** (`OCLIVE_API_PORT` overrides).
- **No LLM bring-up**: `OCLIVE_HTTP_API_MOCK_LLM=1`.
- **Black box**: `examples/oocp-test-suite/run.mjs` (after `GET /health`).

See [examples/headless-kernel-minimal/README.md](../../examples/headless-kernel-minimal/README.md), [OOCP_TEST_SUITE.md](../testing/OOCP_TEST_SUITE.md).

---

## 4. Default LLM simulation (sidecar)

**OpenAI-compatible HTTP** sample:

- **[examples/remote_plugin_openai_compat/README.md](../../examples/remote_plugin_openai_compat/README.md)**

Set the role blueprint's `type: llm` instance to `backend: remote` and configure `OCLIVE_REMOTE_LLM_URL` ([SETTINGS_REFERENCE.md](../cli/SETTINGS_REFERENCE.md)).

---

## 5. Embedded `library` shape

- **`oclive-cli init --project-type library --kernel-source <repo root>`** generates a standalone-`cargo check` **`lib`** that re-exports `OcliveKernel` and host/contracts/runtime/types. It needs neither Tauri nor an HTTP server.
- `OcliveKernel` is the supported facade for trusted Rust hosts. It reuses the same `AppState`, `process_message`, repositories, PluginHost, Event Ring, and resource-coordination path as desktop and headless transports; it does not create another orchestration path.

### Minimal in-process turn

```rust,no_run
use my_oclive_kernel::{types, KernelResult, OcliveKernel, OcliveKernelConfig};

async fn one_turn() -> KernelResult<()> {
    let config = OcliveKernelConfig::new("./data", "./roles");
    let kernel = OcliveKernel::start(config).await?;
    kernel.load_role("my-role").await?;

    let response = kernel
        .process_message(&types::SendMessageRequest {
            role_id: "my-role".into(),
            user_message: "Hello".into(),
            ..Default::default()
        })
        .await?;

    println!("{}", response.reply);
    kernel.shutdown().await;
    Ok(())
}
```

### Stable facade surface

| Category | Entry |
|----------|-------|
| Lifecycle | `OcliveKernelConfig` → `OcliveKernel::start` / `builder` → consuming `shutdown(self)` |
| Roles | `list_roles` · `load_role` · `role_info` |
| Turns | `process_message` · `process_message_stream`; trusted hosts also get `*_with_origin` for `sensor` / `system` |
| Event Ring | Implements `contracts::EventModuleRegistrar`; `propose_proactive_turn` → one-use permit → `process_proactive_turn`; `event_ring_diagnostics` |
| Host adapters | Builder injection for `contracts::LlmClient` and explicit `HostProfile`; errors retain stable `KernelErrorBody` codes |

`shutdown(self)` stops managed directory-plugin/model runtimes and waits for the SQLite pool to close. Hosts may share the handle in an `Arc`, but should recover unique ownership and shut it down explicitly at process teardown.

**Boundary**: this is a Rust source-level facade, not a C ABI. Internal `AppState` and HTTP/Tauri adapters are not integration contracts. The code-level loop is verified; Linux/ARM hardware, resource-budget, and long-soak proof remain under `V-EMBED-01` in [TECHNICAL_DEBT_INVENTORY.md](../../handoff/TECHNICAL_DEBT_INVENTORY.md).

---

## 6. Monolith (kernel_server scaffold only)

See [RFC_OCLIVE_MONOLITH_MODE.md](../rfc/RFC_OCLIVE_MONOLITH_MODE.md) and **`oclive build` / `oclive bench`**. **`library` projects do not use Monolith.**

---

## 7. OTA / remote logging

**P2**, not blocking K1–K4 ([KERNEL_IMPLEMENTATION_PLAN.md](KERNEL_IMPLEMENTATION_PLAN.md) K5).

---

## 8. Links

| Doc | Role |
|-----|------|
| [OCLIVE_CLI_GUIDE.md](../cli/OCLIVE_CLI_GUIDE.md) | `init` / `build` / `bench` / `pack` / `dev` |
| [SETTINGS_REFERENCE.md](../cli/SETTINGS_REFERENCE.md) | Current `slot_registry`, folded backends, and legacy `plugin_backends` |
| [ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md) | Disk pack + **RobotSoulPack** |
| [AGENTS.md](../../AGENTS.md) | Collaboration & tests |

**oclive doll core** (sibling folder): template pack + `README.md`.
