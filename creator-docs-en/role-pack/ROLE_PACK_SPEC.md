# Role pack format (ROLE_PACK_SPEC)

[中文](../../creator-docs/role-pack/ROLE_PACK_SPEC.md)

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

On disk, every supported blueprint version uses **one entry file** `pipeline.ocblueprint`. New packs use Stable v4; v2 remains compatible, while v3 is the frozen dual-core Beta. Editors should expose a **role** view vs an **advanced blueprint** view.

**Creator `meta` fields:** `id`, `name`, `version`, `author`, `description`, `personality`, `relations`, `default_relation`, and `scenes`.

**Not for creators:** `slot_registry`, `groups`, **`runtime_config`**, **`pipeline`**, backends/models, enabling dual-core. Stable v4 writes engine settings such as **`reply_quality_anchor`** only under **`runtime_config`**; `meta.*` remains a v2 compatibility fallback (see SETTINGS_REFERENCE §0).

---

## 1–8. Full specification

See the [Chinese ROLE_PACK_SPEC.md](../../creator-docs/role-pack/ROLE_PACK_SPEC.md) for directory layout, legacy manifest/settings, validation, and `oclive collab`.

### Optional scene narrative continuity

`scenes/{scene_id}/scene.json` may contain a `continuity` object. It is separate from core, mutable, and ephemeral personality data, and old packs without it keep legacy behavior.

- `initial_states` contains 1–8 creator-authored states with a unique `id`, `weight` (1–100), optional `time_windows` (`HH:mm`, including overnight windows), and required `sub_location`, `anchor`, `posture`, and `activity`.
- `default_state_id`, when present, references one initial state. The host makes a stable weighted selection from time-eligible states using the session, scene, and virtual day.
- `transitions` contains at most 32 explicit edges. `from` may be empty for any source; `to` references a state; `assistant_reply_markers` are scene-unique explicit action phrases in the final visible assistant reply.
- The host stores only scene/state IDs plus a CAS revision, injects the resolved state through dynamic `PromptInput.extra_sections` for Fast and Deep turns, and does not add a runtime LLM call. Scene switches invalidate the previous state.

