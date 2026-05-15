# Contributing to oclive

[中文](CONTRIBUTING.md)

Thank you for helping improve oclive. High-level goals are described in [creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md](creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md).

## GitHub (CI, Dependabot, branch protection)

After merges to the default branch, **Dependabot** opens PRs per [`.github/dependabot.yml`](.github/dependabot.yml). **CI** runs on Actions. For org/repo settings (branch protection, secrets), see **[creator-docs/getting-started/GITHUB_REPO_CHECKLIST.md](creator-docs/getting-started/GITHUB_REPO_CHECKLIST.md)**.

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
- **Vue / TypeScript:** Match existing composables/stores; align with Tauri DTO field names (e.g. **`reply`** in [`src-tauri/src/models/dto.rs`](src-tauri/src/models/dto.rs)).

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
| **Core HTTP restart smoke (A1.1 PoC)** | **`npm run test:e2e:core-api-restart`** (requires `cargo build -p oclivenewnew-tauri`; defaults to `OCLIVE_HTTP_API_MOCK_LLM=1`) |

**CI alignment:** **`npm run check:release` does not run `npm run test:unit`**; CI runs **`npm run test:unit`** in the **`frontend`** job. Before a release, run **`npm run test:unit`** locally or rely on a green **Actions → frontend** run. Full release gates: [handoff/PRODUCT_RELEASE_CHECKLIST.md](handoff/PRODUCT_RELEASE_CHECKLIST.md).

**CI:** `.github/workflows/ci.yml` runs Rust + **`npm run build`** + **`npm run test:unit`** on Ubuntu and Windows; Ubuntu also runs **OOCP** and **`oclive-cli`** jobs. See root README **Testing**.

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

## Do not commit

- Secrets, tokens, personal paths; keep `.env` out of git (see `.gitignore`).
- You may delete a legacy **`src-tauri/target/`** folder; release bundles live under the **external `target-dir`** `release/bundle/`.

## Discussion & roadmap

For large changes, open an issue or align with monthly roadmap goals to stay consistent with the **runtime vs pack-editor** split.
