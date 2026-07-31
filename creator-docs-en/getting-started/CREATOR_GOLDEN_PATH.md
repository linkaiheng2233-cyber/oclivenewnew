# Creator golden path: your first role pack in 30 minutes

[中文](../../creator-docs/getting-started/CREATOR_GOLDEN_PATH.md)

This path has one goal: produce a role pack that A.I.Live can load, validate, and use in a conversation. [Role Pack Spec](../role-pack/ROLE_PACK_SPEC.md) is the format SSOT; this page does not duplicate its field reference.

## What you need

- The A.I.Live runtime;
- the [role-pack editor](https://github.com/linkaiheng2233-cyber/oclive-pack-editor), or this repository's `oclive-cli`;
- one personality draft. Portraits, scenes, knowledge, and seed memories can come later.

## 1. Create a minimal pack (5 minutes)

Create a pack in the editor, or run this from the repository root:

```powershell
cargo run -p oclive-cli -- pack create -o .\work\my-role --flat --id my-role --name "My Role" --format-blueprint-v2
```

New packs use `pipeline.ocblueprint`. Do not also create the legacy `manifest.json` / `settings.json` pair.

## 2. Write the role, not the runtime (15 minutes)

Start with only these items:

| Content | Location |
|---------|----------|
| Identity, voice, and behavioral boundaries | `core_personality.txt` |
| Name, author, seven traits, and default relations | `meta` in `pipeline.ocblueprint` |
| Seven portable emotion portraits | `portrait_catalog.json` and the PNG assets |
| Optional pre-authored memory events | `memory_seed.json` |

`memory_seed.json` is a read-only seed supplied by the creator. It is separate from user-generated long-term memory. Mutable personality and user memory belong to the runtime and must not be written back into the pack.

For a first pack, leave `slot_registry`, `groups`, remote plugins, dual-core settings, and MCP alone. They belong to distro or advanced integration work.

## 3. Validate and test a conversation (5 minutes)

```powershell
cargo run -p oclive-cli -- pack validate .\work\my-role
```

Import the directory or archive through A.I.Live, load the role, and test at least three turns: a normal greeting, an emotional change, and one personality-boundary question.

When running from source, point `OCLIVE_ROLES_DIR` at the roles root that contains your role directory, then start the desktop app.

## 4. Publish (5 minutes)

```powershell
cargo run -p oclive-cli -- pack publish .\work\my-role -o .\work\my-role-0.1.0.oclivepack
```

Before publishing, confirm that validation passes, no secrets or user memories are included, and asset attribution is complete. See [Pack Versioning](../role-pack/PACK_VERSIONING.md) and [Compatibility](../COMPATIBILITY.md).

## Read next, only when needed

- Scenes, knowledge, identities, or memory: [Creator Learning Path](../role-pack/CREATOR_LEARNING_PATH.md)
- Editor/runtime responsibilities: [Creator Workflow](CREATOR_WORKFLOW.md)
- Distros and plugins: [Documentation Index](DOCUMENTATION_INDEX.md)
