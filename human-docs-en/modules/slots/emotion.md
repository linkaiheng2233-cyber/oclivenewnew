# Slot pack · `emotion` (EN summary)

> Full checklist (ZH): [`human-docs/modules/slots/emotion.md`](../../../human-docs/modules/slots/emotion.md)
> Definition SSOT: [MODULE_MAP §5](../../../handoff/MODULE_MAP_AND_HANDOFF.md)

**You plug in**: `plugin_backends` key `emotion` · trait `UserEmotionAnalyzer` · hook `turn_pipeline/pre.rs` → `EmotionResult` → Prompt · Turn Thinking Auto.

**Distinction**: This slot analyzes **user utterance** emotion; [complex-emotion facility](../facilities/complex-emotion.md) consumes emotion output → `narrative_hint` (not a sixth-slot key).

**Do**: `builtin` · `remote` · `directory` · `none` backends · align pre output to `EmotionResult` · use `Emotion` enum from dto.

**Don't**: Register `complex_emotion` in `slot_registry` as a slot · conflate with the complex-emotion facility.

**Read next**: [MODULE_MAP §10 facility ①](../../../handoff/MODULE_MAP_AND_HANDOFF.md) · [complex-emotion](../facilities/complex-emotion.md) · [`emotion.rs`](../../../kernel/crates/oclive_kernel_types/src/models/emotion.rs).
