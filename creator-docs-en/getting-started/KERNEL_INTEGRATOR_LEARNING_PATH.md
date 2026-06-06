# Kernel integrator learning path

For **headless HTTP**, **embedded**, and **hardware** teams shipping an oclive-compatible runtime. Read [PURE_KERNEL_BOUNDARY.md](PURE_KERNEL_BOUNDARY.md). Scaffold with **`oclive-cli`**: `cargo run -p oclive-cli -- …`.

---

## Beginner (~30 min)

| Step | Goal | Read |
|------|------|------|
| 1 | Kernel-in-the-middle picture | [KERNEL_AND_MODULES_ARCHITECTURE.md](KERNEL_AND_MODULES_ARCHITECTURE.md) |
| 2 | “Pure kernel” scope | [PURE_KERNEL_BOUNDARY.md](PURE_KERNEL_BOUNDARY.md) |
| 3 | Generate a minimal project | `cargo run -p oclive-cli -- init` ([OCLIVE_CLI_GUIDE.md](../cli/OCLIVE_CLI_GUIDE.md)) |

**Done when:** `cargo build` works in the generated tree and you can locate `roles/` + `settings.json` conventions.

---

## Intermediate (~1–2 h)

| Topic | Read |
|-------|------|
| **`process_message` flow** | Reference host: **`crates/oclive_kernel_host/src/domain/chat_engine/process_message.rs`**, **`turn_pipeline.rs`**; summary [BUS_FACTOR_NOTES](../../handoff/BUS_FACTOR_NOTES.md) |
| **`PluginHost` slots** | **`crates/oclive_kernel_host/src/domain/ports/plugin_host.rs`** · [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md) |
| **Backends & fallback** | [SETTINGS_REFERENCE.md](../cli/SETTINGS_REFERENCE.md) · [CONFIGURATION_FILES.md](../guides/CONFIGURATION_FILES.md) |

**Done when:** You can name the main `send_message` stages you expect in logs.

---

## Advanced (~half day)

| Topic | Read |
|-------|------|
| **OOCP / HTTP** | [OOCP_TEST_SUITE.md](../testing/OOCP_TEST_SUITE.md) · [`examples/oocp-test-suite/`](../../examples/oocp-test-suite/) · [headless-kernel-minimal](../../examples/headless-kernel-minimal/README.md) |
| **Monolith** | [RFC_OCLIVE_MONOLITH_MODE.md](../rfc/RFC_OCLIVE_MONOLITH_MODE.md) · `oclive-cli init --monolith` + `build` / `bench` |
| **`--kernel-source`** | [OCLIVE_CLI_GUIDE.md](../cli/OCLIVE_CLI_GUIDE.md) |
| **Single-track platform doc** | [KERNEL_PLATFORM_DEVELOPER_PATH.md](KERNEL_PLATFORM_DEVELOPER_PATH.md) |

**Done when:** **`GET /health`** (or equivalent) passes on device and you complete one minimal chat round (CI-style mock LLM env vars per OOCP doc).

---

## Relation to this repo

- **Contracts** (`KernelErrorBody`, DTOs) live in **`oclive_kernel_runtime`** + [KERNEL_ERROR_CODE_CONVENTION.md](KERNEL_ERROR_CODE_CONVENTION.md).  
- **`src-tauri`** is the fullest reference host; trim for embedded but keep **error JSON shape** compatible with shared docs/tools.
