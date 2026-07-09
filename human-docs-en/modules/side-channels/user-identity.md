# Side channel pack · user identity (EN summary)

> Full checklist (ZH): [`human-docs/modules/side-channels/user-identity.md`](../../human-docs/modules/side-channels/user-identity.md)  
> Definition SSOT: [MODULE_MAP §11](../../handoff/MODULE_MAP_AND_HANDOFF.md)

**You plug in**: Registry `id` **`user_identity`** (not a six-slot key) · `user_identities/` · `turn_pipeline/pre.rs` · enters `process_message` at **pre**.

**Do**: Identity file format · pre injection into `PromptInput` · [RFC_SIDE_CHANNEL](../../creator-docs/rfc/RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md) scope.

**Don't**: Write to `plugin_backends` as a slot · confuse with MCP user (agent authorization domain).

Phase2 delivered — see [USER_IDENTITY_REPLY_POST_PROCESSOR_PHASE2](../../handoff/USER_IDENTITY_REPLY_POST_PROCESSOR_PHASE2.md) (historical, not a todo list).

**Read next**: [ROLE_PACK_SPEC](../../creator-docs/role-pack/ROLE_PACK_SPEC.md) · [reply-post-process](reply-post-process.md) · [CROSS_HOST_MEMORY](../../creator-docs/role-pack/CROSS_HOST_MEMORY.md).
