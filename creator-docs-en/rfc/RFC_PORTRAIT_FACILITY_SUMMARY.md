# Portrait Facility (#3) — English summary

See full RFC: [RFC_PORTRAIT_FACILITY.md](../../creator-docs/rfc/RFC_PORTRAIT_FACILITY.md) (Chinese SSOT).

## Disk layout (A2)

- `config.json` → `portrait_catalog.enabled` (boolean)
- `portrait_catalog.json` → `schema_version` + `assets[]` (closed id set)
- Simple packs export **7 fixed ids** (`happy_default`, …, `shy_default`) with B1 defaults (`enabled: true`)

## Runtime

- **Portrait Director** (Phase 3): LLM picks a catalog `id` using dialogue + complex emotion `narrative_hint`
- Response adds optional `visual_state_id`; legacy `portrait_emotion` tag retained
- CoPresent: rule maps `bot_emotion` → catalog tag → id (no director LLM by default)
- Legacy packs without catalog behave as v0.3 filename heuristics

## Validation

- `oclive pack validate`: unique ids, safe paths, 7-slot coverage when enabled
