# Scheme index

Audit table for `data/ci/schemes/*.json`. Regenerate the rows when adding or editing a
scheme; the validator checks that every scheme file has a row.

| id | status | includes | jobs | author | created_at | derived_from (impact_map / catalog / planner) |
|---|---|---|---|---|---|---|
| `docs` | active | `creator-docs/**`, `creator-docs-en/**`, `dev-notes/**`, `docs/**`, `handoff/**`, `human-docs/**`, `human-docs-en/**` + 11 root docs | dimension5-acceptance, stale-paths | ai | 2026-10-23 | `d0ba836b` / `fd3391d6` / `e2f44fe0` |
| `docs-tooling` | active | `scripts/ai-docs*`, `scripts/check-vendor.mjs`, `scripts/sync-vendor.mjs`, `scripts/vendor/stackmap/**` | dimension5-acceptance, stale-paths, layering-ratchet | ai | 2026-10-23 | `d0ba836b` / `fd3391d6` / `e2f44fe0` |
