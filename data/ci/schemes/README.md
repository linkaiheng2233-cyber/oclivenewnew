# CI schemes

Named, machine-validated sets of CI jobs for a path pattern.

A scheme is **not** a second planner. It is a frozen copy of what the deterministic
planner already selects for the paths it covers. That property is what lets an AI add
schemes without weakening coverage: the validator rejects any scheme whose job set is
not exactly what the planner would pick.

## File shape

```json
{
  "id": "docs-tooling",
  "title": "Documentation tooling",
  "status": "draft",
  "includes": ["scripts/ai-docs.mjs", "handoff/catalog/**"],
  "jobs": ["dimension5-acceptance", "stale-paths", "layering-ratchet"],
  "derived_from": {
    "impact_map": "<git blob sha of data/ci/impact-map.v1.json>",
    "catalog": "<git blob sha of data/ci/validation-catalog.v1.json>",
    "planner": "<git tree sha of kernel/crates/oclive_ci_plan>"
  },
  "evidence": { "replay_commits": 0, "divergences": 0 },
  "author": "ai",
  "created_at": "2026-10-22T16:45:00+08:00",
  "rationale": "why these paths belong together"
}
```

`status` is `draft` until the planner-equality check (Phase B) enters CI; a draft scheme
is documentation only and must not change selection.

## Invariants (enforced by `scripts/check-ci-schemes.mjs`)

1. **Equal to the planner** (Phase B, trusted-base binary): `jobs` equals the planner's
   selected validators for the union of `includes`. Phase A enforces the static subset.
2. **No control-plane paths**: an include may not overlap any `risk_overrides` selector
   (`ci.yml`, `ci-rerun-flake.yml`, `data/ci/**`, `Cargo.toml`, `Cargo.lock`,
   `package.json`, `package-lock.json`, the planner crates, `ci-execution-policy.mjs`,
   `collect-ci-compare-evidence.mjs`, `dimension5-acceptance.mjs`,
   `check-domain-layering.mjs`).
3. **Fresh derivation**: `derived_from` must match the current blobs; when the catalog or
   impact map changes, old schemes fail validation until re-derived.
4. **Real paths and jobs**: every include resolves to at least one tracked path and every
   job exists in `data/ci/validation-catalog.v1.json`.

## Governance

- **AI may add or edit schemes without human approval.** Every change must:
  - keep the four invariants green;
  - append a row to `INDEX.md`;
  - include the generated notice block in the PR body;
  - use an `ai-scheme:` commit prefix.
- **`data/ci/impact-map.v1.json` and the validation catalog are different**: an AI may
  propose changes there, but a human approves them in a separate "registry revision" PR.
- The plan currently running a PR is computed from the **base** commit's data, so a scheme
  added by a PR cannot shorten that same PR's checks.

## Runtime authorisation

A ready PR runs selectively only when the planner's selected workflow-job set **equals** an
`active` scheme's `jobs`. The hard-coded module whitelist was removed in Phase C1: this list
is now the single promotion source. A `draft` scheme is documentation only.

## Why subset use is safe

The planner unions modules: adding paths only adds checks. A scheme derived from the full
include set therefore runs a superset of what any subset of those paths would require.
