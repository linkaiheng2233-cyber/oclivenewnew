# Slot pack · `emotion` (EN summary)

> Full checklist (ZH): [`human-docs/modules/slots/emotion.md`](../../../human-docs/modules/slots/emotion.md)
> Definition SSOT: [MODULE_MAP §5](../../../handoff/MODULE_MAP_AND_HANDOFF.md)

**You plug in**: blueprint `slot_registry.type: emotion` → folded `PluginBackends.emotion` (legacy v1: `settings.json.plugin_backends.emotion`) · trait `UserEmotionAnalyzer` · hook `turn_pipeline/pre.rs` → `EmotionResult` → Prompt · Turn Thinking Auto.

**Distinction**: This slot analyzes **user-utterance** emotion as a prompt clue or one fallback input. For the [complex-emotion facility](../facilities/complex-emotion.md), a valid main-LLM `[EMO]` is authoritative for current reply emotion, remote/directory is fallback only, and any usable hint is carried into the next turn (not a six-slot key).

**Do**: `builtin` · `remote` · `directory` · `none` backends · align pre output to `EmotionResult` · use `Emotion` enum from dto.

**Don't**: Treat a `slot_registry` `complex_emotion` facility instance as a stable seventh slot or add it to the six-key `plugin_backends` contract · conflate it with the emotion slot.

**Read next**: [MODULE_MAP §10 facility ①](../../../handoff/MODULE_MAP_AND_HANDOFF.md) · [complex-emotion](../facilities/complex-emotion.md) · [`emotion.rs`](../../../kernel/crates/oclive_kernel_types/src/models/emotion.rs).
