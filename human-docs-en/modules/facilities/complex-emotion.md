# Facility pack · complex emotion (EN summary)

> Full checklist (ZH): [`human-docs/modules/facilities/complex-emotion.md`](../../../human-docs/modules/facilities/complex-emotion.md)
> Definition SSOT: [MODULE_MAP §10 facility ①](../../../handoff/MODULE_MAP_AND_HANDOFF.md)

**You plug in**: **Not** a `plugin_backends` six-key; blueprints may declare a `type: complex_emotion` facility instance in the open `slot_registry` · code anchors `complex_emotion.rs`, `turn_pipeline/pre.rs`, `co_present/run_middle.rs`, `post/post_llm.rs`, and `complex_emotion_store.rs`.

**Distinction**: [emotion slot](../slots/emotion.md) analyzes the user utterance as possible fallback evidence. This facility handles reply-emotion labels and cross-turn narrative carry-over: the current prompt receives only a content-free signal from the prior stored hint, while a valid main-LLM `[EMO]` is authoritative for the current turn and remote/directory is fallback only.

**Do**: Keep prior-hint load, Fast deterministic intensity, post-LLM marker/plugin resolution, and TTL persistence separate · use explicit `builtin` to enable hint storage, `remote` / `directory` for plugin fallback, and omitted / `none` to disable hint reads/writes · strip protocol markers from user-visible replies.

**Don't**: Add it to the six-key `plugin_backends` contract or call it a stable seventh slot · merge with emotion slot docs · silently expand the six-port contract without a Breaking RFC (G1).

**Read next**: [NARRATIVE_HINT_CONTRACT](../../../creator-docs-en/testing/NARRATIVE_HINT_CONTRACT.md) · [slots/emotion](../slots/emotion.md) · [slots/prompt](../slots/prompt.md) · [AI_CHANGE_BOUNDARIES G1](../../../handoff/AI_CHANGE_BOUNDARIES.md).
