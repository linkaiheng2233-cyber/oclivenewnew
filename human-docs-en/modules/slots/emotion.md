# Slot pack · `emotion` (EN summary)

> Full checklist (ZH): [`human-docs/modules/slots/emotion.md`](../../../human-docs/modules/slots/emotion.md)
> Scope: developer entry points and checklist. Public entry points: [MODULE_MAP §3.1.1](../../../handoff/MODULE_MAP_AND_HANDOFF.md#311-base-实现者接入点清单一页); reference Host definitions: [§5](../../../handoff/MODULE_MAP_AND_HANDOFF.md#5-第-2-模块--emotion).
> Updated: 2026-10-09.

## Choose your entry point

Start below for basic clue analysis. Use **Reference Host** for `UserEmotionAnalyzer`, `EmotionResult` or configured backends; rich results and facility rules are not automatically prerequisites for every Base implementation.

### EmotionBase: consume clues without applying Host state

1. Find `EmotionBase::analyze` in the [public contract](../../../kernel/crates/oclive_kernel_contracts/src/slot_base.rs). Read the existing [`KeywordEmotionBase`](../../../kernel/crates/oclive_kernel_runtime/src/domain/base_emotion.rs) agreement and lexicon limits; this implementation returns `Unsupported` for non-empty extra context.
2. Follow the current-material delegation in [`MinimalRoleBaseConsumer`](../../../kernel/crates/oclive_kernel_runtime/src/domain/minimal_role_consumer.rs). The reference Host's builtin analyzer also has a Base view. Its [`minimal conversation entry`](../../../kernel/crates/oclive_kernel_host/src/domain/chat_engine/process_message.rs) uses clues as reference material, without converting reports into seven-dimensional scores or directly applying state.
3. See [`base_only_fixture.rs`](../../../kernel/crates/oclive_kernel_contracts/tests/base_only_fixture.rs) for an independent implementation shape. Authors can provide their own analysis agreement rather than copying the lexicon; the fixture does not establish real semantic quality.

**Base checklist**:

- [ ] Distinguish normal absence from failure; keep complete clues and limits. A lexicon match is not a final judgment of subject, intent or actual emotion.
- [ ] The caller decides how to use results. Clues do not independently write emotion state, long-term memory or acquire new authority.

Definitions remain in the [entry-point checklist](../../../handoff/MODULE_MAP_AND_HANDOFF.md#311-base-实现者接入点清单一页). Advanced emotion-driven memory remains Deferred; this entry adds no mandatory Base interface.

## Reference Host

**You plug in**: blueprint `slot_registry.type: emotion` → folded `PluginBackends.emotion` (legacy v1: `settings.json.plugin_backends.emotion`) · trait `UserEmotionAnalyzer` · hook `turn_pipeline/pre.rs` → `EmotionResult` → Prompt · Turn Thinking Auto.

**Distinction**: This slot analyzes **user-utterance** emotion as a prompt clue or one fallback input. For the [complex-emotion facility](../facilities/complex-emotion.md), a valid main-LLM `[EMO]` is authoritative for current reply emotion, remote/directory is fallback only, and any usable hint is carried into the next turn (not a six-slot key).

**Do**: `builtin` · `remote` · `directory` · `none` backends · align pre output to `EmotionResult` · use `Emotion` enum from dto.

**Don't**: Treat a `slot_registry` `complex_emotion` facility instance as a stable seventh slot or add it to the six-key `plugin_backends` contract · conflate it with the emotion slot.

**Read next**: [MODULE_MAP §10 facility ①](../../../handoff/MODULE_MAP_AND_HANDOFF.md) · [complex-emotion](../facilities/complex-emotion.md) · [`emotion.rs`](../../../kernel/crates/oclive_kernel_types/src/models/emotion.rs).
