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

---

## 9. Configuration file (`config.json`)

Optional JSON at **`distros/chat-pro/roles/{role_id}/config.json`**. Loaded by the host at role load time; **not** merged into `pipeline.ocblueprint`. Drives **immersive-mode** virtual clock, **Ebbinghaus memory decay**, **mention reinforcement**, and **relation estrangement**. Omitted keys use Rust defaults (same as the “Default” column below).

### 9.1 Example

```json
{
  "time": { "speed": 5.0, "decay_on_jump": true },
  "memory": {
    "decay_halflife_days": 7.0,
    "reinforcement_factor": 0.3,
    "min_strength_for_prompt": 0.1
  },
  "relation": {
    "decay_halflife_days": 30.0,
    "estrangement_threshold": 0.3
  }
}
```

Reference: `distros/chat-pro/roles/mumu/config.json`.

### 9.2 Top-level keys

| Key | Purpose |
|-----|---------|
| `time` | Virtual clock ratio and jump-time forgetting |
| `memory` | Long-term memory decay and reinforcement |
| `relation` | Intimacy estrangement and relation downgrade |
| `turn_thinking` | Co-present Fast/Deep routing, Deep latch, ephemeral situation summary — see §9.11 |

### 9.3 `time`

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `speed` | number | `5.0` | Real:virtual **minute** ratio (1 real minute = `speed` virtual minutes) |
| `decay_on_jump` | bool | `false` | Apply personality-delta time decay after manual virtual-time jumps |
| `decay_per_day` | number | `1.0` | Personality idle/jump decay strength (per virtual day) |

On first immersive entry, if `life_schedule` exists and no anchor is stored yet, virtual time starts at the **first schedule entry** weekday + `time_start`.

### 9.4 `memory`

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `decay_halflife_days` | number | `7.0` | Memory weight half-life in **virtual days** (~50% strength after one half-life) |
| `reinforcement_factor` | number | `0.3` | Slows decay for repeated topics: effective half-life × (1 + factor × (mention_count − 1)) |
| `min_strength_for_prompt` | number | `0.1` | Memories below `importance × weight` after decay are excluded from the chat prompt |

Formula: remaining weight ≈ initial × e^(−λ × virtual_age), λ = ln(2) / effective_half_life.

### 9.5 `relation`

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `decay_halflife_days` | number | `30.0` | Favorability (0–100) half-life in virtual days since last interaction |
| `estrangement_threshold` | number | `0.3` | When favorability/100 falls below this, relation stage **demotes one level** |

Estrangement runs at **turn start** in immersive mode only; each actual chat turn applies a small recovery bump so interaction is not fully erased by decay.

Types: `oclive_kernel_types::RolePackConfigFile`. Parse errors: host **warns** and keeps defaults; role load continues.

### 9.11 `turn_thinking` (Wave F · co-present routing)

**Full schema (Chinese SSOT):** [ROLE_PACK_SPEC.md §9.11](../../creator-docs/role-pack/ROLE_PACK_SPEC.md#911-turn_thinkingwave-f) · RFC [RFC_TURN_THINKING_PERSISTENCE_SUMMARY.md](../rfc/RFC_TURN_THINKING_PERSISTENCE_SUMMARY.md).

Optional object for **when to route Deep**, **latch until reconciliation** (e.g. Quarrel → Apology), and **ephemeral_archive** (rule-written situation summary with TTL, injected as `【局面摘要】`). Host defaults OR-merge with pack rules; **no player UI toggle**.

Validated by `oclive pack validate` (signal enums, TTL 1–8). Pack editor UI: **PE-TURN-01** (open in `oclive-pack-editor`).
