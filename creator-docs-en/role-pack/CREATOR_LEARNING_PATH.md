# Role-pack creator learning path

[中文](../../creator-docs/role-pack/CREATOR_LEARNING_PATH.md)

Complete the [30-minute golden path](../getting-started/CREATOR_GOLDEN_PATH.md) first. This page is an advanced router; it does not duplicate field definitions from the [Role Pack Spec](ROLE_PACK_SPEC.md).

## Read by the task in front of you

| Goal | Read and verify |
|------|-----------------|
| Add locations or situations | [Scene Guide](CREATOR_SCENE_GUIDE.md); the role remains consistent after a scene change |
| Add world knowledge | [Worldview Knowledge](WORLDVIEW_KNOWLEDGE.md); reloading the role exposes the new knowledge |
| Support different user identities | [User Identity RFC](../../creator-docs/rfc/RFC_USER_IDENTITY_AND_REPLY_POST_PROCESSOR.md); default and scene identities do not conflict |
| Add prior events | `memory_seed.json` in the [Role Pack Spec](ROLE_PACK_SPEC.md); no real user data is included |
| Tune core and mutable personality | [Personality Archive Notes](../../docs/personality-archive-notes.md); runtime updates never overwrite the core file |
| Supply seven base portraits | [Portable Core](ROLE_PACK_SPEC.md#portable-core---profile-portable-core); validation passes with the `portable-core` profile |
| Maintain a legacy pack | [V1 to V2 Migration](V1_TO_V2_MIGRATION.md); the migrated pack has one format SSOT |
| Publish a new version | [Pack Versioning](PACK_VERSIONING.md); version and minimum runtime constraints agree |

If a section anchor moves, use the table of contents in [ROLE_PACK_SPEC.md](ROLE_PACK_SPEC.md).

## Three boundaries

1. Creators own character identity, assets, scenes, and read-only seed memories.
2. The runtime owns user chats, long-term memory, and mutable personality; these do not ship in a public role pack.
3. Distro and plugin developers own slot backends, permissions, and platform enhancements; the portable base pack must work without them.

See [Role Pack Boundary](../../handoff/ROLE_PACK_BOUNDARY.md) for the full ownership model.

## Before every release

```powershell
cargo run -p oclive-cli -- pack validate <role-directory>
cargo run -p oclive-cli -- pack validate <role-directory> --profile portable-core
```

Then import the pack and run a multi-turn conversation in each target distro. The [OCLive CLI Guide](../cli/OCLIVE_CLI_GUIDE.md) is the command reference.
