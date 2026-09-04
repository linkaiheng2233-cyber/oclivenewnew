# Kernel integrator learning path

For **headless HTTP**, **embedded**, and **hardware** teams shipping an oclive-compatible runtime. Read [PURE_KERNEL_BOUNDARY.md](PURE_KERNEL_BOUNDARY.md). Scaffold with **`oclive-cli`**: `cargo run -p oclive-cli -- …`.

---

## Beginner (~30 min)

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

- **Contracts** (`KernelErrorBody`, DTOs) live in **`oclive_kernel_runtime`** + [KERNEL_ERROR_CODE_CONVENTION.md](KERNEL_ERROR_CODE_CONVENTION.md).  
- Complete orchestration is maintained once in **`kernel/crates/oclive_kernel_host`**, with the thin desktop shell under `distros/desktop-tauri`. Trusted Rust hosts use it through **`OcliveKernel`**. `oclive-cli ... --project-type library --kernel-source` links that full facade; manually linking only `oclive_kernel_runtime` still provides pure contracts/policy only.
- Embedded hosts may trim peripheral pieces, but should preserve the **error JSON shape** and Event Ring authority boundary. Do not build a second pipeline around internal `AppState`.