The canonical schema example and validation semantics are in [Chinese §1.2](../../creator-docs/role-pack/ROLE_PACK_SPEC.md#12-scenesscene_idscenejson-叙事连续性可选). Editor-side generation of 3–8 reviewable candidates remains tracked as `PE-CONTINUITY-01`.

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
| `chat_storage` | Chat persistence / mirror policy — see §9.5a |
| `reply_post_processor` | Optional post-LLM shaping — see §9.7 |
| `portrait_catalog` | Portrait facility (facility 3) — see §9.9 |
| `visual_presentation` | Visual stage (facility 4) — see §9.10 |
| `meta_action_templates` | Break-wall meta-action attitude copy — see §9.8 |
| `turn_thinking` | Co-present Fast/Deep routing — see §9.11 |

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

### 9.5a `chat_storage` (chat backend & replay · hybrid)

Runtime always uses **HybridConversationStore** (SQLite truth + optional JSON mirror). `backend` enum **`hybrid` \| `file` \| `sqlite`** controls the **JSON mirror** only (`hybrid` on; `file`/`sqlite` off) — it does **not** swap independent store implementations. Full tables: [ZH ROLE_PACK_SPEC §9.5a](../../creator-docs/role-pack/ROLE_PACK_SPEC.md#95a-chat_storage聊天记录后端与回放--phase-3-hybrid) · [CHAT_STORAGE_ARCHITECTURE.md](../../handoff/CHAT_STORAGE_ARCHITECTURE.md).

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `backend` | string | `hybrid` | Mirror policy |
| `location` | string | `global` | `"role_pack"` or `"global"` for JSON mirror path |
| `replay_similarity_threshold` | number | `0.6` | Replay dedupe (0.1–1.0) |

Validated by `oclive pack validate` when the section is present.

### 9.6 Relation to blueprint / DB

| Concept | `config.json` | Blueprint / DB |
|---------|---------------|----------------|
| Memory FIFO size | — | `runtime_config.memory_config` / `policy.toml` |
| Favor baseline / event deltas | — | `meta.relations` + turn event engine |
| LTM content / `mention_count` | Decay/reinforce knobs | SQLite `long_term_memory` |
| Virtual time anchors | `time.speed` etc. | `role_runtime.virtual_time_*` |

### 9.11 `turn_thinking` (Wave F · co-present routing)

**Full schema (Chinese SSOT):** [ROLE_PACK_SPEC.md §9.11](../../creator-docs/role-pack/ROLE_PACK_SPEC.md#911-turn_thinkingwave-f) · RFC [RFC_TURN_THINKING_PERSISTENCE_SUMMARY.md](../rfc/RFC_TURN_THINKING_PERSISTENCE_SUMMARY.md).

Optional object for **when to route Deep**, **latch until reconciliation** (e.g. Quarrel → Apology), and **ephemeral_archive** (rule-written situation summary with TTL, injected as `【局面摘要】`). Host defaults OR-merge with pack rules; **no player UI toggle**.

Validated by `oclive pack validate` (signal enums, TTL 1–8). Pack editor UI: **PE-TURN-01** (open in `oclive-pack-editor`).

### 9.7 `reply_post_processor` (Reply Post-Processor · off by default)

Optional post-LLM reply shaping in **`config.json`**. **Independent channel** — not a six-slot entry and **not** configured in `slot_registry` or blueprints.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | **`false`** | When `false`, pass-through (users unaffected) |
| `backend` | string | `"builtin"` | `builtin` \| `remote` \| `directory` |
| `builtin` | object | — | `profile` (`standard` \| `minimal`), `max_chars`, `strip_leading_quote` |
| `remote` | object | — | `url`, `timeout_ms`; JSON-RPC `reply_post_process.process` |
| `directory` | object | — | `plugin_id`; plugin `provides` must include `reply_post_process` |

**Distro merge:** `distro.oclive.toml` → `[post_process].chain=minimal` forces effective `builtin.profile=minimal` when enabled. See [DISTRO_CAPABILITY_PROFILE.md](../kernel/DISTRO_CAPABILITY_PROFILE.md).

**RPC contract (directory / remote):** method `reply_post_process.process`; params `raw_reply`, `user_message`, `role_id`, `scene_id`, `locale`; result `display_reply` (+ optional `diagnostic`). Normative detail: [PLUGIN_V1.md §reply_post_process](../plugin-and-architecture/PLUGIN_V1.md).

**Validation:** `oclive pack validate` requires non-empty `remote.url` when `enabled=true` and `backend=remote`; directory requires non-empty `plugin_id`.

**DTO:** `include_raw_reply: true` may surface `raw_reply` when post-processing changes text (`SendMessageResponse` schema **16**).

### 9.8 `meta_action_templates` (break-wall meta actions · optional)

Host does **not** require this; VS Code / clients may inject attitude copy as a normal user turn after storage edits. Keys: `undo` / `regenerate` / `edit` / `delete` (`enabled`, `attitude_text`). Empty attitude or `enabled: false` → silent. Full tables: [ZH §9.8](../../creator-docs/role-pack/ROLE_PACK_SPEC.md#98-meta_action_templates破壁元操作--可选).

### 9.9 `portrait_catalog` (portrait facility · optional)

RFC summary: [RFC_PORTRAIT_FACILITY_SUMMARY.md](../rfc/RFC_PORTRAIT_FACILITY_SUMMARY.md). `config.json` → `portrait_catalog.enabled` loads sibling `portrait_catalog.json`. **Condensed:** EN does not duplicate the full asset-field table — see [ZH §9.9](../../creator-docs/role-pack/ROLE_PACK_SPEC.md#99-portrait_catalog立绘设施--v04--a2-磁盘).

### 9.10 `visual_presentation` (visual stage · draft · off by default)

RFC summary: [RFC_VISUAL_PRESENTATION_FACILITY_SUMMARY.md](../rfc/RFC_VISUAL_PRESENTATION_FACILITY_SUMMARY.md). Fields: `enabled`, `backend` (`none` \| `image` \| `live2d` \| `rig3d` \| `procedural` \| `directory`), `resources`. **No** second AI image-pick here; input is facility-3 `visual_state_id`. Full tables: [ZH §9.10](../../creator-docs/role-pack/ROLE_PACK_SPEC.md#910-visual_presentation视觉表现设施--草案--默认关闭).

---

## 11. Versioning and migration

Role packs have two version layers — do not mix them:

| Layer | Where | Expectation |
|-------|--------|-------------|
| **Pack release version** | Blueprint `meta.version` or legacy `manifest.version` | **semver string** (e.g. `1.2.0`); creator-facing release id |
| **Format / contract version** | Blueprint `schema_version`; legacy `settings.schema_version`; optional `manifest.min_runtime_version` | Controls parse path and load rejection; detail in [PACK_VERSIONING.md](PACK_VERSIONING.md) |

**Relation to schema / manifest**

- **Canonical shape** is `pipeline.ocblueprint`: new packs use Stable `schema_version` **4**, existing **2** remains compatible, and **3** is reserved for the frozen dual-core Beta. Pack layout and validation follow the [Chinese ROLE_PACK_SPEC](../../creator-docs/role-pack/ROLE_PACK_SPEC.md) §§1–2 / §6.
- Declared `includes[]` are strict activation dependencies: paths use portable ASCII letters/digits plus `_` `.` `/` `-`; missing, escaping, unreadable, malformed, or contract-invalid merged fragments fail both `pack validate` and role loading. Best-effort merging is preview-only; expert routing uses its dedicated loader rather than a generic include target.
- **Legacy** still uses `manifest.json` + `settings.json`; key whitelist, `min_runtime_version` (host semver gate), and unknown-key policy live in [PACK_VERSIONING.md](PACK_VERSIONING.md) — do not restate those tables here.
- JSON Schema / CLI: `oclive pack validate`; implementation SSOT is `oclive_validation`.

### Portable Core (`--profile portable-core`)

Portable Core is the cross-distro minimum, not a ceiling on distro features. A v2/v3/v4 pack validated with this profile must provide a non-empty `core_personality.txt`, enable `config.json` → `portrait_catalog.enabled`, and include local `image` assets for the seven stable IDs: `happy_default`, `sad_default`, `angry_default`, `neutral_default`, `excited_default`, `confused_default`, and `shy_default`. Hosts must be able to load the persona and run a basic turn; visual, voice, UI, agent, and hardware extensions remain distro `HostProfile` concerns. Validate with `oclive pack validate <role-dir> --profile portable-core`. Full capability parity is a separate distro conformance test.
A minimal portable pack still needs a blueprint shell: host scanning only recognizes `pipeline.ocblueprint` (or legacy `manifest.json`) as a role entry, so the seven images plus `core_personality.txt` alone are not enough. The minimal shell is `schema_version` + `meta` (`id` equal to the role folder name, `name`, `version`, `author`, `description`, and at least one `relations` entry) + a `slot_registry` that must be non-empty with at least one `type: llm` instance (allowed backends: `ollama` / `remote` / `directory` / `none`); an empty `{}` is rejected by `pack validate` and host loading. The other five six-slot backends (`memory`, `emotion`, `event`, `prompt`, `agent`) are optional and fall back to builtin defaults at runtime (`PluginBackends::default()`). `complex_emotion` is a separate facility (not one of the six slots) with different semantics: **omitted (no entry) = complex emotion disabled** (provider skipped, no hint produced); explicit `builtin` = enabled; explicit `none` = explicitly disabled (equivalent to omitted, removing implicit ambiguity). This semantic lands with emotion-engine phase-B M1; until then, an absent entry still falls back to builtin at runtime. Full `slot_registry` entries in shipped packs such as `deepseek` / `mumu` reflect Chat Pro distro feature needs (remote models, emotion engine, agent), not minimum-format requirements. `pack create --format-blueprint-v2` scaffolds the fuller seven-slot template, which is a convenience default rather than the minimum.

Portable state is split into two JSON documents. `.ocpersona` carries the immutable core identity plus an optional mutable-profile snapshot; import may restore only the mutable profile after matching the installed role id and core. `.ocmemory` carries optional creator-authored `memory_seed` entries and runtime long-term memories. Chat logs, short-term cache, and ephemeral situation state are excluded from both. Optional role-pack `memory_seed.json` is read-only at runtime, participates in retrieval without decay, and is never merged into user LTM. Validate with `oclive-cli pack validate-persona` and `validate-memory`; extension data belongs under the top-level `extensions` object.

### Chat Pro adult role extension (`adult_extension.json`, optional)

Adult behavior is an optional distro extension inside the same role pack, not a Portable Core requirement. Unsupported distros must ignore the root-level file and continue to run the base persona, scenes, and identities. Adult persona, dialogue guidance, and scene directions belong only in this file and must not be copied into base prompt files.

The v1 object requires `character_is_adult: true`; `pacing.mode` is `creator` or `ai`, `suggested_interval_ms` is positive, and every `scenes` key must reference a base scene id. Chat Pro injects it only after local adult confirmation plus the global and per-role gates. Legacy user-identity `adult_eligible` metadata remains readable for pack compatibility and creator guidance, but is not a hidden fourth authorization gate. `SendMessageResponse.schema` **16** returns `adult_beat` with dialogue, silent narration, interaction state, and an optional interval while retaining dialogue in the base `reply` compatibility field. Malformed envelopes use a prompt-safe fallback.

Interaction state persists by role/chat scene. Disabling either gate clears active state immediately; scene or identity changes first request a natural ending. Adult memories use a separate `content_scope=adult`; ordinary chat only reads ordinary memory plus a non-explicit relationship bridge. The pack editor exposes an independent adult-extension page only after the complete base pack validates, and exports one combined pack.

Chat Pro background pre-generation is a host runtime capability; it adds no `adult_extension.json` field and does not change Portable Core. Future model beats first enter cancellable staged-beat storage and must not update chat history, memory, relationship, events, personality, or narrative continuity. Only after that chat is foreground and both pacing and prior-voice gates are satisfied does the host commit and render one beat in order; narration remains silent. New user input, exit, gate shutdown, or identity change cancels in-flight work and discards beats not yet committed. Queue capacity is one positive-integer Chat Pro setting shared across all roles and chats; lowering it pauses new generation without deleting staged beats.

**Breaking fields (must rewrite on migration; silent ignore is wrong)**

- Blueprint bumps select explicit contracts: v2 keeps legacy engine fields under `meta`; Stable v4 makes top-level **`runtime_config`** the single engine-settings source; v3 remains the opt-in dual-core Beta.
- Legacy → v2: fold dual files into blueprint **`meta`** + **`slot_registry`**; **must not** coexist with legacy dual files; blueprints **must not** carry `steps[]` / `entry` / `module_relations` (first-turn path remains `process_message` → `co_present`, not old DSL scheduling).
- Unknown top-level keys or invalid backend enum values fail validation; slot/backend names must match PLUGIN_V1 (`plugin_backends` · `slot_registry.type`).

**Recommended migration steps (high-level · details linked only)**

1. Back up the pack; confirm whether you have legacy dual files or an existing `pipeline.ocblueprint`.
2. **Legacy → v2**: follow [V1_TO_V2_MIGRATION.md](V1_TO_V2_MIGRATION.md), then `pack validate`.
3. Keep existing v2 packs on v2 unless intentionally migrating. Use v3 only for the frozen dual-core Beta ([V2_TO_V3_MIGRATION.md](V2_TO_V3_MIGRATION.md)); otherwise create or migrate to Stable v4 and move engine fields to `runtime_config` without duplicating them in `meta`.
4. When declaring a minimum host, align `min_runtime_version` / `--host-version` with [PACK_VERSIONING.md](PACK_VERSIONING.md).

> **Non-goal:** no batch auto-migration CLI in-tree; hand-edit + validate. Index: [DOCUMENTATION_INDEX.md](../getting-started/DOCUMENTATION_INDEX.md). Full Chinese §11: [ROLE_PACK_SPEC §11](../../creator-docs/role-pack/ROLE_PACK_SPEC.md#11-版本与迁移).

---

## Missing-section policy (this EN page)

This English page is **condensed**. Normative field tables for directory layout, legacy `manifest`/`settings`, RobotSoulPack, and any section not expanded above remain in the **[Chinese ROLE_PACK_SPEC](../../creator-docs/role-pack/ROLE_PACK_SPEC.md)** (SSOT). Do not treat EN silence as “absent from product.”
