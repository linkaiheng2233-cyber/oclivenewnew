# oclive-cli user guide

**oclive-cli** is the official oclive **kernel / headless project** scaffold: interact in the terminal (or script) to generate a **standalone `cargo build`-able** minimal project for hardware, sidecars, and multiple distribution shapes sharing the same configuration shape.

**Source**: [`kernel/crates/oclive-cli/`](../../kernel/crates/oclive-cli/)

**Contract reference** (full host): [`PLUGIN_V1.md`](../plugin-and-architecture/PLUGIN_V1.md)
**Blueprint `slot_registry` and folded six-slot reference**: [SETTINGS_REFERENCE.md](SETTINGS_REFERENCE.md)

---

## Install and help

From the **oclivenewnew repo root**:

```bash
cargo build -p oclive-cli
cargo run -p oclive-cli -- --help
cargo run -p oclive-cli -- init --help
```

The end of `init --help` lists **presets and the six-slot backend matrix**. The current legacy `init` scaffold writes that matrix as `plugin_backends`, matching the generated root **`CONFIG_REFERENCE.md`**. Current reference-host blueprint packs express composition through v4 `pipeline.ocblueprint` / `slot_registry`; that format is not the kernel-minimal role contract.

**Role pack spec and validation**: [ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md); **`pack`** subcommands are in section 6 of that doc and below.

**Aligned with code**: top-level commands match `kernel/crates/oclive-cli/src/main.rs`. Default help exposes 15 stable entries. The ten known experimental commands remain callable only with the global `--experimental` flag, while the legacy project-archive `template` command remains callable but hidden. Removed top-level `publish` is not an alias.

