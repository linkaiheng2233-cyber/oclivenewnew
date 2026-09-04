# Worldview knowledge (role pack assets)

[中文](../../creator-docs/role-pack/WORLDVIEW_KNOWLEDGE.md)

How **co-present main dialogue** loads Markdown under `distros/chat-pro/roles/{role_id}/knowledge/`, injects it into the prompt, and how **`event_hints`** augment [`EventDetector`](../../kernel/crates/oclive_kernel_runtime/src/domain/event_detector.rs) keywords (independent of six-slot backend selection).

## Directory and enable rules

- **Directory name:** fixed **`knowledge/`** at the role-pack root, beside current `pipeline.ocblueprint`.
- **Auto-enable:** when current blueprint `meta` has **no** `knowledge` field and `knowledge/` exists, load all `.md` files recursively.
- **Explicit off:** put `"knowledge": { "enabled": false }` in current blueprint `meta`; legacy manifest/settings blocks remain readable through compatibility merge.
- **Explicit on, no files:** `enabled: true` but no `.md` from glob → load fails (no silent half-pack).

## Current blueprint `meta.knowledge` block

```json
"meta": {
  "knowledge": {
    "enabled": true,
    "glob": "knowledge/**/*.md"
  }
}
```

- **`glob`:** must start with **`knowledge/`**; implementation recursively enumerates all `.md` under `knowledge/` (consistent with `**/*.md` convention).
- **Legacy compatibility:** legacy reference-host combined directories without `pipeline.ocblueprint` still read manifest/settings, where `settings.json.knowledge` overrides the legacy manifest field. New Stable v4 packs in that reference-host blueprint family should not maintain a second copy; this does not constrain another distro's product-pack format.

## Markdown and YAML front matter

Each `.md` **must** start with front matter or load errors.

```markdown
---
id: lore_city
tags: [雾城, 主线]
scenes: [home]
weight: 1.0
event_hints:
  quarrel:
    keywords: ["决裂", "分手"]
  praise:
    keywords: ["神作"]
---

Body: lore text for retrieval and prompt assembly.
```

| Field | Required | Description |
|-------|----------|-------------|
| `id` | Yes | Unique in pack; cited in prompt. |
| `tags` | No | Retrieval score boost. |
| `scenes` | No | Omit/empty = all scenes; else only when current `scene_id` matches. |
| `weight` | No | Default `1.0`; multiplied into retrieval score. |
| `event_hints` | No | Event types: `quarrel` / `apology` / `praise` / `complaint` / `confession` / `joke` / `ignore`; value is `keywords` array (optional `weight` reserved). |

## Runtime behavior (summary)

1. **Load:** [`RoleStorage::load_role_from_dir`](../../kernel/crates/oclive_kernel_host/src/infrastructure/storage/role.rs) parses knowledge after validation into **`Role::knowledge_index`** (`Arc`, in-memory only).
2. **Retrieval:** lightweight overlap scoring + `scenes` filter, Top-K → **【世界观设定】** section. **Co-present:** [`PromptBuilder::build_prompt`](../../kernel/crates/oclive_kernel_runtime/src/domain/prompt_builder/mod.rs) (after schedule inference, before long-term memory). **Remote life** (`remote_life`): [`build_remote_life_prompt`](../../kernel/crates/oclive_kernel_runtime/src/domain/remote_life_prompt.rs) filters by **character scene** `character_scene_id` (symmetric to co-present `scene_id`).
3. **Events:** retrieved blocks merge into [`KnowledgeEventAugment`](../../kernel/crates/oclive_kernel_types/src/models/knowledge.rs) → [`EventDetector::detect_with_augment`](../../kernel/crates/oclive_kernel_runtime/src/domain/event_detector.rs) and rule fallback in `estimate_event_impact` (B1: supplemental keywords, not replacing built-in emotion gates). Remote life path fixes event estimate to `Ignore`; knowledge does not re-run full event pipeline.

## Debugging tips

- Load failures include file path and reason (front matter / duplicate `id` / unknown `event_hints` key, etc.).
- Disable LLM event estimate (`OCLIVE_EVENT_IMPACT_LLM=0`) to observe rule-only + `event_hints` more easily.

## Related docs

- [PACK_VERSIONING.md](./PACK_VERSIONING.md) — versioning and `knowledge` contract  
- [../plugin-and-architecture/PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md) — replaceable subsystems (worldview knowledge is **not** a plugin backend)
