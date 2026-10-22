# Scheme index

Audit table for `data/ci/schemes/*.json`. Regenerate the rows when adding or editing a
scheme; the validator checks that every scheme file has a row.

| id | status | includes | jobs | author | created_at | derived_from (impact_map / catalog / planner) |
|---|---|---|---|---|---|---|
| `docs-tooling` | draft | `scripts/ai-docs*`, `scripts/check-vendor.mjs`, `scripts/sync-vendor.mjs`, `scripts/vendor/stackmap/**`, `handoff/catalog/**` | dimension5-acceptance, stale-paths, layering-ratchet | ai | 2026-10-22 | `847ffcd1` / `fd3391d6` / `a0de9a9e` |
