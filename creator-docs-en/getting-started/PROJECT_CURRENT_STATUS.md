# Project current status (fact snapshot)

**Purpose**: a short, checkable snapshot of versions, shipped surfaces, and changelog entry points. Use [DOCUMENTATION_INDEX](DOCUMENTATION_INDEX.md) to find topic documentation.

[中文](../../creator-docs/getting-started/PROJECT_CURRENT_STATUS.md)

**Snapshot date**: 2026-08-20 (update this page’s opening paragraph and date on major milestones or version bumps)

---

## App and repo version

| Item | Value |
|------|--------|
| Desktop app semver | **0.5.0** (align `package.json`, `distros/desktop-tauri/tauri.conf.json`, `distros/desktop-tauri/Cargo.toml`) |
| Default HTTP API (`--api`) | `http://127.0.0.1:8420` (`GET /health` is public for readiness; all other routes require `OCLIVE_API_TOKEN` by default) |
| User-visible change log | **[CHANGELOG.en.md](../../CHANGELOG.en.md)** (English) · **[CHANGELOG.md](../../CHANGELOG.md)** (Chinese; keep both in sync for each entry) |

---

## What this repo (`oclivenewnew`) delivers

- **Runtime**: Tauri desktop, role-pack import, `process_message`, six slots, directory / Remote plugins, and local HTTP `--api`; see [architecture overview](OCLIVE_ARCHITECTURE_OVERVIEW.md).
- **Kernel programme**: milestones **K0–K5** are closed in plan except **P2 (OTA / remote logs, etc.)**; verification and CI: [KERNEL_IMPLEMENTATION_PLAN.md](KERNEL_IMPLEMENTATION_PLAN.md) and root [AGENTS.md](../../AGENTS.md).
- **Release gates**: [CONTRIBUTING](../../CONTRIBUTING.en.md), CI, and the active [TECHNICAL_DEBT_INVENTORY](../../handoff/TECHNICAL_DEBT_INVENTORY.md).

---

## Sister repos and i18n

- **oclive-pack-editor**, **oclive-vscode**, and **oclive-plugin-market** integrate through role packs, plugin contracts, and distro profiles; the launcher is archived.
- **Four-repo UI bilingual baseline**: historical [I18N_FOUR_REPO_BASELINE.md](../../handoff/archive/I18N_FOUR_REPO_BASELINE.md).
- **Creator English docs (`creator-docs-en/`)**: closure scope and update rules: [Documentation bilingual closure baseline](../README.md#documentation-bilingual-closure-baseline) in `creator-docs-en/README.md` (Chinese `creator-docs/` remains authoritative; roadmap long reads may stay Chinese-only).

---

## Roadmap and alignment habits

| Need | Doc |
|------|-----|
| Monthly vision | [../../creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md](../../creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md) |
| Experience backlog | [../../creator-docs/roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md](../../creator-docs/roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md) |
| Active engineering debt and freezes | [TECHNICAL_DEBT_INVENTORY](../../handoff/TECHNICAL_DEBT_INVENTORY.md) |

Before release or contract changes: root [README.md](../../README.md) `npm run check` / `check:release`, and update **both** CHANGELOG files for user-visible changes.
