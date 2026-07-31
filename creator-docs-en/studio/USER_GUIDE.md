# A.I.Live Studio User Guide

[中文](../../creator-docs/studio/USER_GUIDE.md)

**A.I.Live Studio** (repository **`oclive-studio`**, codename unchanged) combines the former launcher and pack editor: **Launch mode** and **Create mode** in one app, linked to the **oclivenewnew** runtime via a shared **roles root** on disk.

## Install and bundled zip

- Studio-only: download from [oclive-studio Releases](https://github.com/linkaiheng2233-cyber/oclive-studio/releases).
- **Bundled zip** (studio + runtime): run `scripts/package-studio-release.ps1` (Windows) or `scripts/package-studio-release.sh` at the repo root.

## Configuration (`studio-config.json`)

Primary file: **`studio-config.json`** under the app data directory.

Key fields: `rolesDir` (`OCLIVE_ROLES_DIR`), `ocliveExe` / `ocliveProjectRoot`, `ocliveLlmMode`, Remote LLM URLs, `lastMode`.

**Compatibility**: legacy `launcher-config.json` is migrated once to `studio-config.json`. If both exist, **studio-config.json wins**.

## Launch mode

Configure roles root and LLM, run **environment diagnostics** (doctor-style checks), then **Start A.I.Live**.

## Create mode

Lazy-loaded at `/create`: edit packs, validate, export to roles root, **trial chat** via `--api` with config from `studio-config.json`.

### Editing workflow (Create mode)

1. **Pick or create a role pack** under `distros/chat-pro/roles/<roleId>/` with **`pipeline.ocblueprint`** plus assets per [ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md). New packs use Stable v4; v2 remains compatible. **Legacy v1 (deprecated):** `manifest.json` / `settings.json` — [V1_TO_V2_MIGRATION.md](../role-pack/V1_TO_V2_MIGRATION.md).
2. **Edit blueprint** — `meta` and **`slot_registry`** (`type` + `backend` per instance); run **Validate** (`oclive pack validate`, exact v2 / v3 / v4 dispatch).
3. **Resources & scenes** — `core_personality.txt`, scene folders; **architecture graph** shows `slot_registry` and optional **`groups`** (read-only grouping).
4. **Save** to the configured roles root; optional hot reload when `ocliveProjectRoot` is set.

### Trial chat (test a role pack)

1. Open the pack in Create mode and click **Trial chat**.
2. Studio spawns **oclivenewnew `--api`** with `OCLIVE_ROLES_DIR` and LLM settings from `studio-config.json`.
3. Send messages in the panel; on failure, check **Environment diagnostics** and `RUST_LOG=info` on the runtime process.
4. After blueprint or scene changes, trial chat again (API process may restart automatically).

### Export a role pack

1. Ensure **validation passes** (no blocking `pipeline.ocblueprint` / asset errors).
2. **Export** / **Publish to roles root** — copies the folder into `rolesDir` from config (confirm before overwrite).
3. Optional **pack** step produces `.oclive-plugin` or zip with SHA-256 summary; use **Launch mode** to chat with the exported pack.

## Onboarding

Three-step wizard on first run; storage key `studio.onboarding.completed`. **Show onboarding again** from the top bar.

## Deep links

- `oclive-studio://create` → Create mode  
- `oclive-studio://create?roleId=xxx` → Create mode with role hint  

## See also

- [CREATOR_WORKFLOW.md](../getting-started/CREATOR_WORKFLOW.md)
- [RFC_STUDIO_MERGE.md](../rfc/RFC_STUDIO_MERGE.md)
