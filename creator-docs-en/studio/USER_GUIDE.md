# oclive Studio User Guide

**oclive Studio** (`oclive-studio`) combines the former launcher and pack editor: **Launch mode** and **Create mode** in one app, linked to the **oclivenewnew** runtime via a shared **roles root** on disk.

## Install and bundled zip

- Studio-only: download from [oclive-studio Releases](https://github.com/oclive-app/oclive-studio/releases).
- **Bundled zip** (studio + runtime): run `scripts/package-studio-release.ps1` (Windows) or `scripts/package-studio-release.sh` at the repo root.

## Configuration (`studio-config.json`)

Primary file: **`studio-config.json`** under the app data directory.

Key fields: `rolesDir` (`OCLIVE_ROLES_DIR`), `ocliveExe` / `ocliveProjectRoot`, `ocliveLlmMode`, Remote LLM URLs, `lastMode`.

**Compatibility**: legacy `launcher-config.json` is migrated once to `studio-config.json`. If both exist, **studio-config.json wins**.

## Launch mode

Configure roles root and LLM, run **environment diagnostics** (doctor-style checks), then **Start oclive**.

## Create mode

Lazy-loaded at `/create`: edit packs, validate, export to roles root, **trial chat** via `--api` with config from `studio-config.json`.

## Onboarding

Three-step wizard on first run; storage key `studio.onboarding.completed`. **Show onboarding again** from the top bar.

## Deep links

- `oclive-studio://create` → Create mode  
- `oclive-studio://create?roleId=xxx` → Create mode with role hint  

## See also

- [CREATOR_WORKFLOW.md](../getting-started/CREATOR_WORKFLOW.md)
- [RFC_STUDIO_MERGE.md](../rfc/RFC_STUDIO_MERGE.md)
