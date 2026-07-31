# Role Pack Customization

[中文](../../creator-docs/role-pack/CREATOR_ROLE_PACK_CUSTOMIZATION.md)

This page is the short authoring path. The complete contract is
[ROLE_PACK_SPEC.md](ROLE_PACK_SPEC.md). See
[CREATOR_USER_RELATIONS.md](CREATOR_USER_RELATIONS.md) for relation details and
[CREATOR_SCENE_GUIDE.md](CREATOR_SCENE_GUIDE.md) for scene authoring.

## Current format

New packs use **`pipeline.ocblueprint`** schema v2 or v3 as their single source
of truth. Do not add legacy `manifest.json` or `settings.json` beside it.

```text
roles/<role id>/
├── pipeline.ocblueprint
├── core_personality.txt
├── config.json                 # optional runtime policy
├── memory_seed.json            # optional read-only seed
├── portrait_catalog.json       # required by Portable Core
├── user_identities/
│   ├── index.json
│   └── <identity>.md
├── scenes/<scene id>/
│   ├── scene.json
│   └── description.txt
├── knowledge/*.md
└── assets/images/
```

The directory name must match `pipeline.ocblueprint` → `meta.id`. Use a stable
identifier; path separators, dot segments, control characters, Windows reserved
device names, and leading or trailing whitespace are rejected.

## Persona, relations, and identities

- `core_personality.txt` is the Tier 0 persona and cannot be overwritten by
  runtime evolution.
- A non-empty `meta.personality` must contain exactly seven values in `0.0–1.0`.
- `meta.default_relation` must reference an entry in `meta.relations`.
- `user_identities/index.json` may map richer user templates to those relations.
- Runtime mutable persona state belongs in the database, not in the pack.

Identity templates describe who the user is and the interaction boundary. They
should not duplicate the whole character persona, speak for the user, or silently
promote the relationship.

## Scenes and continuity

`meta.scenes` and `scenes/<scene id>/` form the effective scene set. A scene may
contain presentation data, time windows, remote-presence material, and optional
narrative continuity. Continuity tracks small state such as location, pose, and
activity; it does not replace memory, emotion, or the core persona.

## Portraits and Portable Core

For the portable visual baseline, enable `portrait_catalog` in `config.json`,
create `portrait_catalog.json`, and provide existing safe paths for:

`happy_default`, `sad_default`, `angry_default`, `neutral_default`,
`excited_default`, `confused_default`, and `shy_default`.

Portable Core is a minimum cross-distro contract, not a ceiling. A role may keep
additional scenes, portraits, voice resources, knowledge, or distro-specific
features when those files have a clear consumer and validation path.

## Voice and side channels

`voice_profile.json` is optional and overrides only speech tasks for that role;
switching roles must not rewrite global voice settings. `memory_seed.json` is an
initial read-only event seed, not the runtime memory database. `.ocpersona` and
`.ocmemory` remain separate migration artifacts.

## Validation

Mumu is tailored to Chat Pro and is not a universal upper bound. For a lighter
portable structure, refer to `distros/chat-pro/roles/deepseek/`.

```powershell
cargo run -p oclive-cli -- pack validate .\distros\chat-pro\roles\<role id>
cargo run -p oclive-cli -- pack validate .\distros\chat-pro\roles\<role id> --profile portable-core
```

Run the second command only for roles claiming Portable Core. Passing it does not
prove voice, full UI, or every distro-specific enhancement.

Legacy packs may still use `pack validate --profile legacy`. Migrate them with
[V1_TO_V2_MIGRATION.md](V1_TO_V2_MIGRATION.md), then
remove the legacy files so the pack has only one source of truth.

## Further reading

- [CREATOR_LEARNING_PATH.md](CREATOR_LEARNING_PATH.md)
- [PACK_VERSIONING.md](PACK_VERSIONING.md)
- [CROSS_HOST_MEMORY.md](CROSS_HOST_MEMORY.md)
- [personality-archive-notes.md](../../docs/personality-archive-notes.md)
