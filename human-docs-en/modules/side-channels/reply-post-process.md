# Side channel pack · reply post-process (EN summary)

> Full checklist (ZH): [`human-docs/modules/side-channels/reply-post-process.md`](../../../human-docs/modules/side-channels/reply-post-process.md)
> Definition SSOT: [MODULE_MAP §11](../../../handoff/MODULE_MAP_AND_HANDOFF.md)

**You plug in**: Registry `id` **`reply_post_process`** · role pack `config.json` · `turn_pipeline/post.rs` after post_llm · enters `process_message` at **post**.

**Do**: Polish rules · remote post-processors · documented `config.json` switches · keep DTO field **`reply`**.

**Don't**: Use `response` instead of **`reply`** · rewrite in Vue bypassing kernel · paste full [REPLY_POST_PROCESSOR_DESIGN_REPORT](../../../handoff/REPLY_POST_PROCESSOR_DESIGN_REPORT.md) into PRs.

**Read next**: [RFC_SIDE_CHANNEL](../../../creator-docs/rfc/RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md) · [role-pack-config](../packs/role-pack-config.md) · [slots/llm](../slots/llm.md) · [slots/agent](../slots/agent.md).
