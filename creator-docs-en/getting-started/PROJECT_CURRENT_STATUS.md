# Project current status (fact snapshot)

**SSOT scope**: current versions and candidate delivery status. [RELEASE_VERSIONING](../development/RELEASE_VERSIONING.md) owns versioning rules; MODULE_MAP owns runtime responsibilities. Other entry pages link here rather than duplicating the current-version table.

[中文](../../creator-docs/getting-started/PROJECT_CURRENT_STATUS.md)

**Snapshot date**: 2026-10-29. Update when versions or delivery status change; editing this page does not create new runtime verification.

---

## App and repo version

| Item | Value |
|------|--------|
| Desktop app semver | **0.6.0-rc.2** ([package.json](../../package.json), [Tauri config](../../distros/desktop-tauri/tauri.conf.json), [desktop manifest](../../distros/desktop-tauri/Cargo.toml) agree; a candidate, not a public release) |
| Windows MSI numeric version | **0.6.0.2**, mapped to the desktop candidate independently of CLI/kernel versions |
| CLI / runtime / validation | **0.1.0 / 0.2.0 / 0.1.0**; [CLI](../../kernel/crates/oclive-cli/Cargo.toml), [runtime](../../kernel/crates/oclive_kernel_runtime/Cargo.toml), [validation](../../kernel/crates/oclive_validation/Cargo.toml) manifests |
| Default HTTP API (`--api`) | `http://127.0.0.1:8420` (`GET /health` is public for readiness; all other routes require `OCLIVE_API_TOKEN` by default) |
| User-visible change log | **[CHANGELOG.en.md](../../CHANGELOG.en.md)** (English) · **[CHANGELOG.md](../../CHANGELOG.md)** (Chinese; keep both in sync for each entry) |

---

## What this repo (`oclivenewnew`) delivers

- **2026-10-28–29 source follow-up**: check the [batch execution record](../../handoff/debt-marathon/waves/WAVE-20261028-REVIEW-FIXES.md) for role file/database recovery, grant persistence, plugin input boundaries, web-install consent, authenticated snapshots and rendering recovery, and the [Host reliability continuation](../../handoff/debt-marathon/waves/WAVE-20261029-HOST-RELIABILITY.md) for synchronous RPC, migration recovery diagnostics and mirror ordering. Each record owns its verification status. No installer was rebuilt or published; the existing candidate below does not contain these repairs. Source tests do not prove a new installer or remote CI.

- **Chat Pro internal candidate**: role library → character workspace, separate recent conversations per character, identity/relationship, records, model connection, categorized settings and Help & recovery. See [USER_MANUAL](USER_MANUAL.md) and the [legacy/deferred surface map](../../handoff/MODULE_MAP_AND_HANDOFF.md#chatpro-deferred-surfaces). Installers bundle neither model runners nor weights.
- **Candidate verification boundary**: local tag `baseline/chatpro-0.6.0-rc.2-internal-20261027` binds `b4341eb22e987ffef4a19ccad3784e2402cd2bd1`. Release/integration gates, browser regressions and isolated installation/startup passed locally for that source. The candidate is unsigned and unpublished; remote CI for that commit, fresh Windows compatibility and real model chat using this new package remain unverified. Later documentation-only commits do not claim to rerun those program tests.
- **Runtime**: Tauri desktop, role-pack import, `process_message`, six slots, directory / Remote plugins, local HTTP `--api`, and the stable Rust in-process facade **`oclive_kernel_host::OcliveKernel`** that starts neither Tauri nor HTTP; see [architecture overview](OCLIVE_ARCHITECTURE_OVERVIEW.md).
- **Source repairs after that candidate**: fix internal workspace installation resolution, diagnostic authentication redaction, and cloud-probe save/lifecycle boundaries. Execution evidence and remaining limits are in the [review follow-up](../../handoff/debt-marathon/waves/WAVE-20261027-CHATPRO-ROUND-REVIEW.md#review-fixes). The existing rc.2 installer above remains bound to `b4341eb2` and does not automatically contain these later changes.
- **Kernel programme**: the **K0–K5** code paths are closed and complete in-process turns are verified. **P2 (OTA / remote logs, etc.)** and **V-EMBED-01 Linux/ARM/hardware, resource-budget, and long-soak proof** remain open; evidence: [KERNEL_IMPLEMENTATION_PLAN.md](KERNEL_IMPLEMENTATION_PLAN.md) and root [AGENTS.md](../../AGENTS.md).
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
