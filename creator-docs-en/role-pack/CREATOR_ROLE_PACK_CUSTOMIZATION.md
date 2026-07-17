# Role pack user customization — creator guide

[中文](../../creator-docs/role-pack/CREATOR_ROLE_PACK_CUSTOMIZATION.md)

For **authors building or maintaining role packs**: configure **user identity, display names, favor, and memory options** so players see the relationship labels and initial experience you intend.  
For scene directories and `scene.json`, see [Role pack scene system — creator guide](./CREATOR_SCENE_GUIDE.md); field-level identity rules in [Creator notes: user identity and initial favor](./CREATOR_USER_RELATIONS.md).

---

## 1. What is a role pack?

A **role pack** is a folder loaded from disk:

```text
distros/chat-pro/roles/<role id>/
```

`<role id>` must match top-level **`id`** in `manifest.json`. The host rejects path separators, `.`/`..`, control characters, Windows reserved device names, and surrounding whitespace. Stable lowercase English (digits/underscores/hyphens) or stable Unicode names are supported; avoid renaming to prevent save/import path conflicts.

---

## 2. Recommended directory layout

```text
distros/chat-pro/roles/<role id>/
├── manifest.json           # Required: metadata, user relations, scenes, memory policy
├── core_personality.txt    # Strongly recommended: core personality profile for main model; not rewritten at runtime
├── config.json             # Optional: virtual time, etc.
└── scenes/                 # Recommended: per-scene subdirs
    ├── <scene_id>/
    │   ├── scene.json
    │   └── description.txt
    └── ...
```

On load, the host reads `manifest.json` and validates; `core_personality.txt` and `scenes/` complete dialogue and display. If **`evolution.personality_source`** in `settings.json` is **`profile`**, **mutable personality profile** lives only in local DB and is model-maintained — not hand-authored in the pack; see **[docs/personality-archive-notes.md](../../docs/personality-archive-notes.md)**.

---

## 3. `manifest.json` — user-customization fields

### 3.1 Top-level display-related fields

| Field | Description |
|-------|-------------|
| `id` | Unique role id; matches folder name. |
| `name` | Display name in role list. |
| `version` / `author` / `description` | Version, author, blurb. |
| `model` | Optional default Ollama model (overridable in app). |
| `default_personality` | Optional seven-dimension personality initial values (~0–1). Under `profile` source, often view-only; still recommended. |
| `scenes` | Scene id list; merged with `scenes/` subdirs — see scene guide. |

### 3.2 `user_relations`: player “identity”

Each **user identity** is one entry in `user_relations`:

- **Key:** internal **English relation id** (e.g. `friend`, `classmate`, `parent`); referenced by saves/API — **keep stable after release**.
- **Value:** hints, multipliers, initial favor for that identity.

Example:

```json
"user_relations": {
  "classmate": {
    "display_name": "同学",
    "prompt_hint": "你和角色是同班同学，说话随意，会聊功课与课间琐事",
    "favor_multiplier": 1.0,
    "initial_favorability": 30
  },
  "parent": {
    "display_name": "父母",
    "prompt_hint": "你扮演孩子的家长，角色会嘴硬但在意你们的感受",
    "favor_multiplier": 1.15,
    "initial_favorability": 70
  }
}
```

| Field | Required | Description |
|-------|----------|-------------|
| `display_name` | No | **UI label** (any language). Empty → English key; app has fallbacks for common keys in testing — **still set explicitly in shipping packs**. |
| `prompt_hint` | No | **Relation hint for the model** — how user and role interact in this identity. |
| `favor_multiplier` | Default exists | Favor change multiplier; must be **positive**; validated on load. |
| `initial_favorability` | Default exists | Initial favor on **first relation**; **0–100**; validated on load. |

### 3.3 `default_relation`

Must be an English id **present in `user_relations` keys**; invalid key → pack **fails to load**.

### 3.4 `memory_config` and scene consistency

If using `memory_config.topic_weights`, **top-level keys must be scene ids** in `manifest.scenes` or merged `scenes/` dirs; else load fails with **Chinese error** — see [Creator notes: user identity and initial favor](./CREATOR_USER_RELATIONS.md).

### 3.5 `evolution` (optional)

Event impact, **`personality_source`**, mutable profile step size (`max_change_per_event`); defaults exist. Summary: [distros/chat-pro/roles/README_MANIFEST.md](../../distros/chat-pro/roles/README_MANIFEST.md) §5.3.

---

## 4. `core_personality.txt` and user identity

**Core personality profile** describes **who the character is**, voice, and boundaries; runtime **must not** rewrite this file.  
**User identity** (parent / classmate / lover, etc.) lives mainly in `user_relations` and `prompt_hint`. Keep them aligned — e.g. “user plays parent” packs should use child perspective in the profile, not conflict with `prompt_hint`.

---

## 5. Load validation and debugging

The host validates `manifest.json` on directory load (non-empty `id`/`name`, non-empty `user_relations`, valid `default_relation`, `topic_weights` vs scenes, numeric legality). **Failures return explicit Chinese messages** — fix JSON and retry.

---

## 6. Suggested creator workflow

1. Create `distros/chat-pro/roles/<your_role_id>/`; write `manifest.json` `id`, `name`, `user_relations`, `default_relation`.
2. Set **`display_name`**, **`prompt_hint`**, **`initial_favorability`**, **`favor_multiplier`** per identity.
3. Configure **`scenes`** and `scenes/<scene_id>/`; add **`topic_weights`** if needed.
4. Write **`core_personality.txt`** aligned with identities; for **`profile`** source, configure **`evolution`** in `settings.json` — do not hand-write mutable runtime profile in pack.
5. Load in app; check identity dropdown labels, scene list, and dialogue.

---

## 7. Further reading

- [docs/personality-archive-notes.md](../../docs/personality-archive-notes.md) — core/mutable profile and `personality_source`.
- [Creator notes: user identity and initial favor](./CREATOR_USER_RELATIONS.md) — `display_name`, `default_relation`, favor, `topic_weights`.
- [Role pack scene system — creator guide](./CREATOR_SCENE_GUIDE.md) — scenes, `scene.json`, `description.txt`, switching.

If load still fails with correct structure, share **full error text** and relevant `manifest.json` snippets.
