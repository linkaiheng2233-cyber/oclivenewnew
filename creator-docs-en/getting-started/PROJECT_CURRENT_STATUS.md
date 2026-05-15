# Project current status (fact snapshot)

**Purpose**: a **short, checkable** snapshot for collaborators and release hygiene (version, what ships in this repo, kernel vs product gates, where user-facing changes are logged). **Does not replace** verification detail in [KERNEL_IMPLEMENTATION_PLAN.md](KERNEL_IMPLEMENTATION_PLAN.md) or the classified doc map in [PROJECT_STATUS_AND_ALIGNMENT.md](PROJECT_STATUS_AND_ALIGNMENT.md).

[中文](../../creator-docs/getting-started/PROJECT_CURRENT_STATUS.md)

**Snapshot date**: 2026-05-15 (update this page’s opening paragraph and date on major milestones or version bumps)

---

## App and repo version

| Item | Value |
|------|--------|
| Desktop app semver | **0.2.0** (align `package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`) |
| Default HTTP API (`--api`) | `http://127.0.0.1:8420` (`GET /health`) |
| User-visible change log | **[CHANGELOG.en.md](../../CHANGELOG.en.md)** (English) · **[CHANGELOG.md](../../CHANGELOG.md)** (Chinese; keep both in sync for each entry) |

---

## What this repo (`oclivenewnew`) delivers

- **Runtime**: Tauri desktop; role pack import (`.ocpak` / `.zip` / folder); chat orchestration **`process_message`**; six-slot `plugin_backends`; directory plugins; remote sidecar; local HTTP `--api`; startup health checks (see [PROJECT_OVERVIEW.md](../../creator-docs/getting-started/PROJECT_OVERVIEW.md)).
- **Kernel programme**: milestones **K0–K5** are closed in plan except **P2 (OTA / remote logs, etc.)**; verification and CI: [KERNEL_IMPLEMENTATION_PLAN.md](KERNEL_IMPLEMENTATION_PLAN.md) and root [AGENTS.md](../../AGENTS.md).
- **Product “first launch” hard gates**: still governed by [PRODUCT_AND_KERNEL_GAP_CHECKLIST.md](../../handoff/PRODUCT_AND_KERNEL_GAP_CHECKLIST.md) **§A**, decoupled from kernel milestones per **§D** of that checklist.

---

## Sister repos and i18n

- **oclive-pack-editor**, **oclive-launcher**, **oclive-plugin-market**: integrate via on-disk packs and shared docs; see [PROJECT_OVERVIEW.md](../../creator-docs/getting-started/PROJECT_OVERVIEW.md).
- **Four-repo UI bilingual baseline**: [I18N_FOUR_REPO_BASELINE.md](../../handoff/I18N_FOUR_REPO_BASELINE.md).
- **Creator English docs (`creator-docs-en/`)**: closure scope and update rules: [Documentation bilingual closure baseline](../README.md#documentation-bilingual-closure-baseline) in `creator-docs-en/README.md` (Chinese `creator-docs/` remains authoritative; roadmap long reads may stay Chinese-only).

---

## Roadmap and alignment habits

| Need | Doc |
|------|-----|
| Monthly vision | [../../creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md](../../creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md) |
| Experience backlog | [../../creator-docs/roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md](../../creator-docs/roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md) |
| One page: progress + goals + doc map by purpose | [PROJECT_STATUS_AND_ALIGNMENT.md](PROJECT_STATUS_AND_ALIGNMENT.md) |

Before release or contract changes: root [README.md](../../README.md) `npm run check` / `check:release`, and update **both** CHANGELOG files for user-visible changes.
