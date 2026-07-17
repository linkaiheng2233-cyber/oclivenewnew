# Facility pack · complex emotion (EN summary)

> Full checklist (ZH): [`human-docs/modules/facilities/complex-emotion.md`](../../../human-docs/modules/facilities/complex-emotion.md)
> Definition SSOT: [MODULE_MAP §10 facility ①](../../../handoff/MODULE_MAP_AND_HANDOFF.md)

**You plug in**: **Not** a `plugin_backends` six-key · code anchor `complex_emotion.rs` · `turn_pipeline/pre.rs` · output `PromptInput.previous_complex_emotion_narrative_hint` (consumed next turn).

**Distinction**: [emotion slot](../slots/emotion.md) analyzes user utterance; this facility builds narrative hint from emotion + context.

**Do**: Edit `complex_emotion.rs` narrative logic · default on (skippable flag) · pre injection.

**Don't**: Add `complex_emotion` to `slot_registry` · merge with emotion slot docs · silently expand to seventh slot without RFC (G1).

**Read next**: [slots/emotion](../slots/emotion.md) · [slots/prompt](../slots/prompt.md) · [AI_CHANGE_BOUNDARIES G1](../../../handoff/AI_CHANGE_BOUNDARIES.md).
