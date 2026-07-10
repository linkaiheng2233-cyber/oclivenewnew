# Creator notes: user identity and initial favor

[中文](../../creator-docs/role-pack/CREATOR_USER_RELATIONS.md)

Overview of role-pack “user customization” teaching: [Creator role pack customization](./CREATOR_ROLE_PACK_CUSTOMIZATION.md).

This page explains **`user_relations`** (user identity) fields in `manifest.json` and validation rules at load time.

## Relation key `id` and display name `display_name`

- **Keys** in `user_relations` (e.g. `friend`, `classmate`) are **English identifiers** used internally for saves, API, and default relation — keep stable; avoid renaming after release.
- **`display_name`** (optional): **display text** in UI dropdowns and relation preview. If omitted or empty, display falls back to the key (English).
- On export, if display name differs from key, `display_name` is written; if same, omitted for compact JSON.

Example:

```json
"user_relations": {
  "friend": {
    "display_name": "好友",
    "prompt_hint": "你们是好朋友，说话随意亲密",
    "favor_multiplier": 1.0,
    "initial_favorability": 45
  }
}
```

## `default_relation`

- Must reference a **key that exists in `user_relations`** (when non-empty string).
- Used for new conversations or when no relation is specified.

## `favor_multiplier` and `initial_favorability`

- **`favor_multiplier`:** favor change multiplier; must be a **finite positive** number.
- **`initial_favorability`:** initial favor when **user–role relation is first established** (0–100); must be finite; host clamps to valid range on load.

## `memory_config.topic_weights` and scenes

- Top-level keys in `topic_weights` **must be scene ids** present in at least one of:
  - top-level **`scenes`** in `manifest.json`, or
  - **subdirectory names** under `distros/chat-pro/roles/{role_id}/scenes/` (merged/deduplicated with manifest `scenes`).
- Otherwise load fails with a **Chinese error message** (for manifest correction).

## Validation timing

Validation runs when loading a role from disk (`manifest.json` → runtime `Role`); failures are not silently ignored — fix pack config per the message.
