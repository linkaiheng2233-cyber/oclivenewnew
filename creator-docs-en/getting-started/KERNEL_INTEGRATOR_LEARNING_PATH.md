# Kernel integrator learning path

For **headless HTTP**, **embedded**, and **hardware** teams shipping an oclive-compatible runtime. Read [PURE_KERNEL_BOUNDARY.md](PURE_KERNEL_BOUNDARY.md). Scaffold with **`oclive-cli`**: `cargo run -p oclive-cli -- …`.

---

## Start with the six Base slots (~10 min)

Run `cargo run -p oclive_kernel_runtime --example base_first_host`, then inspect the [example](../../kernel/crates/oclive_kernel_runtime/examples/base_first_host.rs) and the [Base-only fixture](../../kernel/crates/oclive_kernel_contracts/tests/base_only_fixture.rs). A small external Host selects `MemoryBase`, `PromptBase`, and `LlmBase` as needed; Memory may be omitted, and Emotion, Event, and Agent are not stages of this turn. It also verifies that an unsupported extra Prompt requirement prevents the LLM call. The example uses two limited in-memory reference implementations and a local Echo LLM. It does not load a complete `Role`, database, ChatPro configuration, network service, or real model. Its single-poll helper works only for these immediately ready implementations; a real Host must drive `BaseCallFuture` with its own executor.

**Validation direction:** prove that Base requests, normal empty results, and errors can be called independently; then verify the commitments of concrete implementations; finally test Host/distro adapters and product turns. The six slots are optional capabilities, not a fixed six-stage pipeline. This example is neither a general Host orchestrator nor proof of a separately packaged small Kernel or Stable API.

---

## Reference Host / CLI beginner path (~30 min)

[中文](../../creator-docs/getting-started/KERNEL_INTEGRATOR_LEARNING_PATH.md)

| Step | Goal | Read |
|------|------|------|
| 1 | Kernel-in-the-middle picture | [KERNEL_AND_MODULES_ARCHITECTURE.md](KERNEL_AND_MODULES_ARCHITECTURE.md) |
| 2 | Minimal-kernel boundary | [PURE_KERNEL_BOUNDARY.md](PURE_KERNEL_BOUNDARY.md) |
| 3 | Generate a minimal project | `cargo run -p oclive-cli -- init` ([OCLIVE_CLI_GUIDE.md](../cli/OCLIVE_CLI_GUIDE.md)) |

**Done when:** `cargo build` works in the generated tree and you can distinguish its root-level `roles/` / `plugins/` from the monorepo's `distros/chat-pro/roles/` / `plugins/`. The current non-dual `init` example is still a legacy reference-host combined directory; Stable examples in the current reference-host blueprint family use a v4 `pipeline.ocblueprint` SSOT. Neither is the kernel-minimal role contract; CLI generation/recognition for that contract remains tracked as `D-CLI-BLUEPRINT-05` in [TECHNICAL_DEBT_INVENTORY.md](../../handoff/TECHNICAL_DEBT_INVENTORY.md).

---

## Intermediate (~1–2 h)

| Topic | Read |
|-------|------|
| **`process_message` flow** | Reference host: **`kernel/crates/oclive_kernel_host/src/domain/chat_engine/process_message.rs`**, **`turn_pipeline.rs`**; summary [BUS_FACTOR_NOTES](../../handoff/BUS_FACTOR_NOTES.md) |
| **`PluginHost` slots** | **`kernel/crates/oclive_kernel_host/src/domain/ports/plugin_host.rs`** · [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md) |
| **Backends & fallback** | [SETTINGS_REFERENCE.md](../cli/SETTINGS_REFERENCE.md) · [CONFIGURATION_FILES.md](../guides/CONFIGURATION_FILES.md) |

**Done when:** You can name the main `send_message` stages you expect in logs.

---

## Advanced (~half day)

| Topic | Read |
|-------|------|
| **OOCP / HTTP** | [OOCP_TEST_SUITE.md](../testing/OOCP_TEST_SUITE.md) · [`examples/oocp-test-suite/`](../../examples/oocp-test-suite/) · [headless-kernel-minimal](../../examples/headless-kernel-minimal/README.md) |
| **Monolith** | [RFC_OCLIVE_MONOLITH_MODE.md](../rfc/RFC_OCLIVE_MONOLITH_MODE.md) · `oclive-cli init --monolith` + `build` / `bench` |
| **`--kernel-source`** | [OCLIVE_CLI_GUIDE.md](../cli/OCLIVE_CLI_GUIDE.md) |
| **Stable in-process facade** | `oclive_kernel_host::OcliveKernel`; after generating a library, follow [KERNEL_PLATFORM_DEVELOPER_PATH.md](KERNEL_PLATFORM_DEVELOPER_PATH.md) §5 for role load, turn, and shutdown |
| **Single-track platform doc** | [KERNEL_PLATFORM_DEVELOPER_PATH.md](KERNEL_PLATFORM_DEVELOPER_PATH.md) |

**Done when:** either **`GET /health`** passes on device, or your process completes `OcliveKernel::start → load_role → process_message → shutdown`; then complete one minimal chat round with an injected mock `LlmClient` or the HTTP mock-LLM environment.

---

## Relation to this repo

- **Contracts**: DTOs and `KernelErrorBody` live in **`oclive_kernel_types`**; port traits live in **`oclive_kernel_contracts`**; naming and payload rules in [KERNEL_ERROR_CODE_CONVENTION.md](KERNEL_ERROR_CODE_CONVENTION.md).
- Complete orchestration is maintained once in **`kernel/crates/oclive_kernel_host`**, with the thin desktop shell under `distros/desktop-tauri`. Trusted Rust hosts use it through **`OcliveKernel`**. `oclive-cli ... --project-type library --kernel-source` links that full facade; manually linking only `oclive_kernel_runtime` provides the reference engines and helpers but not the complete Host orchestration facade.
- Embedded hosts may trim peripheral pieces, but should preserve the **error JSON shape** and Event Ring authority boundary. Do not build a second pipeline around internal `AppState`.
