# Contributing to A.I.Live

[中文](CONTRIBUTING.md)

Thank you for helping improve **A.I.Live** (engineering codename **oclive**). High-level goals are described in [creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md](creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md).

**Human onboarding (no Cursor):** start at [human-docs/README.md](human-docs/README.md) — L0–L2 cover clone, `tauri:dev`, and `npm run check` in ~30 minutes. Deeper contract SSOT remains partly Chinese under `creator-docs/`.

## GitHub (CI, Dependabot, branch protection)

After merges to the default branch, **Dependabot** opens PRs per [`.github/dependabot.yml`](.github/dependabot.yml). **CI** runs on Actions. For org/repo settings (branch protection, secrets), see **[creator-docs/getting-started/GITHUB_REPO_CHECKLIST.md](creator-docs/getting-started/GITHUB_REPO_CHECKLIST.md)**.

## Getting help

- **General questions, install, and configuration:** use [**GitHub Issues**](https://github.com/linkaiheng2233-cyber/oclivenewnew/issues) with the **Bug / Feature / Support** templates; prefer `[bug]:` / `[feat]:` / `[support]:` title prefixes (see root [README.en.md](README.en.md) **Support**). First triage is usually within **3–5 business days** (best effort, not an SLA).  
- **Self-serve:** [FAQ](creator-docs/FAQ.md) · [Documentation index](creator-docs/getting-started/DOCUMENTATION_INDEX.md) · [ERROR_CODES](creator-docs/getting-started/ERROR_CODES.md).  
- **Security vulnerabilities:** do **not** disclose in public issues — see [SECURITY.md](SECURITY.md).

## Development environment

- **This repo:** **Node.js ≥ 22** (see root `package.json` engines; optional `.nvmrc`), **npm**, **Rust** stable, **Ollama** (optional for local dialogue).
- **Windows:** **Visual Studio Build Tools** (MSVC linker).
- **After clone:** run **`npm install`** at the repo root; **`npm run tauri:dev`** drives the Tauri + `src-tauri` build.
- **Rust workspace only** (`oclive_validation`, `oclive-cli`, `oclivenewnew-tauri`): **`cargo test --workspace`** from the root, or **`cargo test --manifest-path distros/desktop-tauri/Cargo.toml`** for the desktop crate only.
- **Cargo `target-dir`:** [`.cargo/config.toml`](.cargo/config.toml) points to **`../oclive-dev-artifacts/oclivenewnew-cargo-target/`** outside the clone.

## Build & run locally

```bash
npm install
npm run tauri:dev
npm run dev
npm run build
```

**Local HTTP API** (same binary as the GUI): add **`--api`** to the built executable; see the root [README.md](README.md) (Chinese) or [README.en.md](README.en.md) for details.

## Code style (Rust / Vue)

- **Rust**
  - **Format:** `cargo fmt`; CI uses **`npm run check:rust:fmt`** (`cargo fmt --manifest-path distros/desktop-tauri/Cargo.toml --all -- --check`).
  - **Clippy:** Root **[`Cargo.toml`](Cargo.toml)** defines **`[workspace.lints.rust]`** and **`[workspace.lints.clippy]`**. Local + CI: **`cargo clippy --manifest-path distros/desktop-tauri/Cargo.toml --all-targets --all-features -- -D warnings`** (**`npm run check:rust:clippy`**): **warnings are errors**.
  - **`unwrap` / `expect`:** Prefer **`Result` / `Option` + `context`** in product code; integration tests may use **`#![allow(clippy::unwrap_used, clippy::expect_used)]`** at the crate root. Do not widen allows elsewhere.
- **Vue / TypeScript:** Match existing composables/stores; align with Tauri DTO field names (e.g. **`reply`**, defined in **`oclive_kernel_runtime`** models and re-exported from `kernel/crates/oclive_kernel_types/src/models/mod.rs`).

## Commits

- **[Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/)**: `type(optional-scope): short description`.
- **Common types:** **`feat`**, **`fix`**, **`docs`**, **`chore`**, **`refactor`**, **`test`**, **`perf`**, **`ci`**.
- **Examples:** **`docs: update README feature matrix`** · **`fix(chat): handle empty session id`**.

## Tests (before merge)

| Scenario | Command |
|----------|---------|
| Day-to-day (matches `npm run check`) | **`npm run check`** (`vite build` + **`cargo fmt` / `clippy` / `cargo test --lib`** for `src-tauri`) |
| Release or engine/contract changes | **`npm run check:release`** (full **`cargo test`** including `tests/`) |
| Rust workspace only | **`cargo test --workspace`** |
| Frontend unit only | **`npm run test:unit`** (Vitest) |
| **Core HTTP restart smoke (A1.1a)** | **`npm run test:e2e:core-api-restart`** (requires `cargo build -p oclivenewnew-tauri`; defaults to Mock LLM) |
| **Web preview shell E2E (A1.1b)** | **`npm run build && npm run test:e2e:preview`** (Playwright + `vite preview`; **CI: Ubuntu `frontend` only**). **Windows local:** if the built-in `webServer` times out, run **`npm run preview -- --host 127.0.0.1 --port 4180 --strictPort`** in one terminal, then in another set **`$env:PW_TEST_USE_EXTERNAL='1'`** (PowerShell) and run **`npm run test:e2e:preview`** |

**CI alignment:** **`npm run check:release`** already chains **`npm run test:unit`** and **`npm run verify:ui`** (see root `package.json`); **Playwright (`npm run test:e2e:preview`) is not in `check:release`** and runs on Ubuntu **`frontend` only**. For UI changes, also run **`npm run build && npm run test:e2e:preview`** on Linux/macOS or rely on a green Ubuntu frontend job. Release sign-off also checks CI, CHANGELOG, [compatibility](creator-docs-en/COMPATIBILITY.md), and [versioning](creator-docs-en/development/RELEASE_VERSIONING.md).

**CI:** `.github/workflows/ci.yml` runs Rust + **`npm run build`** + **`npm run test:unit`** on Ubuntu and Windows; **Ubuntu `frontend`** also runs **`npm run test:e2e:preview`**; Ubuntu also runs **OOCP** and **`oclive-cli`** jobs. See root README **Testing**.

## Module ownership (current maintainer)

| Area | Path | Owner | Notes |
|------|------|-------|-------|
| Desktop host | `distros/desktop-tauri/` | @linkaiheng2233-cyber | Tauri IPC, HTTP `--api` |
| Orchestration | `kernel/crates/oclive_kernel_host/src/domain/chat_engine/` | same | `process_message` / `co_present` |
| Kernel crates | `kernel/crates/oclive_kernel_{types,contracts,runtime}` | same | DTOs, traits, runtime |
| Validation | `kernel/crates/oclive_validation` | same | manifest / v2 blueprint |
| CLI | `kernel/crates/oclive-cli` | same | `init`, `bench`, `test`, `doctor` |
| Frontend | `distros/shared/` + `distros/chat-pro/` (Vue workspaces) | same | Pinia, plugin manager, i18n |
| Docs | `creator-docs/`, `handoff/` | same | contracts & release gates |

See **[`handoff/BUS_FACTOR_NOTES.md`](handoff/BUS_FACTOR_NOTES.md)** for entry paths after the kernel crate split.

## Code navigation (by topic)

| Goal | Start here |
|------|------------|
| One message end-to-end | `kernel/crates/oclive_kernel_host/src/domain/chat_engine/process_message.rs` → `turn_pipeline.rs` |
| Multi-instance merge rules | `kernel/crates/oclive_kernel_host/src/domain/slot_runner.rs` |
| Plugin backend resolution | `plugin_host.rs` + `slot_resolver.rs` |
| Blueprint load / save | `infrastructure/storage.rs` + `kernel/crates/oclive_validation` |
| Plugin implementation | `PLUGIN_V1.md` + traits in `oclive_kernel_contracts` |
| Architecture trade-offs | [`creator-docs/architecture/DESIGN_DECISIONS.md`](creator-docs/architecture/DESIGN_DECISIONS.md) |

## Common change scenarios

| Scenario | Touch | Also update |
|----------|-------|-------------|
| New slot type or merge policy | `slot_runner.rs`, `slot_resolver.rs`, `oclive_validation` | `ROLE_PACK_SPEC.md`, frontend graph |
| New plugin backend | `plugin_host.rs`, model enums, `PLUGIN_V1.md` | blueprint / settings docs |
| Co-present stage order | `turn_pipeline.rs` (careful) | `DESIGN_DECISIONS.md`, OOCP tests |
| New DB column | `distros/desktop-tauri/migrations/`, repositories | documented table names only |
| New Tauri command | `distros/desktop-tauri/src/api/`, `lib.rs` handler | `tauri-api.ts`, DTO field names |

## Pull requests

1. **Fork / feature branch**; one PR per concern. Contract changes (manifest, DTO, PLUGIN_V1) need **docs** + **`kernel/crates/oclive_validation`** when applicable.
2. **Description:** motivation, behavior change, risks, manual verification; link issues if any.
3. **Self-check:** at least **`npm run check`**; for persistence / HTTP / orchestration, prefer **`npm run check:release`**; kernel scaffolds may add **`cargo run -p oclive-cli -- --experimental test -o . --json`**.
4. **Review:** module owner (table above) or delegate; CI, security, i18n, and contract docs must stay aligned.
5. **Merge bar:** every required main-CI job is green. `ci-impact-plan` is shadow evidence only; failures in the separate Nightly workflow do not directly block merging but must be tracked. Breaking changes follow [`BREAKING_CHANGE_PROCESS.md`](handoff/BREAKING_CHANGE_PROCESS.md).

### Dimension 5 baseline (before PR / release)

Dimension 5 is defined by `node scripts/dimension5-acceptance.mjs --ci`. **Re-run when touching:**

| Path | ID | Command |
|------|-----|---------|
| `kernel/crates/oclive_kernel_host/src/domain/**` | D-LAYER-01 | `node scripts/check-domain-layering.mjs` |
| `Cargo.lock` / `kernel/crates/oclive_sqlx/**` | D-CI-03 | `node scripts/dimension5-acceptance.mjs --ci` |
| `kernel_ensure_plan_v1.json` / `oclive-cli` ensure | D-VSCODE-02 | `cargo test -p oclive-cli --test kernel_ensure_plan_snapshot` |
| `.github/workflows/ci.yml` | D-CI-01/02 | full `node scripts/dimension5-acceptance.mjs --ci` |
| `CHANGELOG.md` / `CHANGELOG.en.md` | K-DOC-02 | `node scripts/check-changelog-parity.mjs` |
| host re-exports of runtime engines | D-OPUS-05 | `node scripts/check-host-reexport-imports.mjs` |

**Quick release gate:** `node scripts/dimension5-acceptance.mjs --ci` · `node scripts/check-domain-layering.mjs` · `cargo test -p oclive-cli --test kernel_ensure_plan_snapshot`

### When CI fails

| Job | What to do |
|-----|------------|
| `cargo fmt` | Run `cargo fmt --all` locally |
| `cargo clippy` | `cargo clippy --workspace --all-targets --all-features -- -D warnings` |
| `cargo test` (Windows integration) | Trust **Ubuntu CI**; locally try `cargo test --workspace --lib` |
| `frontend` | `npm run test:unit` |
| `oocp-test-suite` | `OCLIVE_HTTP_API_MOCK_LLM=1`, free port; see [OOCP_TEST_SUITE.md](creator-docs/testing/OOCP_TEST_SUITE.md) |
| `cargo audit` inside `dimension5-acceptance` | Run `cargo audit` at repo root and track [KNOWN_VULNERABILITIES.md](creator-docs/security/KNOWN_VULNERABILITIES.md); this is a required gate |
| `npm-audit` | Required high-severity gates cover both production dependencies and the full development graph: run `npm audit --omit=dev --audit-level=high`, `npm audit --audit-level=high`, and use `npm ls` to verify peer relationships. See [KNOWN_VULNERABILITIES.md](creator-docs-en/security/KNOWN_VULNERABILITIES.md) for the current baseline |
| Role packs | `cargo run -p oclive-cli -- pack validate <role>` |

## Breaking changes

**Full process, compatibility expectations, and PR/migration templates:** read **[`handoff/BREAKING_CHANGE_PROCESS.md`](handoff/BREAKING_CHANGE_PROCESS.md)** (engineering discipline §C2; product execution items live in [`handoff/PRODUCT_LINE_TASK_BUCKETS.md`](handoff/PRODUCT_LINE_TASK_BUCKETS.md)).

Summary:

1. **Open an issue** (or RFC for large surface) describing migration impact on role packs, `plugin_backends`, HTTP OOCP / `invoke` DTOs; label the PR **BREAKING**.  
2. **PR must include:** updates to **`kernel/crates/oclive_validation`** (if manifest/settings keys change), **`PLUGIN_V1.md` / `ERROR_CODES.md` / `COMPATIBILITY.md`** as applicable, **`creator-docs/`** / **`creator-docs-en/`** mirrors, and **`CHANGELOG.md` + `CHANGELOG.en.md`** entries.  
3. **Review:** at least one maintainer confirms compatibility shims, migration paths, CI, CHANGELOG, and compatibility docs.

## Documentation

- **User-visible copy:** avoid duplicated hard-coded strings (see [AGENTS.md](AGENTS.md) for the plugin manager entry pattern).
- **Contracts & DB:** follow `distros/chat-pro/roles/README_MANIFEST.md`, `RoleStorage::load_role`, and **`kernel/crates/oclive_validation`**; **do not invent** SQL table names.
- **Doc index:** [creator-docs/getting-started/DOCUMENTATION_INDEX.md](creator-docs/getting-started/DOCUMENTATION_INDEX.md).
- **Releases & compatibility:** on semver bumps or contract changes, review [compatibility](creator-docs-en/COMPATIBILITY.md), [versioning](creator-docs-en/development/RELEASE_VERSIONING.md), and both CHANGELOG files; pack rules stay in [PACK_VERSIONING.md](creator-docs/role-pack/PACK_VERSIONING.md).

## Do not commit

- Secrets, tokens, personal paths; keep `.env` out of git (see `.gitignore`).
- You may delete a legacy **`distros/desktop-tauri/target/`** folder; release bundles live under the **external `target-dir`** `release/bundle/`.

## Discussion & roadmap

For large changes, open an issue or align with monthly roadmap goals to stay consistent with the **runtime vs pack-editor** split.
