# oclive-cli user guide

**oclive-cli** is the official oclive **kernel / headless project** scaffold: interact in the terminal (or script) to generate a **standalone `cargo build`-able** minimal project for hardware, sidecars, and multiple distribution shapes sharing the same configuration shape.

**Source**: [`kernel/crates/oclive-cli/`](../../kernel/crates/oclive-cli/)  
**Contract reference** (full host): [`PLUGIN_V1.md`](../plugin-and-architecture/PLUGIN_V1.md)  
**Authoritative `plugin_backends` field reference**: [SETTINGS_REFERENCE.md](SETTINGS_REFERENCE.md)

---

## Install and help

From the **oclivenewnew repo root**:

```bash
cargo build -p oclive-cli
cargo run -p oclive-cli -- --help
cargo run -p oclive-cli -- init --help
```

The end of `init --help` lists **presets and the `plugin_backends` matrix** (same as the generated project root **`CONFIG_REFERENCE.md`**).

**Role pack spec and validation**: [ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md); **`pack`** subcommands are in section 6 of that doc and below.

**Aligned with code**: top-level commands match `kernel/crates/oclive-cli/src/main.rs`. See the Chinese guide for the full **A / B / C** tier table, deprecated aliases, and **planned** (not yet implemented) items.

**Planned CLI** (not shipped): `pack diff`/`update`, `kernel update`, `dev --inject`, `bench history clear`/`export`/`import` — [VISION_ROADMAP_MONTHLY.md](../../creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md#oclive-cli-脚手架计划中).

---

## `doctor`: environment diagnostics

```bash
cargo run -p oclive-cli -- doctor
cargo run -p oclive-cli -- doctor --json
cargo run -p oclive-cli -- doctor -o ./my-project
cargo run -p oclive-cli -- doctor --fix
```

Checks Rust/Cargo, C++ toolchain, memory/disk, Ollama (`http://127.0.0.1:11434/api/tags`), GitHub reachability, workspace writability. At the **oclivenewnew root** with `distros/chat-pro/roles/*/pipeline.ocblueprint`, also runs three v2 blueprint checks: **`blueprint_file_format`**, **`slot_registry_llm`** (at least one `type: llm`), **`slot_position_unique`**. Fail items → non-zero exit. JSON Schema: `kernel/crates/oclive-cli/schemas/oclive_doctor_report.schema.json`.

**`doctor config-resolve`** (effective six-slot backends + source chain; **default** uses `oclive_kernel_runtime::resolve_session_plugin_backends` **pure resolution** + on-disk role packs — **no** SQLite / Axum / Tauri):

```bash
cargo run -p oclive-cli -- doctor config-resolve mumu
cargo run -p oclive-cli -- doctor config-resolve mumu --session-id demo --json
cargo run -p oclive-cli -- doctor config-resolve mumu -o distros/chat-pro/roles --json
# Optional deep diagnosis: in-memory AppState full-chain parity (needs diagnostics-host feature)
cargo run -p oclive-cli --features diagnostics-host -- doctor config-resolve mumu --via-host --json
```

With `--json`, **stdout is a single JSON document**; human-readable titles go to stderr. Dependency boundary: [COMPATIBILITY.md](../COMPATIBILITY.md) · [`doctor_config_resolve.rs`](../../kernel/crates/oclive-cli/src/doctor_config_resolve.rs) · runtime SSOT [`plugin_resolution.rs`](../../kernel/crates/oclive_kernel_runtime/src/domain/plugin_resolution.rs).

**`doctor execution-plan`** resolves v4 extensions, Provider candidates, permissions/dependencies, and distro-specific degradation. It is read-only and never starts a plugin:

```bash
cargo run -p oclive-cli --features diagnostics-host -- doctor execution-plan mumu --json
cargo run -p oclive-cli --features diagnostics-host -- doctor execution-plan my-role \
  -o ./distros/chat-pro/roles \
  --app-data-dir ./tmp/app-data \
  --distro-profile ./distros/desktop-tauri/resources/distro-profiles/theater.oclive.toml \
  --json
```

The command explicitly requires `diagnostics-host` because it reuses the host role parser, Capability Registry, and Plan Compiler; the default CLI dependency surface remains lightweight. The `ExecutionPlan` is in memory only. `resource_coordination: not_evaluated` with no `resource_plan` means pure compilation did not probe devices. Desktop diagnostics refresh the Resource Coordinator and attach a read-only candidate resource plan. An unavailable required extension is `blocked`; an unavailable optional extension is `degraded`.

---

## `pack`: validate and publish role packs

From repo root:

```bash
# Blueprint pack (dispatched by exact schema_version)
cargo run -p oclive-cli -- pack validate ./distros/chat-pro/roles/mumu --host-version 0.2.0
cargo run -p oclive-cli -- pack validate ./distros/chat-pro/roles/legacy-example --profile legacy
cargo run -p oclive-cli -- pack create -o ./out/my-role --flat --id com.example.demo --name Demo --format-blueprint-v4
cargo run -p oclive-cli -- pack publish ./out/my-role -o ./dist/com.example.demo-0.1.0.oclivepack
```

- **`validate` (exact v2/v3/v4 dispatch)**: `pipeline.ocblueprint` (`meta`, `slot_registry`, at least one `type: llm`, etc.). v4 is Stable; v3 is the frozen dual-core Beta — see [ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md).
- **`validate --profile legacy`**: merged `manifest.json` / `settings.json`, `plugin_backends`, `min_runtime_version` vs `--host-version`, etc.
- **`validate --profile robot-soul`**: RobotSoulPack rules after legacy validation (ROLE_PACK_SPEC §6).
- **`create`**: minimal pack; prefer **`--format-blueprint-v4`** for new Stable packs. `--format-blueprint-v2` remains for compatibility; with `--flat`, `-o` is the role root.
- **`publish`**: **`.oclivepack`** ZIP; top-level folder is **`meta.id`** (v2/v3/v4) or **`manifest.id`** (legacy).

**JSON Schema**: `kernel/crates/oclive-cli/schemas/pipeline.ocblueprint.v2.schema.json`, `pipeline.ocblueprint.v3.schema.json`, and `pipeline.ocblueprint.v4.schema.json`; legacy: `role_pack_manifest.schema.json`, `role_pack_settings.schema.json`, `role_pack_index.schema.json`.

---

## `plugin create`: plugin scaffold

Generate a **directory** plugin (Node `rpc_server.mjs` + child process) or **remote HTTP** plugin (Python `rpc_server.py`) with `manifest.json` (`id`, `provides`, `permissions`, `rpcMethods`), README, and RPC stubs.

```bash
cargo run -p oclive-cli -- plugin create my-llm-plugin --type directory --provides llm -o ./distros/chat-pro/plugins/
cargo run -p oclive-cli -- plugin create my-remote --type remote --provides memory --provides emotion -o ./out/plugin --non-interactive
cargo run -p oclive-cli -- plugin create my-plugin
```

**`--provides`**: `llm` | `memory` | `emotion` | `event` | `prompt` | `agent` | `complex_emotion` (repeatable). Output defaults to `./distros/chat-pro/plugins/`; final path is `<output>/<plugin_id>/`. See [PLUGIN_AUTHOR_LEARNING_PATH.md](../plugin-and-architecture/PLUGIN_AUTHOR_LEARNING_PATH.md).

---

## `dev`: watch role pack directories

Run from an **existing** kernel / scaffold project root (with `Cargo.toml`). **Recursive notify** on **`distros/chat-pro/roles/**/manifest.json`** and **`distros/chat-pro/roles/**/settings.json`**; **500ms debounce** then:

`[oclive dev] role pack '<id>' changed — reload`

**`--reload-cmd`** runs a shell command after changes.

```bash
cargo run -p oclive-cli -- dev -o /path/to/project
cargo run -p oclive-cli -- dev -o /path/to/project --roles roles --reload-cmd "echo reload"
cargo run -p oclive-cli -- dev -o /path/to/project --no-watch
```

---

## `init`: create a project

### Interactive (default)

```bash
cargo run -p oclive-cli -- init -o ./out/my-kernel
```

Flow includes: project name, type (headless binary / library), multi-select backend slots, `builtin` / `remote` / `directory` / `none` (`llm` also has **`ollama`**), optional plugin toggles, whether to generate sample `distros/chat-pro/roles/default`; **headless service (`kernel_server`)** ends with **developer compile options** (off by default).

### Non-interactive + presets

| Preset | Meaning |
|--------|---------|
| `minimal` | All six slots `builtin` semantics; `llm` is **`ollama`**; `agent` **omits JSON key**; `complex_emotion` is `none`; plugin placeholders off |
| `mixed` | Matrix-aligned: `llm=ollama`, `agent` / `complex_emotion` `builtin`; some plugin docs on |
| `full` | `llm=remote`, `complex_emotion=remote`, other slots `builtin`; all plugin docs on |

```bash
cargo run -p oclive-cli -- init --non-interactive --quiet --preset minimal -o /tmp/my-kernel
cargo run -p oclive-cli -- init --non-interactive --quiet --preset minimal --skip-role-pack -o /tmp/my-kernel-no-roles
```

`--skip-role-pack`: do not create `distros/chat-pro/roles/` (blank kernel project).

Enable Monolith (non-interactive: add **`--monolith`**; **kernel_server** only):

```bash
cargo run -p oclive-cli -- init --non-interactive --preset full --monolith -o /tmp/my-monolith-kernel
cargo build --release --manifest-path /tmp/my-monolith-kernel/Cargo.toml
cargo build --release --features monolith --manifest-path /tmp/my-monolith-kernel/Cargo.toml
```

Full matrix text is at the end of **`init --help`** or in [SETTINGS_REFERENCE.md](SETTINGS_REFERENCE.md) under “`oclive-cli` preset matrix”.

Library type:

```bash
cargo run -p oclive-cli -- init --non-interactive --quiet --preset mixed --project-type library -o /tmp/my-lib
```

### Common flags

| Flag | Meaning |
|------|---------|
| `-o` / `--output` | Output directory (must be empty or not exist; created) |
| `--non-interactive` | Use `--preset`, no dialoguer prompts |
| `--quiet` | Suppress config summary and completion messages (scripting) |
| `--preset` | `minimal` \| `full` \| `mixed` |
| `--project-type` | `kernel-server` \| `library` |
| `--project-name` | Default `my_oclive_kernel` |
| `--monolith` | Non-interactive: enable Monolith; generates `monolith.toml`, `vendor/oclive_monolith_builtin/`, dual `[[bin]]` (`main.rs` / `main_monolith.rs`) and `process_message_monolith.rs` (**kernel_server only**; ignored when incompatible with `--project-type library`) |
| `--author` / `--license` / `--description` | Written into generated `Cargo.toml` (`license` defaults to **MIT**; interactive author defaults to `git config user.name`) |

Non-interactive mode does **not** require any `--backend-*` flags; if passed, they override only listed slots.

---

## Generated artifacts

- **Stub `Cargo.toml`**: currently depends only on **`serde` / `serde_json`**, not assuming `oclive_kernel_runtime` split crates exist. When wiring a real kernel, switch to `path` / version deps and replace `main.rs` / `lib.rs` entrypoints.
- **`distros/chat-pro/roles/default/settings.json`**: includes **`_comment_*`** and full **`plugin_backends`** (including seventh key `complex_emotion`); trim invalid keys per [SETTINGS_REFERENCE.md](SETTINGS_REFERENCE.md) when matching the full host (e.g. strings `none` the host rejects).
- **`CONFIG_REFERENCE.md` (project root)**: preset matrix and one-liner per slot; **developer compile options (Monolith)** and RFC link.
- **End of `init --help`**: preset matrix, **`--monolith`**, pointer to [RFC_OCLIVE_MONOLITH_MODE.md](../rfc/RFC_OCLIVE_MONOLITH_MODE.md).
- **Generated README**: textual pointers to `oclive_kernel_server`, OOCP, directory plugins based on toggles.

---

## High-coupling compile mode (Monolith)

**Applies to**: headless **`kernel_server`** placeholder projects; developers comparing **standard** vs **`-monolith`** binaries. **Does not apply**: embedded **library** (`--monolith` is ignored).

**Behavior**: `init --monolith` generates **`monolith.toml`**, `vendor/oclive_monolith_builtin/`, **`src/process_message_monolith.rs`** (welded slots call the vendor crate statically; unwelded slots use trait/PluginHost placeholders), **`Cargo.toml`** **`[features] monolith`** and second **`[[bin]]`** (**`src/main.rs`** standard entry, **`src/main_monolith.rs`** Monolith entry, avoiding duplicate-bin path warnings).

### `build` subcommand

From an **existing** Monolith project root (must contain `monolith.toml`):

```bash
cargo run -p oclive-cli -- build -o /path/to/kernel-project
cargo run -p oclive-cli -- build -o /path/to/kernel-project --release --features somefeat
cargo run -p oclive-cli -- build -o /path/to/kernel-project --no-cargo
```

- **`--no-cargo`**: only regenerate `process_message_monolith.rs` and vendor, do not invoke `cargo`.
- **`--release`** / **`--features`**: forwarded to each `cargo build`; the Monolith second build automatically adds **`monolith`** feature.
- **After `--`**: extra args forwarded to `cargo build`.

**Common build failures**: on `cargo build` failure, the CLI parses stderr and prints fix hints (missing crate, linker, Rust version, OpenSSL, OOM). Otherwise see raw output and run **`oclive doctor`**.

### `bench` subcommand

After regenerating sources and dual builds, runs each binary `--runs` times as subprocesses; inside the subprocess **`OCLIVE_KERNEL_BENCH_ITERS`** controls the hot loop. Output is **JSON** (`schema_version: 2`, includes `binary_size`, `peak_memory`, `build_time`); schema at **`kernel/crates/oclive-cli/schemas/oclive_bench_report.schema.json`**.

```bash
cargo run -p oclive-cli -- bench --release -o /path/to/kernel-project --runs 30 --inner-iters 500 --output ./bench-report.json
cargo run -p oclive-cli -- bench --release -o /path/to/kernel-project --json
```

- **`--save`**: append this report to project root **`bench_history.json`** (local file, do not commit).
- **`--compare`**: do not run sampling; read **last two** entries from **`bench_history.json`** and print comparison (needs at least two history rows).
- **`--history`**: print a trend table of all saved runs; with ≥2 rows, shows **↑/↓/→** vs previous. Use **`--json`** for tooling.

```bash
cargo run -p oclive-cli -- bench --release -o ./my-kernel --save
cargo run -p oclive-cli -- bench --history -o ./my-kernel
```

`--json`: print report JSON to **stdout** only (progress on **stderr**) for piping and schema checks.

**Risk**: placeholder project has **no** real `PluginHost` behavior.

Canonical design: [RFC_OCLIVE_MONOLITH_MODE.md](../rfc/RFC_OCLIVE_MONOLITH_MODE.md).

---

## CI relationship

Repo **`.github/workflows/ci.yml`** **`cli`** job runs `cargo test -p oclive-cli` (includes E2E: `init`, `build`, `bench` smoke). A lighter **`cli-bench`** job runs one round of `bench` (no perf threshold).

---

## Suggested roadmap

1. After **`oclive_kernel_runtime`** lands in the workspace, add **`--kernel-source path`** to the CLI to auto-write `Cargo.toml` deps.  
2. When aligning with `MODULE_NONE_SEMANTICS`, add **auto-validation** for “logical none” vs “loadable JSON”, or a `cargo oclive-validate-settings` subcommand.

---

[中文](../../creator-docs/cli/OCLIVE_CLI_GUIDE.md)
