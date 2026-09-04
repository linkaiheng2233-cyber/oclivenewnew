# Side channel pack · reply post-process (EN summary)

> Full checklist (ZH): [`human-docs/modules/side-channels/reply-post-process.md`](../../../human-docs/modules/side-channels/reply-post-process.md)
> Definition SSOT: [MODULE_MAP §11](../../../handoff/MODULE_MAP_AND_HANDOFF.md)

**You plug in**: Registry `id` **`reply_post_process`** · role pack `config.json` · `turn_pipeline/post/post_llm.rs` after semantic/state consumers and before chat append · enters `process_message` at **post**.

**Do**: Polish rules · remote post-processors · documented `config.json` switches · preserve semantic/state → polish → display/transcript append order · keep DTO field **`reply`** · treat Agent `minimal_response` as an explicit bypass.

**Don't**: Use `response` instead of **`reply`** · rewrite in Vue bypassing kernel · claim Agent shortcuts pass through this channel · paste historical phase reports into PRs. Current contract: [RFC](../../../creator-docs/rfc/RFC_USER_IDENTITY_AND_REPLY_POST_PROCESSOR.md).

**Read next**: [RFC_SIDE_CHANNEL](../../../creator-docs/rfc/RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md) · [role-pack-config](../packs/role-pack-config.md) · [slots/llm](../slots/llm.md) · [slots/agent](../slots/agent.md).
