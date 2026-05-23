# Role pack format (ROLE_PACK_SPEC)

> **Boundary (required):** **Role packs contain only identity, personality, relations, and prompt content. System configuration (slots, backends, models, interaction mode, dual-core, etc.) is owned by the blueprint.** See **[handoff/ROLE_PACK_BOUNDARY.md](../../handoff/ROLE_PACK_BOUNDARY.md)** and **[SETTINGS_REFERENCE.md](../cli/SETTINGS_REFERENCE.md)**.

**Author learning path:** [CREATOR_LEARNING_PATH.md](CREATOR_LEARNING_PATH.md)

This specification describes on-disk role packs **aligned with the A.I.Live host loader** (desktop Tauri, headless `kernel_server`, launchers). Engineering codename **oclive**; authoritative detail remains in source and linked docs.

[中文全文](../role-pack/ROLE_PACK_SPEC.md)

---

## 0. Role pack vs blueprint

| Component | Entry-level creators | Blueprint / admins |
|-----------|----------------------|-------------------|
| **Role pack** | `meta` identity, **`personality`**, **`relations`**, **`prompts/`**, scene prose | — |
| **Blueprint** | **Do not edit** unless you integrate hosts | **`slot_registry`**, **`groups`**, **`backend`**, **`model`**, **`interaction_mode`**, **`memory_config`**, **`runtime_config.dual_core.enabled`** (RFC), … |

On disk, v2 often uses **one file** `pipeline.ocblueprint` with both **`meta`** (creator slice) and **`slot_registry`** (blueprint). Editors should expose a **role** view vs an **advanced blueprint** view.

**Creator `meta` fields:** `id`, `name`, `version`, `author`, `description`, `personality`, `relations`, `default_relation`, `scenes`, `reply_quality_anchor`.

**Not for creators:** `slot_registry`, `groups`, **`runtime_config`**, **`pipeline`**, backends/models, enabling dual-core. **`reply_quality_anchor`** lives in **`runtime_config`** (see SETTINGS_REFERENCE §0).

---

## 1–8. Full specification

See the [Chinese ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md) for directory layout, legacy manifest/settings, validation, and `oclive collab`.
