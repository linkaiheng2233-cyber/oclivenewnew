# oclive-cli user guide

**oclive-cli** is the official oclive **kernel / headless project** scaffold: interact in the terminal (or script) to generate a **standalone `cargo build`-able** minimal project for hardware, sidecars, and multiple distribution shapes sharing the same configuration shape.

**Source**: [`crates/oclive-cli/`](../../crates/oclive-cli/)  
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

---

## `pack`: validate and publish role packs

From repo root:

```bash
cargo run -p oclive-cli -- pack validate ./roles/mumu --host-version 0.2.0
cargo run -p oclive-cli -- pack create -o ./out/my-role --flat --id com.example.demo --name Demo
cargo run -p oclive-cli -- pack publish ./out/my-role -o ./dist/com.example.demo-0.1.0.oclivepack
```

- **`validate`**: checks merged `manifest.json` / `settings.json`, `plugin_backends` deserialization, seven-dim `default_personality` range, `interaction_mode`, `min_runtime_version` vs `--host-version`, etc. (aligned with host disk load; no DB).
- **`create`**: minimal valid directory; with `--flat`, `-o` is the role root (otherwise creates `roles/<id>/`).
- **`publish`**: zips the role directory as **`.oclivepack`**; top-level folder name inside the ZIP is **`manifest.id`**.

**JSON Schema** (IDE / `ajv`, etc.): `crates/oclive-cli/schemas/role_pack_manifest.schema.json`, `role_pack_settings.schema.json`, `role_pack_index.schema.json`.

---

## `dev`: watch role pack directories

Run from an **existing** kernel / scaffold project root (with `Cargo.toml`). By default recursively watches **`--roles`** (default `roles/`) for changes; debounces **`manifest.json`** / **`settings.json`** and prints hints; **`--reload-cmd`** runs a shell command after changes (e.g. notify a sidecar to reload).

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

Flow includes: project name, type (headless binary / library), multi-select backend slots, `builtin` / `remote` / `directory` / `none` (`llm` also has **`ollama`**), optional plugin toggles, whether to generate sample `roles/default`; **headless service (`kernel_server`)** ends with **developer compile options** (off by default).

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

`--skip-role-pack`: do not create `roles/` (blank kernel project).

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

Non-interactive mode does **not** require any `--backend-*` flags; if passed, they override only listed slots.

---

## Generated artifacts

- **Stub `Cargo.toml`**: currently depends only on **`serde` / `serde_json`**, not assuming `oclive_kernel_runtime` split crates exist. When wiring a real kernel, switch to `path` / version deps and replace `main.rs` / `lib.rs` entrypoints.
- **`roles/default/settings.json`**: includes **`_comment_*`** and full **`plugin_backends`** (including seventh key `complex_emotion`); trim invalid keys per [SETTINGS_REFERENCE.md](SETTINGS_REFERENCE.md) when matching the full host (e.g. strings `none` the host rejects).
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

### `bench` subcommand

After regenerating sources and dual builds, runs each binary `--runs` times as subprocesses; inside the subprocess **`OCLIVE_KERNEL_BENCH_ITERS`** controls the hot loop. Output is **JSON** (`schema_version: 1`); schema at **`crates/oclive-cli/schemas/oclive_bench_report.schema.json`**.

```bash
cargo run -p oclive-cli -- bench --release -o /path/to/kernel-project --runs 30 --inner-iters 500 --output ./bench-report.json
cargo run -p oclive-cli -- bench --release -o /path/to/kernel-project --json
```

- **`--save`**: append this report to project root **`bench_history.json`** (local file, do not commit; pair with **`--compare`**).
- **`--compare`**: do not run sampling; read **last two** entries from **`bench_history.json`** and print comparison (needs at least two history rows).

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
