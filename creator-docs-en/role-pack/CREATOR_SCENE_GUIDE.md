# Role pack scene system — creator guide

[中文](../../creator-docs/role-pack/CREATOR_SCENE_GUIDE.md)

For overall pack structure and user-identity configuration, see [Creator role pack customization](./CREATOR_ROLE_PACK_CUSTOMIZATION.md).

This guide explains how to **configure, extend, and debug scenes** in a role pack so dialogue, memory policy, and scene switching match your intent. **Usually you only change pack files; no host source edits.**

---

## 1. What is a scene?

A **scene** is the **space or situation** where the role and user interact (home, school, office, amusement park, etc.).

The app will:

- Choose **memory and policy** by current scene (when configured);
- Send scene-related context into the **main dialogue model** for tone and environment (alongside the pack **core personality profile**);
- Try **automatic scene switching** from user lines, or let the user **switch manually** in the UI.

---

## 2. Directory layout

Example for role id `mumu`:

```text
distros/chat-pro/roles/mumu/
├── manifest.json              # Declares scenes list, memory topic weights, etc.
├── core_personality.txt       # Core personality profile (scene-independent)
├── config.json                # Optional: time decay, etc.
└── scenes/
    ├── home/
    │   ├── scene.json         # Required: metadata (name, keywords, …)
    │   └── description.txt    # Strongly recommended: detailed setting for main model
    ├── school/
    │   ├── scene.json
    │   └── description.txt
    └── ...
```

- **`scene_id`**: matches folder name (e.g. `home`, `school`); use **lowercase English + underscores**, avoid spaces/special chars for cross-platform and API stability.

---

## 3. Declaring scenes in `manifest.json`

List all scene ids in the top-level **`scenes`** array (**explicit listing recommended**):

```json
"scenes": ["home", "school", "company", "park"]
```

Notes:

- The host merges **`manifest.json` `scenes`** with **subdirectory names under `scenes/`**, then sorts and deduplicates.
- If both are empty, built-in `default` logic applies; **normally configure at least one scene directory + `scene.json`.**

### 3.1 Memory topic weights `memory_config.topic_weights` (optional)

For “topic types preferred in this scene” memory hints, configure **topic → weight** per **scene id** (weights 0–1, relative scale matters):

```json
"memory_config": {
  "scene_weight_multiplier": 1.2,
  "topic_weights": {
    "home": { "日常": 0.75, "学习": 0.15, "工作": 0.1 },
    "school": { "学习": 0.85, "日常": 0.15 }
  }
}
```

- **New scenes not in this table:** no “more likely to discuss topic X here” hint; **runtime unaffected**, one auxiliary hint omitted.
- Add a `topic_weights` row for new scenes to align with existing logic.

---

## 4. `scene.json` fields

The host reads `scenes/<scene_id>/scene.json`. All fields optional; **at least `name` and several `keywords` recommended**.

| Field | Type | Purpose |
|-------|------|---------|
| `name` | string | **Display name** (any language), e.g. dropdown, system hints, “current scene” |
| `welcome_message` | string | **Welcome line when entering** (overrides drawing from `monologues`) |
| `keywords` | string[] | **Scene-switch rules**: user lines containing these words more likely switch here; also used for short description if no `description.txt` |
| `events` | string[] | Plot tags for concatenation and atmosphere |
| `monologues` | string[] | Welcome/monologue pool when no `welcome_message` |
| `time_windows` | object[] | **Optional:** auto-switch only in virtual time windows (below) |

### 4.1 `time_windows` (advanced)

Each entry: `{ "start": "HH:MM", "end": "HH:MM" }` (24h).  

- If **non-empty:** **auto-switch** only when **in-app virtual time** falls in a window.
- If **empty:** no time restriction.

Useful for “amusement park closed at night”; for debugging, remove `time_windows` or widen ranges.

---

## 5. `description.txt` (strongly recommended)

Path: `scenes/<scene_id>/description.txt`  

