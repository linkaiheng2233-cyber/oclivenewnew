# Role pack author learning path

[中文](../../creator-docs/role-pack/CREATOR_LEARNING_PATH.md)

Time-boxed steps. **Normative layout** remains [ROLE_PACK_SPEC.md](ROLE_PACK_SPEC.md) and [distros/chat-pro/roles/README_MANIFEST.md](../../distros/chat-pro/roles/README_MANIFEST.md). CLI from repo root: **`cargo run -p oclive-cli -- pack …`**.

---

## Migrate v1 → v2 (~10 min)

Packs still on **`manifest.json` + `settings.json`**: **[V1_TO_V2_MIGRATION.md](V1_TO_V2_MIGRATION.md)** — `pack migrate-to-blueprint` → default `pack validate` → smoke chat in the host.

---

## Beginner (~30 min)

| Step | Goal | Read / do |
|------|------|-------------|
| 1 | Know the on-disk shape | [ROLE_PACK_SPEC.md](ROLE_PACK_SPEC.md) **§1** |
| 2 | Generate a minimal pack | `cargo run -p oclive-cli -- pack create -o <parent> --id my_first_role --format-blueprint-v2` (writes `pipeline.ocblueprint`; see [../cli/OCLIVE_CLI_GUIDE.md](../cli/OCLIVE_CLI_GUIDE.md)) |
| 3 | Open in the editor | **oclive-pack-editor** for v2 blueprint or legacy files ([CREATOR_WORKFLOW.md](../getting-started/CREATOR_WORKFLOW.md)) |
| 4 | Edit façade fields | v2: **role-pack `meta` only** + **`prompts/`**; do not edit `slot_registry`; legacy: [README_MANIFEST](../../distros/chat-pro/roles/README_MANIFEST.md) |

**Done when:** `pack validate <role-root>` passes (default v2); legacy packs use `--profile legacy`.

### Permission boundary

Edit only **role-pack** fields; do not touch **`slot_registry`** or **`dual_core.enabled`**. See [ROLE_PACK_SPEC §0](ROLE_PACK_SPEC.md#0-role-pack-vs-blueprint) · [ROLE_PACK_BOUNDARY.md](../../handoff/ROLE_PACK_BOUNDARY.md).

---

## Intermediate (~1–2 h)

| Topic | Read |
|-------|------|
| **Seven-dim personality** | [README_MANIFEST](../../distros/chat-pro/roles/README_MANIFEST.md) · [docs/personality-archive-notes.md](../../docs/personality-archive-notes.md) |
| **System prompts / openings** | ROLE_PACK_SPEC + [WORLDVIEW_KNOWLEDGE.md](../../creator-docs/role-pack/WORLDVIEW_KNOWLEDGE.md); final chat prompt is driven by **`slot_registry` `type: prompt`** and engine policy |
| **Slots & modules 1–6** | [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md) · [SETTINGS_REFERENCE.md](../cli/SETTINGS_REFERENCE.md) |

**Done when:** You can configure **`slot_registry`** instances (`type` + `backend`) and explain directory `plugin` / `plugins` → manifest `id`.

---

## Advanced (~half day)

| Topic | Notes |
|-------|--------|
| **`reply_quality_anchor`** | See README_MANIFEST + ROLE_PACK_SPEC merged settings table; behavior follows validation + host load rules |
| **`pipeline.ocblueprint` v2 (recommended SSOT)** | [ROLE_PACK_SPEC.md](ROLE_PACK_SPEC.md) · [BLUEPRINT_V2_IMPLEMENTATION_PLAN.md](../../handoff/BLUEPRINT_V2_IMPLEMENTATION_PLAN.md). **Desktop orchestration** is **`process_message` → `co_present`** (no blueprint `steps[]`; [AGENTS.md](../../AGENTS.md)). Desktop graph can **`save_role_slot_registry`** |
| **Validate** | Default v2: `pack validate <role-root>`; legacy: `--profile legacy`; headless: `--profile robot-soul` (legacy shape; ROLE_PACK_SPEC §6) |
| **Editor wasm checks** | **oclive-pack-editor** `wasm:build` + “run all checks” |

**Done when:** `pack validate` is clean and you know which keys are host-validated vs author-only.

### Memory & relation evolution (`config.json`)

In **immersive mode**, optional **`distros/chat-pro/roles/{id}/config.json`** (not the blueprint) drives human-like forgetting and estrangement. Full field list: **[ROLE_PACK_SPEC §9](ROLE_PACK_SPEC.md#9-configuration-file-configjson)** (Chinese spec is authoritative; English summary in [creator-docs-en/ROLE_PACK_SPEC.md](../creator-docs-en/role-pack/ROLE_PACK_SPEC.md)).

| Mechanism | Idea | Main keys |
|-----------|------|-----------|
| **Memory decay** | Long-term memory weight decays exponentially with virtual age (Ebbinghaus); weak memories drop out of the prompt | `memory.decay_halflife_days`, `memory.min_strength_for_prompt` |
| **Reinforcement** | Similar topics bump `mention_count` and **extend** effective half-life | `memory.reinforcement_factor`, `memory.similarity_threshold` |
| **Estrangement** | Favorability decays with virtual days since last chat; may demote relation stage | `relation.decay_halflife_days`, `relation.estrangement_threshold` |
| **Virtual time** | `speed` real:virtual minutes; optional decay on manual time jumps | `time.speed`, `time.decay_on_jump` |

Starter snippet:

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

With `life_schedule` in `meta`, first immersive entry aligns virtual time to the **first schedule slot** start. Smoke-test: repeat the same topic (mention_count rises); leave the role idle (favor / stage may drop).

---

## Publish

| Step | Command / doc |
|------|----------------|
| **`.oclivepack`** | `cargo run -p oclive-cli -- pack publish <role-root> -o <path>` |
| **Community index JSON** | [ROLE_PACK_INDEX.md](ROLE_PACK_INDEX.md) · [../../creator-docs/roadmap/MARKET_LAUNCHER_INTEGRATION.md](../../creator-docs/roadmap/MARKET_LAUNCHER_INTEGRATION.md) |
| **Host compatibility** | [COMPATIBILITY.md](../COMPATIBILITY.md) · `manifest.min_runtime_version` |

---

## Next

- Versioning: [PACK_VERSIONING.md](../../creator-docs/role-pack/PACK_VERSIONING.md)  
- Editor validation roadmap: [EDITOR_VALIDATION_ROADMAP.md](../../creator-docs/role-pack/EDITOR_VALIDATION_ROADMAP.md)
