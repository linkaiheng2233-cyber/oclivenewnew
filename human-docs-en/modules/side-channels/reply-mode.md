# Side channel pack · reply mode (EN summary)

> Full checklist (ZH): [`human-docs/modules/side-channels/reply-mode.md`](../../../human-docs/modules/side-channels/reply-mode.md)
> Definition SSOT: [MODULE_MAP §11](../../../handoff/MODULE_MAP_AND_HANDOFF.md)

**You plug in**: Registry `id` **`reply_mode`** (not a six-slot key) · role pack `config.json` → `reply_mode` · `turn_pipeline/post.rs` after `reply_post_process` · presentation-only post-processing.

**Do**: Split LLM output on a line-only separator and strip the marker · persist one assistant message with `reply_segments` metadata · render multiple bubbles from segments · inject the separator protocol via the host prompt (never in persona text).

**Don't**: Write to `plugin_backends` as a slot · hard-code role ids in Vue · add chat tables · put segments into memory text.

Current boundaries: [RFC_REPLY_MODE](../../../creator-docs/rfc/RFC_REPLY_MODE.md) and [MODULE_MAP](../../../handoff/MODULE_MAP_AND_HANDOFF.md).

**Read next**: [ROLE_PACK_SPEC](../../../creator-docs/role-pack/ROLE_PACK_SPEC.md) · [reply-post-process](reply-post-process.md) · [RFC_SIDE_CHANNEL](../../../creator-docs/rfc/RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md).