- **Content:** Long-form setting for the **main dialogue LLM** — environment, relationships, tone boundaries, taboos, typical beats; natural language is fine.
- **Effect:** Enters the “scene setting (from role pack…)” block in the main prompt; host **truncates** very long text (thousands of characters) to protect context.
- **Without file:** host builds a short blurb from `scene.json` `name`, `keywords`, `events`; works for new scenes but less nuanced than hand-written text.

**Writing tip:** 1–3 overview paragraphs, then bullets for how the role addresses the user, common topics, and what to avoid.

---

## 6. How the host uses scenes (no code changes)

### 6.1 Scene list and display names

- List from **manifest + disk directories** merged.
- Display name prefers **`scene.json` `name`**; then built-in id→label map; else raw id.

### 6.2 Main dialogue prompt

- **Current scene display name** + **topic hint** (if `topic_weights`) + **`description.txt` or auto short blurb** enter main dialogue; model should reply in that atmosphere.

### 6.3 Automatic scene switching (backend)

1. **Rule layer:** movement words (go, come, back, to, enter, leave, visit, …) plus **keywords / display name / id** scoring may switch scene.
2. **Model layer:** if rules miss, a **small model** outputs JSON from candidate list + per-line summaries.
3. **Summaries** from pack: first non-empty line of `description.txt`, or “name + first keyword”.

**More colloquial keywords → more reliable auto-switch.**

### 6.4 Policy plugin `config/policy.toml` (optional)

- File lives in app **`config/policy.toml`** (separate from a single pack).
- **`[scene_bindings]`** maps **scene_id** → **`conservative`** / **`exploratory`** profiles (memory filter, default importance, etc.).
- **Unlisted scene_id:** uses **`default_profile`**, no error.
- Copy a row and change id to match a new scene’s policy.

---

## 7. Client behavior (for testing)

- After send, if backend switches scene, a **system hint** may appear (e.g. entering a scene next turn).
- User lines with **movement intent** and **multiple scenes** may show **“Where to?”** destination picker **after this turn’s reply**; selection calls the same switch API as the top bar.
- **Single-scene packs:** no destination list — **expected**.

---

## 8. Adding a scene: checklist

1. Create `distros/chat-pro/roles/<role_id>/scenes/<new_scene_id>/`.
2. Write `scene.json`: at least `name`, many `keywords`, optional `welcome_message`, `events`, `monologues`.
3. Write `description.txt` (recommended).
4. Append id to **`manifest.json` `scenes`**.
5. (Optional) Add topic weights under **`memory_config.topic_weights`**.
6. (Optional) Add **`scene_bindings`** in **`config/policy.toml`**.
7. Reload role or restart app; test dialogue and switching.

---

## 9. FAQ and troubleshooting

| Symptom | Likely cause | Fix |
|---------|--------------|-----|
| UI shows only one scene | Pack has one scene or manifest not listing others | Add directories and manifest entries |
| Auto-switch never hits a scene | Keywords don’t match user phrasing | Add colloquial keywords, check homophones |
| Scene never enters at some times | `time_windows` and virtual time outside window | Adjust or remove windows for testing |
| No “topic bias” line for new scene | No `topic_weights` entry | Add row in manifest |
| Policy unlike expected | No `policy.toml` binding | Add `scene_bindings` or use default |

---

## 10. Best practices

1. **Stable ids:** change scene ids rarely to avoid `current_scene` drift in saves/DB.
2. **Keywords: more is better:** cover “go home”, “to the office”, “at school”, etc.
3. **`description.txt` describes performance:** less system meta, more **how the role speaks and what they care about here**.
4. **Skip time windows first:** add `time_windows` after basic switching works.
5. **Split with core profile:** `core_personality.txt` = character voice; scene files = **space and situation**; avoid duplicating long identical text.

---

## 11. Versioning and compatibility

- Bump **`manifest.json` `version`** with content changes for distribution and debugging.
- New optional fields after app upgrades are generally **backward compatible**; old `scene.json` may omit them.

---

If behavior differs from this doc, record **role id, manifest `scenes`, problematic `scene_id`, user line, and system hints** for precise diagnosis.
