# Project current status (fact snapshot)

**SSOT scope**: current versions and candidate delivery status. [RELEASE_VERSIONING](../development/RELEASE_VERSIONING.md) owns versioning rules; MODULE_MAP owns runtime responsibilities. Other entry pages link here rather than duplicating the current-version table.

[中文](../../creator-docs/getting-started/PROJECT_CURRENT_STATUS.md)

**Snapshot date**: 2026-11-01. Update when versions or delivery status change; editing this page does not create new runtime verification.

---

## App and repo version

| Item | Value |
|------|--------|
| Desktop app semver | **0.6.0-rc.4** ([package.json](../../package.json), [Tauri config](../../distros/desktop-tauri/tauri.conf.json), [desktop manifest](../../distros/desktop-tauri/Cargo.toml) agree; a candidate, not a public release) |
| Windows MSI numeric version | **0.6.0.4**, mapped to the desktop candidate independently of CLI/kernel versions |
| CLI / runtime / validation | **0.1.0 / 0.2.0 / 0.1.0**; [CLI](../../kernel/crates/oclive-cli/Cargo.toml), [runtime](../../kernel/crates/oclive_kernel_runtime/Cargo.toml), [validation](../../kernel/crates/oclive_validation/Cargo.toml) manifests |
| Default HTTP API (`--api`) | `http://127.0.0.1:8420` (`GET /health` is public for readiness; all other routes require `OCLIVE_API_TOKEN` by default) |
| User-visible change log | **[CHANGELOG.en.md](../../CHANGELOG.en.md)** (English) · **[CHANGELOG.md](../../CHANGELOG.md)** (Chinese; keep both in sync for each entry) |

---

## What this repo (`oclivenewnew`) delivers

- **2026-11-01 rc.4 patch candidate**: bound retries for Windows access-denied errors while deleting SQLite WAL/SHM files awaiting their final closing handle after a failed migration. Persistent failures retain the primary error and backup; backup reads and database copies do not gain broader retries. Gates, builds and acceptance must bind this candidate's final SHA and package manifest rather than inherit rc.3 results. The maintainer selected an isolated-user-data acceptance path on this computer; a fresh Windows OS remains unverified.

- **2026-10-30 rc.3 internal candidate**: this candidate consolidates the role workspace and later reliability fixes; execution and packaging boundaries are owned by the [rc.3 wave](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/waves/WAVE-20261030-CHATPRO-RC3.md). The old desktop rc.2 directory and ZIP were verified and archived. Installer, bundled kernel and guides must bind the same frozen commit, identified by `PACKAGE-MANIFEST.txt` and `SHA256SUMS.txt`. No public release.

- **Chat Pro internal candidate**: role library → character workspace, separate recent conversations per character, identity/relationship, records, model connection, categorized settings and Help & recovery. See [USER_MANUAL](USER_MANUAL.md) and the [legacy/deferred surface map](../../handoff/MODULE_MAP_AND_HANDOFF.md#chatpro-deferred-surfaces). Installers bundle neither model runners nor weights.
- **Candidate verification boundary**: rc.3 runs its own source gates, isolated install/upgrade and native startup checks; evaluate results against the recorded SHA and package manifest. The prior rc.2 local results belong only to `b4341eb22e987ffef4a19ccad3784e2402cd2bd1` and do not certify this package. Installers are unsigned; a fresh Windows environment and full manual product acceptance remain separate work.
- **Runtime**: Tauri desktop, role-pack import, `process_message`, six slots, directory / Remote plugins, local HTTP `--api`, and the stable Rust in-process facade **`oclive_kernel_host::OcliveKernel`** that starts neither Tauri nor HTTP; see [architecture overview](OCLIVE_ARCHITECTURE_OVERVIEW.md).
- **Included source scope**: workspace resolution, diagnostic authentication redaction and cloud-probe save/lifecycle handling are in the [first follow-up](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/waves/WAVE-20261027-CHATPRO-ROUND-REVIEW.md#review-fixes); role/grant/plugin/snapshot recovery is in the [batched fixes](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/waves/WAVE-20261028-REVIEW-FIXES.md), and RPC/migration/mirror ordering is in the [Host follow-up](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/waves/WAVE-20261029-HOST-RELIABILITY.md). Each wave retains its original SHA-bound evidence; the old rc.2 package does not automatically contain these changes.
- **Kernel programme**: the **K0–K5** code paths are closed and complete in-process turns are verified. **P2 (OTA / remote logs, etc.)** and **V-EMBED-01 Linux/ARM/hardware, resource-budget, and long-soak proof** remain open; evidence: [KERNEL_IMPLEMENTATION_PLAN.md](KERNEL_IMPLEMENTATION_PLAN.md) and root [AGENTS.md](../../AGENTS.md).
- **Release gates**: [CONTRIBUTING](../../CONTRIBUTING.en.md), CI, and the active [TECHNICAL_DEBT_INVENTORY](../../handoff/TECHNICAL_DEBT_INVENTORY.md).

---

## Sister repos and i18n

- **oclive-pack-editor**, **oclive-vscode**, and **oclive-plugin-market** integrate through role packs, plugin contracts, and distro profiles; the launcher is archived.
- **Four-repo UI bilingual baseline**: historical [I18N_FOUR_REPO_BASELINE.md](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/archive/I18N_FOUR_REPO_BASELINE.md).
- **Creator English docs (`creator-docs-en/`)**: closure scope and update rules: [Documentation bilingual closure baseline](../README.md#documentation-bilingual-closure-baseline) in `creator-docs-en/README.md` (Chinese `creator-docs/` remains authoritative; roadmap long reads may stay Chinese-only).

---

## Roadmap and alignment habits

| Need | Doc |
|------|-----|
| Monthly vision | [../../creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md](../../creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md) |
| Experience backlog | [../../creator-docs/roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md](../../creator-docs/roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md) |
| Active engineering debt and freezes | [TECHNICAL_DEBT_INVENTORY](../../handoff/TECHNICAL_DEBT_INVENTORY.md) |

Before release or contract changes: root [README.md](../../README.md) `npm run check` / `check:release`, and update **both** CHANGELOG files for user-visible changes.
