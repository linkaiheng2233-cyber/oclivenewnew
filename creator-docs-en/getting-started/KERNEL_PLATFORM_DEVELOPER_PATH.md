# Platform developer path: scaffold to deploy (single track)

One minimal path for **integrators / hardware / gateways**, aligned with [PURE_KERNEL_BOUNDARY.md](PURE_KERNEL_BOUNDARY.md) and [KERNEL_IMPLEMENTATION_PLAN.md](KERNEL_IMPLEMENTATION_PLAN.md).

[中文](../../creator-docs/getting-started/KERNEL_PLATFORM_DEVELOPER_PATH.md)

---

## 1. Prereqs

1. Clone **[oclivenewnew](https://github.com/linkaiheng2233-cyber/oclivenewnew)**.
2. **Rust** + **Node 20+** (for OOCP black-box).
3. Optional: **oclive doll core** pack next to this repo (school/industry doll delivery template); cross-linked from its `README.md`.

---

## 2. Single track

| Step | Action | Outcome |
|------|--------|---------|
| 1 | `cargo build -p oclive-cli` | CLI ready |
| 2 | `cargo run -p oclive-cli -- init --kernel-source <repo root> -o <proj> …` | **kernel_server** or **library** with path deps |
| 3 | Author **`distros/chat-pro/roles/<id>/`** (`pack create` or copy [examples/robot-soul-minimal](../../examples/robot-soul-minimal/)) | `pack validate`; devices: **`--profile robot-soul`** ([ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md)) |
| 4 | Directory plugins / sidecars (optional) | [DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md), [REMOTE_PLUGIN_PROTOCOL.md](../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md) |
| 5 | `cargo run -p oclive-cli -- pack validate <role root> [--profile robot-soul]` | Contract + RobotSoulPack |
| 6 | Headless | **`cargo run -p oclive_kernel_server -- --api`** or generated project **`cargo run`**; or **`oclivenewnew-tauri --api`** |
| 7 | Ship | Binary + `distros/chat-pro/roles/` + `distros/chat-pro/plugins/` (if directory) + env: `OCLIVE_ROLES_DIR`, `OCLIVE_API_PORT`, `OCLIVE_HTTP_API_MOCK_LLM` (bring-up), … |

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

Set `plugin_backends.llm = "remote"` and `OCLIVE_REMOTE_LLM_URL` ([SETTINGS_REFERENCE.md](../cli/SETTINGS_REFERENCE.md)).

---

## 5. Embedded `library` shape

- **`oclive-cli init --project-type library --kernel-source <repo root>`** → **`lib`** depending on **`oclive_kernel_runtime`** (no Tauri).
- Use **`oclive_kernel_runtime::`** DTOs and pure `domain` in your process; **full turn orchestration** (`process_message`, `AppState`) stays in **`oclivenewnew-tauri`**—link or call via HTTP as needed.

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
| [SETTINGS_REFERENCE.md](../cli/SETTINGS_REFERENCE.md) | `plugin_backends` |
| [ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md) | Disk pack + **RobotSoulPack** |
| [AGENTS.md](../../AGENTS.md) | Collaboration & tests |

**oclive doll core** (sibling folder): template pack + `README.md`.
