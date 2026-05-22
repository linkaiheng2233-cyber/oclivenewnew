# Project overview (three repos, one mental model)

This page collects **how the three repos split work**, what is **already shipped in this repo**, **where to read next**, **common commands**, **human vs automation roles**, and **what is still backlog**. Details live in topic docs.

> **Kernel-centric architecture diagram** (kernel in the center, modules around it; static asset + Mermaid): [KERNEL_AND_MODULES_ARCHITECTURE.md](KERNEL_AND_MODULES_ARCHITECTURE.md). Links onward to [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md) and module-specific pages.

---

## 1. What each repository is

| Repository | Role | Stack (high level) |
|--------------|------|--------------------|
| **oclivenewnew** (this repo) | **Runtime:** player chat, pack loading, engine, Tauri desktop | Rust + Vue + Tauri |
| **oclive-pack-editor** (separate clone, often sibling folder) | **Authoring tool:** edit / export `roles/{id}/` or zip | Vue + Tauri (**different** `package.json` from runtime) |
| **oclive-launcher** (separate repo) | **Launcher:** paths, starts runtime & editor; **environment & troubleshooting** | Vue + Tauri |

**The only contract between them:** the on-disk **role pack** (same layout as `roles/{roleId}/`). Runtime and editor meet through **import/export** or **`OCLIVE_ROLES_DIR`**, not heavy cross-process IPC.

---

## 2. What this repo already covers (summary)

- **Role packs (v2):** `pipeline.ocblueprint` SSOT (`meta`, `slot_registry`, optional `groups`); import `.ocpak` / `.zip` / folders. Legacy `manifest.json`+`settings.json` — migration only ([V1_TO_V2_MIGRATION.md](../../creator-docs/role-pack/V1_TO_V2_MIGRATION.md)).
- **Tooling:** `npm run check` (daily), `npm run check:release` (before release); Rust fmt / clippy / `cargo test`.
- **CI:** GitHub Actions on **Ubuntu + Windows** for Rust and `npm run build` (see `.github/workflows/ci.yml`).
- **Docs:** `creator-docs/`, `roles/README_MANIFEST.md`, import checklist `roles/TESTING_ROLE_PACK_IMPORT.md`, roadmap & backlog pages.

---

## 3. Documentation map (where to start)

| Need | Document |
|------|----------|
| **Master index (ZH)** | [DOCUMENTATION_INDEX.md](../../creator-docs/getting-started/DOCUMENTATION_INDEX.md) |
| **Current status (version, ships, bilingual CHANGELOG)** | [PROJECT_CURRENT_STATUS.md](PROJECT_CURRENT_STATUS.md) |
| **Status snapshot & goal alignment (kernel + product + doc map by purpose)** | [PROJECT_STATUS_AND_ALIGNMENT.md](PROJECT_STATUS_AND_ALIGNMENT.md) |
| **Kernel & six-slot diagram** | [KERNEL_AND_MODULES_ARCHITECTURE.md](KERNEL_AND_MODULES_ARCHITECTURE.md) |
| **Creator: from pack to oclive** | [CREATOR_WORKFLOW.md](../../creator-docs/getting-started/CREATOR_WORKFLOW.md) |
| **manifest / import** | [roles/README_MANIFEST.md](../../roles/README_MANIFEST.md) |
| **Personality archive design** | [docs/personality-archive-notes.md](../../docs/personality-archive-notes.md) |
| **Design axis evolution** | [docs/design-axis-evolution.md](../../docs/design-axis-evolution.md) |
| **Import manual test list** | [roles/TESTING_ROLE_PACK_IMPORT.md](../../roles/TESTING_ROLE_PACK_IMPORT.md) |
| **Monthly roadmap** | [VISION_ROADMAP_MONTHLY.md](../../creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md) |
| **Experience backlog** | [BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md](../../creator-docs/roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md) |
| **Build, CI, release bar, Sentry** | Root [README.md](../../README.md) / [README.en.md](../../README.en.md) |
| **Contributing** | [CONTRIBUTING.md](../../CONTRIBUTING.md) / [CONTRIBUTING.en.md](../../CONTRIBUTING.en.md) |

---

## 4. Common commands (repo root)

| Command | Purpose |
|---------|---------|
| `npm run dev` / `npm run tauri:dev` | Local development |
| `npm run check` | Before daily PRs: `vite build` + `cargo fmt` / `clippy` / **`cargo test --lib`** |
| `npm run check:release` | **Before release or engine changes:** full **`cargo test`** (matches CI Rust job) |
| `npm run check:rust:test:all` | Rust tests only, full |

---

## 5. Who does what

### You on your machine

- **Git:** `clone` / `pull` / **push**, then confirm **Actions** is green.
- **LNK1104** (Windows linker): close locking processes, `cargo test -j 1`, etc. — environment-specific.
- **Release decisions:** version numbers, `CHANGELOG`, optional `VITE_SENTRY_DSN`, installer signing, release notes.
- **Smoke:** install → launch → chat → import pack; editor export → oclive import (use the checklist).

### Good fit for dev / AI collaboration

- Features & refactors, docs & CI scripts, backlog tasks, fixing failures and adding tests.

---

## 6. Compared to the vision: backlog / in flight

Not “missing quality” — **phase and scheduling are product choices**:

- In-editor quick chat, launcher one-click Ollama/model, marketplace → see [BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md](../../creator-docs/roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md).
- Contract depth, `min_runtime`, editor vs `load_role` parity → [VISION_ROADMAP_MONTHLY.md](../../creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md) and pack/editor docs.

---

## 7. CI in the other two repos

If you have **oclive-pack-editor** and **oclive-launcher** checked out, each has `.github/workflows/ci.yml` (dual platform; editor also runs Vitest/E2E on Linux). **Push** and check **Actions** in those repos.

---

## 8. Minimal pre-release checklist

1. Local: `npm run check:release` (or `check` + the test scope you accept).  
2. After pushing all involved repos: **CI green**.  
3. Version numbers and `CHANGELOG` updated.  
4. Smoke per `TESTING_ROLE_PACK_IMPORT.md` or equivalent.  
5. License files present for official plugins (see [LICENSE_POLICY.md](../../creator-docs/LICENSE_POLICY.md)).  
6. Distribution story clear: with **no Tauri online updater** configured, ship **offline installers** (see root README).

> Quick license presence: `npm run check:license`.

---

*If this page disagrees with a topic doc or the code, prefer the topic doc + repository code; update this page or `CHANGELOG.md` for major drift.*

---

[中文](../../creator-docs/getting-started/PROJECT_OVERVIEW.md)
