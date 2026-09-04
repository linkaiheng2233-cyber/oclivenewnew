# Creator notes: relations, user identities, and initial favor

[中文](../../creator-docs/role-pack/CREATOR_USER_RELATIONS.md)

See [Role Pack Customization](CREATOR_ROLE_PACK_CUSTOMIZATION.md) for the short authoring path and [ROLE_PACK_SPEC.md](ROLE_PACK_SPEC.md) for the full contract.

Keep these concepts separate:

- **Relation:** `pipeline.ocblueprint` → `meta.relations`; interaction relation, initial favor, and multiplier.
- **User Identity Prompt Template:** optional `user_identities/`; defines who the user is and may map to one relation.
- **Current selection:** the host persists global/per-scene identity ids in SQLite role-runtime state. It does not rewrite the pack and is not a six-slot SessionCache override.

Legacy `manifest.json.user_relations` is the old name for `meta.relations`. The examples below use the current blueprint shape.

## Relation keys and display names

Keys such as `friend` and `classmate` are stable ids used by saves, APIs, and mappings. Avoid renaming them after release. Optional `display_name` supplies UI text; when absent or empty, the UI may show the id.

```json
{
  "schema_version": 4,
  "meta": {
    "relations": {
      "friend": {
        "display_name": "Friend",
        "prompt_hint": "You are close friends and speak casually.",
        "favor_multiplier": 1.0,
        "initial_favorability": 45
      }
    },
    "default_relation": "friend"
  }
}
```

Other required blueprint fields are omitted here; edit a complete v4 example rather than using this fragment alone.

## `default_relation`

- A non-empty value must reference a key in `meta.relations`.
- It is the fallback relation when no more specific mapping applies.
- It is not `user_identities/index.json.default_identity_id`: one selects a relation, the other a user identity template.

## Favor fields

- `favor_multiplier` must be finite and greater than zero.
- `initial_favorability` is the first value for that user–role relation and must be in 0–100.

## Mapping an identity to a relation

For a richer “who the user is” description, use `maps_to_relation_id` in `user_identities/index.json`:

```json
{
  "schema_version": 1,
  "default_identity_id": "classmate_user",
  "identities": {
    "classmate_user": {
      "display_name": "Classmate",
      "template_file": "classmate.md",
      "maps_to_relation_id": "classmate"
    }
  }
}
```

Put the template body in `user_identities/classmate.md`. Without `user_identities/`, the host may fall back to a relation's `prompt_hint`; that compatibility behavior does not make relation and identity the same concept. See the [User Identity RFC](../../creator-docs/rfc/RFC_USER_IDENTITY_AND_REPLY_POST_PROCESSOR.md) for runtime selection and distro restrictions.

## `memory_config.topic_weights` and scenes

Each top-level `topic_weights` key must name a declared scene. The effective scene set merges blueprint `meta.scenes` with `scenes/<scene_id>/` directories; an unknown reference fails role loading.

- Stable v4: `runtime_config.memory_config.topic_weights`;
- compatible v2: `meta.memory_config.topic_weights`;
- legacy: migrate with [V1_TO_V2_MIGRATION.md](V1_TO_V2_MIGRATION.md) instead of keeping both disk shapes.

## Validation

```powershell
cargo run -p oclive-cli -- pack validate .\distros\chat-pro\roles\<role id>
```

The default profile dispatches exactly across blueprint v2/v3/v4. Invalid relation ids, defaults, numeric ranges, scene references, and identity template files are reported rather than ignored.