**Planned CLI** (not shipped): `pack diff`/`update`, `kernel update`, `dev --inject`, `bench history clear`/`export`/`import` — [VISION_ROADMAP_MONTHLY.md](../../creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md#oclive-cli-脚手架计划中).

---

## `scaffold`: local discovery, locking, and bounded generation

```bash
cargo run -p oclive-cli -- scaffold list -o .
cargo run -p oclive-cli -- scaffold inspect com.oclive.scaffold.plugin -o .
cargo run -p oclive-cli -- scaffold validate ./.oclive/scaffolds/example/oclive.scaffold.json
cargo run -p oclive-cli -- scaffold resolve -o . --write-lock --json
cargo run -p oclive-cli -- scaffold generate dev.example.scaffold project \
  -o . --output ../generated --set project_name=demo --accept-untrusted
cargo run -p oclive-cli -- scaffold generate dev.example.scaffold project \
  -o . --output ../preview --set project_name=demo --accept-untrusted --dry-run --json
```

Stage 2A discovers project, user, and compiled official declarations with configurable priority and records their source, maintainer, trust, permissions, namespace, compatibility, and SHA-256 in `.oclive/scaffold.lock.json`. Stage 2B only lets a selected local `instruction` generator materialize files into a **new, absent directory**. The package declares `project.write`, the manifest pins the instruction SHA-256, and the instruction pins every source SHA-256. Project/user packages also need an exact current lock match and per-invocation `--accept-untrusted`. `--set` accepts string values, while `--dry-run` performs the same checks with no writes. A successful run records source and output digests—but no variable values—in `.oclive/scaffold.provenance.json`.

After changing a manifest, inspect the change and rerun `scaffold resolve --write-lock`. V1.0 packages without an instruction digest remain discoverable, but `generate` refuses them and gives the `>=1.1,<2` migration range. Official `builtin` generators remain owned by domain commands such as `oclive init`, `oclive plugin create`, and `oclive pack create`; `scaffold generate` only returns their delegation guidance.

Scaffold Packages still cannot install from a network, execute `commands[].entry`, scripts, or hooks, resolve composition, or control CI workflows, validators, runners, secrets, and gates. Contract SSOT: [RFC_SCAFFOLD_PACKAGE_V1.md](../rfc/RFC_SCAFFOLD_PACKAGE_V1.md).

---

## `doctor`: environment diagnostics

```bash
cargo run -p oclive-cli -- doctor
cargo run -p oclive-cli -- doctor --json
cargo run -p oclive-cli -- doctor -o ./my-project
cargo run -p oclive-cli -- doctor --fix
```

Checks Rust/Cargo, C++ toolchain, memory/disk, Ollama (`http://127.0.0.1:11434/api/tags`), GitHub reachability, and workspace writability. At the **oclivenewnew root** with `distros/chat-pro/roles/*/pipeline.ocblueprint`, it dispatches the declared v2/v3/v4 schema exactly and also checks **`blueprint_file_format`**, **`slot_registry_llm`** (at least one `type: llm`), and **`slot_position_unique`**. Fail items → non-zero exit. JSON Schema: `kernel/crates/oclive-cli/schemas/oclive_doctor_report.schema.json`.

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

**`doctor resource-plan`** previews resource candidates offline. Save the Host's existing `ResourceCoordinationDiagnostics` JSON, then evaluate its profiles, leases, capacities, and finite constraints using the same Host resource compiler. The command neither connects to a Host nor probes local hardware.

```bash
cargo run -p oclive-cli --features diagnostics-host -- doctor resource-plan ./resource-diagnostics.json
cargo run -p oclive-cli --features diagnostics-host -- doctor resource-plan ./resource-diagnostics.json \
  --distro-profile ./distro.oclive.toml --gpu-device-index 0 --json
```

Input must use the current resource diagnostic version (v5); it is not a role pack, blueprint, execution request, or arbitrary resource configuration. Without `--distro-profile`, the captured policy is used. With it, the original HostProfile loader reads `[resource_coordination]`; legacy profiles without that section retain the original defaults. GPU selection uses the lowest device index in the captured snapshot unless explicitly specified; an explicit index is also looked up only in that snapshot. Local `OCLIVE_GPU_DEVICE_INDEX` and resource budget environment overrides do not silently affect the preview. Input files remain unchanged; captured scheduling and candidate results are recomputed.

Human output lists adapter profiles, GPU/RAM/CPU estimates, selections, proposed transitions, and original reason codes. With `--json`, stdout contains one existing `ResourceCandidatePlan` document; stderr explains the offline, non-authoritative scope. Read, type, version, owner, and duplicate adapter ID errors exit nonzero; successfully generated `blocked` or `degraded` diagnostics exit 0. For example, `resource_scheduling_group_conflict` means conflicting constraints, `resource_plan_insufficient_gpu_headroom` means insufficient captured capacity, and `resource_plan_controller_unavailable` means no real controller is available to this preview. Captured files never grant control: even a no-transition candidate with `executable=true` is not live admission or execution permission. The Host must revalidate revision, controllers, and physical capacity for actual execution. This entry provides neither configuration editing nor automatic hardware advice or resource execution, and introduces no new on-disk resource schema. See the [resource RFC](../rfc/RFC_BLUEPRINT_EXTENSION_AND_RESOURCE_COORDINATION.md).

---

## `config resource-policy`: generate an offline resource-policy draft

This noninteractive entry requires `diagnostics-host`. It uses the original distro TOML format and Host validation to generate one new profile; it does not change `~/.oclive/config.toml`, the source distro file, or running configuration.

```toml
# resource-policy.toml: only this section, without other distro settings
[resource_coordination]
strategy = "custom"
gpu_safety_reserve_mib = 1024
[[resource_coordination.commands]]
kind = "residency"
adapter_id = "builtin.test" # Must actually be registered in the supplied capture.
mode = "resident"
```

```bash
cargo run -p oclive-cli --features diagnostics-host -- config resource-policy \
  --distro-profile ./distro.oclive.toml --policy-file ./resource-policy.toml \
  --diagnostics-file ./resource-diagnostics.json --output ./distro-draft.oclive.toml
cargo run -p oclive-cli --features diagnostics-host -- doctor resource-plan ./resource-diagnostics.json \
  --distro-profile ./distro-draft.oclive.toml --json
```

The patch replaces only supplied resource keys and retains omitted keys. `commands` replaces the whole array; `commands = []` clears it explicitly, while the resulting strategy must remain valid. Other settings and unknown extensions retain their TOML values; the original Host loader still determines whether unknown keys have any effect. Serialization does not preserve comments or formatting and may reorder keys; source bytes remain unchanged. The output parent must already exist and the output file must not exist. In-place changes are prohibited. A temporary file in that directory passes the original loader, registry, and finite-intent validation before publication; refusals leave no final draft and never overwrite an existing file.

Invalid types/enums, cross-section patches, bad captures, unregistered adapters, and conflicting intents exit nonzero. Original Host numeric clamps still apply; output reports the effective strategy and GPU reserve, without rewriting raw values in the draft. Valid but degraded observe-only, absent GPU/controller, or insufficient-capacity candidates may be drafted, with original reason codes on stderr. Generation is not live admission and never activates or executes resources. `--gpu-device-index` refers only to the capture; environment overrides do not silently affect evaluation. Interactive guidance, automatic hardware advice, and execution remain unavailable. This bounded tool belongs to [D-SCAFFOLD-RESOURCE-01](../../handoff/TECHNICAL_DEBT_INVENTORY.md), without closing its resource parent debt.

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
- **`validate --profile robot-soul`**: existing reference-host RobotSoulPack rules after legacy validation; passing does not establish conformance to the new minimum (ROLE_PACK_SPEC §6).
- **`validate --profile portable-core`**: validates a v2/v3/v4 blueprint, non-empty `core_personality.txt`, an enabled `portrait_catalog`, and seven fixed default emotion image IDs. The seven-image set is a recommended optional cross-distro standard, but this reference-host profile is distinct from the [kernel minimal contract](../../handoff/ROLE_PACK_BOUNDARY.md), which requires only one visual asset and a persona prompt.
- **`create`**: creates a valid directory in the current reference-host format family; prefer **`--format-blueprint-v4`** for its new Stable examples. `--format-blueprint-v2` remains for compatibility; with `--flat`, `-o` is the role root. It does not generate a cross-distro minimal role pack: the logical contract defines no universal disk format. Developer-supplied definition files and assets can already use [`validate-minimal-local`](../../handoff/ROLE_PACK_BOUNDARY.md#05-第四代码切片调用方指定文件的本地加载准备) for read-only preparation; see [generated artifacts](#generated-artifacts) for the library's basic text usage. Remaining rich lifecycle and actual distro adaptation work is tracked as [D-CLI-BLUEPRINT-05](../../handoff/TECHNICAL_DEBT_INVENTORY.md); successful preparation does not establish role activation.
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

**`--provides`**: `llm` | `memory` | `emotion` | `event` | `prompt` | `agent` | `complex_emotion` (repeatable). Output defaults to the current project's `./plugins/`; pass `-o ./distros/chat-pro/plugins/` explicitly when developing an official monorepo example. Final path is `<output>/<plugin_id>/`. See [PLUGIN_AUTHOR_LEARNING_PATH.md](../plugin-and-architecture/PLUGIN_AUTHOR_LEARNING_PATH.md).

---

## `dev`: watch role pack directories

Run from an **existing** kernel / scaffold project root (with `Cargo.toml`). By default it recursively watches the current **`roles/*/pipeline.ocblueprint`** directly under each role root and continues to support legacy **`roles/*/manifest.json`** and **`roles/*/settings.json`**; use `--roles` to select another root. After a **500ms debounce** it prints:

`[oclive dev] role pack '<id>' changed — reload`

**`--reload-cmd`** runs a shell command after changes.

Edits to v2, v3, or v4 `pipeline.ocblueprint` files produce the same signal. The watcher only emits a development-time reload hint: it neither parses nor rewrites the blueprint and does not create a second role-pack schema.

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
| `minimal` | memory/emotion/event/prompt are `builtin`, and `llm` is **`ollama`**. The logical Agent preset is none, but today's non-dual legacy output **omits the key and therefore resolves to builtin**; its `complex_emotion` hint key is ignored by legacy parsing; plugin placeholders off |
| `mixed` | Matrix-aligned: `llm=ollama`, `agent` / `complex_emotion` `builtin`; some plugin docs on |
| `full` | `llm=remote`, `complex_emotion=remote`, other slots `builtin`; all plugin docs on |

```bash
cargo run -p oclive-cli -- init --non-interactive --quiet --preset minimal -o /tmp/my-kernel
cargo run -p oclive-cli -- init --non-interactive --quiet --preset minimal --skip-role-pack -o /tmp/my-kernel-no-roles
```

`--skip-role-pack`: do not create the root-level `roles/` directory (blank kernel project).

Enable Monolith (non-interactive: add **`--monolith`**; **kernel_server** only):

```bash
cargo run -p oclive-cli -- --experimental init --non-interactive --preset full --monolith -o /tmp/my-monolith-kernel
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

- **Stub `Cargo.toml` without `--kernel-source`**: depends only on **`serde` / `serde_json`** to validate directory/config shape. With `--kernel-source <repo root>`, `kernel_server` links the real headless entry and `library` links host/contracts/runtime/types while re-exporting the complete stable in-process **`OcliveKernel`** facade.
- **`roles/default/settings.json`** (the current non-dual `init` legacy reference-host example): includes `_comment_*`, six-slot `plugin_backends`, and a `complex_emotion` facility hint key ignored by legacy `PluginBackends`. `none` is a legal Noop backend for all six types, although disabling prompt or llm breaks the healthy path. New Stable examples in the current reference-host blueprint family use v4; this neither requires migrating `init` to a complete v4 pack nor defines the kernel-minimal role contract.
- **`CONFIG_REFERENCE.md` (project root)**: preset matrix and one-liner per slot; **developer compile options (Monolith)** and RFC link.
- **End of `init --help`**: preset matrix, **`--monolith`**, pointer to [RFC_OCLIVE_MONOLITH_MODE.md](../rfc/RFC_OCLIVE_MONOLITH_MODE.md).
- **Generated README**: pointers to `oclive_kernel_server`, OOCP, and directory plugins based on project shape/toggles; a linked `library` separately documents the legacy `load_role → process_message` path and converter-prepared `PreparedMinimalRole → process_minimal_message`. Minimal-role handles, requests, results, extension status, and typed errors re-export the original types. The basic result explicitly reports unavailable product extensions without storing chat or constructing a rich `Role`; see [ROLE_PACK_BOUNDARY §0.9](../../handoff/ROLE_PACK_BOUNDARY.md#09-参考-rust-host-的增量基础文本入口). `library-embed --kernel-source <repo root>` generates no role pack by default; developers supply converters. Without `--kernel-source`, the library remains a serde stub.

---

## High-coupling compile mode (Monolith)

**Applies to**: headless **`kernel_server`** placeholder projects; developers comparing **standard** vs **`-monolith`** binaries. **Does not apply**: embedded **library** (`--monolith` is ignored).

**Behavior**: `init --monolith` generates **`monolith.toml`**, `vendor/oclive_monolith_builtin/`, **`src/process_message_monolith.rs`** (welded slots call the vendor crate statically; unwelded slots use trait/PluginHost placeholders), **`Cargo.toml`** **`[features] monolith`** and second **`[[bin]]`** (**`src/main.rs`** standard entry, **`src/main_monolith.rs`** Monolith entry, avoiding duplicate-bin path warnings).

### `build` subcommand

From an **existing** Monolith project root (must contain `monolith.toml`):

```bash
cargo run -p oclive-cli -- --experimental build -o /path/to/kernel-project
cargo run -p oclive-cli -- --experimental build -o /path/to/kernel-project --release --features somefeat
cargo run -p oclive-cli -- --experimental build -o /path/to/kernel-project --no-cargo
```

- **`--no-cargo`**: only regenerate `process_message_monolith.rs` and vendor, do not invoke `cargo`.
- **`--release`** / **`--features`**: forwarded to each `cargo build`; the Monolith second build automatically adds **`monolith`** feature.
- **After `--`**: extra args forwarded to `cargo build`.

**Common build failures**: on `cargo build` failure, the CLI parses stderr and prints fix hints (missing crate, linker, Rust version, OpenSSL, OOM). Otherwise see raw output and run **`oclive doctor`**.

### `bench` subcommand

`bench` remains behind the experimental gate, so commands must place the global **`--experimental`** flag before the subcommand; the examples below already do so.

After regenerating sources and dual builds, runs each binary `--runs` times as subprocesses; inside the subprocess **`OCLIVE_KERNEL_BENCH_ITERS`** controls the hot loop. Output is **JSON** (`schema_version: 2`, includes `binary_size`, `peak_memory`, `build_time`); schema at **`kernel/crates/oclive-cli/schemas/oclive_bench_report.schema.json`**.

```bash
cargo run -p oclive-cli -- --experimental bench --release -o /path/to/kernel-project --runs 30 --inner-iters 500 --output ./bench-report.json
cargo run -p oclive-cli -- --experimental bench --release -o /path/to/kernel-project --json
```

- **`--save`**: append this report to project root **`bench_history.json`** (local file, do not commit).
- **`--compare`**: do not run sampling; read **last two** entries from **`bench_history.json`** and print comparison (needs at least two history rows).
- **`--history`**: print a trend table of all saved runs; with ≥2 rows, shows **↑/↓/→** vs previous. Use **`--json`** for tooling.
- **`--soak --soak-duration <hours>`**: uses an 8–120s accelerated smoke clock by default. Fractional hours are accepted, but accelerated output is not long-duration leak evidence.
- **`--soak-real-time`**: interprets the requested duration as actual wall-clock hours; `--soak-sample-interval <seconds>` controls resource sampling (default 60). Soak completes one `warmup_chats` request before starting the clock and steady-state RSS baseline, then directly samples the Release kernel PID. Schema v2 records RSS/CPU, request failures, early exit, worker join, and `process_reaped`, and any failed criterion returns a non-zero exit.

```bash
cargo run -p oclive-cli -- --experimental bench --release -o ./my-kernel --save
cargo run -p oclive-cli -- --experimental bench --history -o ./my-kernel
cargo run -p oclive-cli -- --experimental bench --soak --soak-real-time --soak-duration 0.01 --soak-sample-interval 5 -o ./my-kernel --output ./soak.json
```

`--json`: print report JSON to **stdout** only (progress on **stderr**) for piping and schema checks. `--output <file>` atomically preserves JSON for regular benchmarks, `--stress`, and `--soak`; when both are present, the stdout-only `--json` contract wins.

**Risk**: placeholder project has **no** real `PluginHost` behavior.

Canonical design: [RFC_OCLIVE_MONOLITH_MODE.md](../rfc/RFC_OCLIVE_MONOLITH_MODE.md).

---

## CI relationship

The main repository also exposes a domain-aware plan; `--shadow` explicitly requests observation-only output:

```bash
cargo run -p oclive-cli -- ci plan --base HEAD^ --head HEAD
cargo run -p oclive-cli -- ci plan --shadow --base HEAD^ --head HEAD
cargo run -p oclive-cli -- ci explain --format markdown
```

`ci plan` reads the centrally owned map, module descriptors, and trusted validation catalog under `data/ci/`. It emits `target/oclive-ci/plan.json`; `ci explain` renders that JSON without recomputing or executing validators. The main repository's Stage 3 policy only uses warning-free, non-fallback, docs-only PR plans to select existing jobs. Non-document PRs, pushes, control-plane changes, and planner failures remain full, and a stable `ci-gate` verifies the actual results. A plan marked `--shadow` must never skip jobs. Validators catalogued as `nightly` remain in the separate scheduled/manual lane. See the [domain-aware CI baseline](../../creator-docs/roadmap/SOMEDAY_TOOLCHAIN_CI.md).

`oclive scaffold` is a separate developer-tool surface. It may help create or inspect standard metadata, but it cannot select CI validators or influence execution policy; CI always re-analyzes generated files independently.

Repo **`.github/workflows/ci.yml`** **`cli`** job runs `cargo test -p oclive-cli` (includes E2E: `init`, `build`, `bench` smoke). The separate **`.github/workflows/nightly-advisory.yml`** **`cli-bench`** job runs one round of `bench` (no perf threshold) and uploads its JSON evidence.

---

## Suggested roadmap

1. **`--kernel-source path`** already writes dependencies for the complete reference runtime. After **K-CORE-BOUNDARY-01** physically extracts the minimal core, add a core-only generation mode while retaining compatibility with the current full-runtime mode.
2. When aligning with `MODULE_NONE_SEMANTICS`, add **auto-validation** for “logical none” vs “loadable JSON”, or a `cargo oclive-validate-settings` subcommand.

---

[中文](../../creator-docs/cli/OCLIVE_CLI_GUIDE.md)
