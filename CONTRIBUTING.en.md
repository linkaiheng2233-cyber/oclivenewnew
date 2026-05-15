# Contributing to oclive

[中文](CONTRIBUTING.md)

Thank you for helping improve oclive. High-level goals are described in [creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md](creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md).

## GitHub (CI, Dependabot, branch protection)

After merges to the default branch, **Dependabot** opens PRs per [`.github/dependabot.yml`](.github/dependabot.yml). **CI** runs on Actions. For org/repo settings (branch protection, secrets), see **[creator-docs/getting-started/GITHUB_REPO_CHECKLIST.md](creator-docs/getting-started/GITHUB_REPO_CHECKLIST.md)**.

## Getting help

- **General questions, install, and configuration:** use [**GitHub Issues**](https://github.com/linkaiheng2233-cyber/oclivenewnew/issues) with the **Bug / Feature / Support** templates; prefer `[bug]:` / `[feat]:` / `[support]:` title prefixes (see root [README.en.md](README.en.md) **Support**). First triage is usually within **3–5 business days** (best effort, not an SLA).  
- **Self-serve:** [FAQ](creator-docs/FAQ.md) · [Documentation index](creator-docs/getting-started/DOCUMENTATION_INDEX.md) · [ERROR_CODES](creator-docs/getting-started/ERROR_CODES.md).  
- **Security vulnerabilities:** do **not** disclose in public issues — see [SECURITY.md](SECURITY.md).

## Development environment

- **This repo:** **Node.js** (18+ recommended), **npm**, **Rust** stable, **Ollama** (optional for local dialogue).
- **Windows:** **Visual Studio Build Tools** (MSVC linker).
- **After clone:** run **`npm install`** at the repo root; **`npm run tauri:dev`** drives the Tauri + `src-tauri` build.
- **Rust workspace only** (`oclive_validation`, `oclive-cli`, `oclivenewnew-tauri`): **`cargo test --workspace`** from the root, or **`cargo test --manifest-path src-tauri/Cargo.toml`** for the desktop crate only.
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
  - **Format:** `cargo fmt`; CI uses **`npm run check:rust:fmt`** (`cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check`).
  - **Clippy:** Root **[`Cargo.toml`](Cargo.toml)** defines **`[workspace.lints.rust]`** and **`[workspace.lints.clippy]`**. Local + CI: **`cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`** (**`npm run check:rust:clippy`**): **warnings are errors**.
  - **`unwrap` / `expect`:** Prefer **`Result` / `Option` + `context`** in product code; integration tests may use **`#![allow(clippy::unwrap_used, clippy::expect_used)]`** at the crate root. Do not widen allows elsewhere.
- **Vue / TypeScript:** Match existing composables/stores; align with Tauri DTO field names (e.g. **`reply`**, defined in **`oclive_kernel_runtime`** models and re-exported from `src-tauri/src/models/mod.rs`).

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

**CI alignment:** **`npm run check:release` does not run `npm run test:unit`**; CI runs **`npm run test:unit`** and **`npm run build`** on **Ubuntu and Windows**; **Playwright (`npm run test:e2e:preview`) runs on Ubuntu `frontend` only** (Windows `frontend` skips it). Before a release, run **`npm run test:unit`** locally; if you touched the UI, on **Linux/macOS** run **`npm run build && npm run test:e2e:preview`**, or rely on a green **Actions → frontend (ubuntu)** run. Full release gates: [handoff/PRODUCT_RELEASE_CHECKLIST.md](handoff/PRODUCT_RELEASE_CHECKLIST.md).

**CI:** `.github/workflows/ci.yml` runs Rust + **`npm run build`** + **`npm run test:unit`** on Ubuntu and Windows; **Ubuntu `frontend`** also runs **`npm run test:e2e:preview`**; Ubuntu also runs **OOCP** and **`oclive-cli`** jobs. See root README **Testing**.

## Pull requests

1. **Fork / feature branch**; one PR per concern. Contract changes (manifest, DTO, PLUGIN_V1) need **docs** + **`crates/oclive_validation`** when applicable.
2. **Description:** motivation, behavior change, risks, manual verification; link issues if any.
3. **Self-check:** at least **`npm run check`**; for persistence / HTTP / orchestration, prefer **`npm run check:release`**.
4. **Review:** CI green, security, user-visible copy; large features should align with the roadmap (issue first).

## Breaking changes

1. **Open an issue** (or RFC for large surface) describing migration impact on role packs, `plugin_backends`, HTTP `/chat`, or Tauri DTOs.  
2. **PR must include:** updates to **`crates/oclive_validation`** (if manifest/settings keys change), **`creator-docs/`** / **`creator-docs-en/`** mirrors when applicable, and **`CHANGELOG.md` + `CHANGELOG.en.md`** entries.  
3. **Reviewer:** at least one maintainer checks CI + [PRODUCT_RELEASE_CHECKLIST.md](handoff/PRODUCT_RELEASE_CHECKLIST.md) P0 rows touched by the change.

## Documentation

- **User-visible copy:** avoid duplicated hard-coded strings (see [AGENTS.md](AGENTS.md) for the plugin manager entry pattern).
- **Contracts & DB:** follow `roles/README_MANIFEST.md`, `RoleStorage::load_role`, and **`crates/oclive_validation`**; **do not invent** SQL table names.
- **Doc index:** [creator-docs/getting-started/DOCUMENTATION_INDEX.md](creator-docs/getting-started/DOCUMENTATION_INDEX.md).
- **Releases & compatibility:** on semver bumps or contract changes, review [`creator-docs/COMPATIBILITY.md`](creator-docs/COMPATIBILITY.md) snapshots and the one-pager table, and walk [handoff/PRODUCT_RELEASE_CHECKLIST.md](handoff/PRODUCT_RELEASE_CHECKLIST.md) “对外说明”; pack rules stay in [PACK_VERSIONING.md](creator-docs/role-pack/PACK_VERSIONING.md).

## Do not commit

- Secrets, tokens, personal paths; keep `.env` out of git (see `.gitignore`).
- You may delete a legacy **`src-tauri/target/`** folder; release bundles live under the **external `target-dir`** `release/bundle/`.

## Discussion & roadmap

For large changes, open an issue or align with monthly roadmap goals to stay consistent with the **runtime vs pack-editor** split.
