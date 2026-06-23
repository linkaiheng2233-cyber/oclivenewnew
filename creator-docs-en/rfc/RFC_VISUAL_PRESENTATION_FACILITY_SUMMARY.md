# Visual Presentation Facility (#4) — English summary

See full RFC: [RFC_VISUAL_PRESENTATION_FACILITY.md](../../creator-docs/rfc/RFC_VISUAL_PRESENTATION_FACILITY.md) (Chinese SSOT).

## Role

Maps `visual_state_id` → optional `performance_directive` for host UI adapters. **No second LLM.**

## Config

- `config.json` → `visual_presentation.enabled` + `backend` (`image` | `live2d` | `rig3d` | `procedural` | `directory`)
- Distro gating: `distro.oclive.toml` → `[visual_presentation].mode` (`off` | `image_only` | `stage_full`)

## Adapters

| Phase | Adapter |
|-------|---------|
| v0.4 | `image` — PNG/WebP path in directive |
| Theater | `live2d` — `Live2DStageAdapter.vue` (fallback image when Cubism unavailable) |
| Future | `rig3d` / `procedural` / `directory` — see `distros/shared/src/adapters/visual/` |

## DTO

`SendMessageResponse.performance_directive` optional; `enabled: false` omits field.
