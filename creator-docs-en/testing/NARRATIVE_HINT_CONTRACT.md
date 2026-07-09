# `narrative_hint` end-to-end contract (AB1)

[中文](../../creator-docs/testing/NARRATIVE_HINT_CONTRACT.md)

**Status**: Aligned with co-present path in desktop-tauri and `oclive_kernel_runtime::PromptBuilder`.

## 1. Data shapes

| Stage | Type / storage | Field |
|-------|----------------|-------|
| Resolve input | `ComplexEmotionInput` | `previous_narrative_hint: String` (prior turn cache; empty on first turn) |
| Resolve output | `ComplexEmotionOutput` | `narrative_hint: String` |
| Session cache | `AppState::last_complex_emotion_narrative_hint` | `HashMap<srid, String>` (in-process, **not** SQLite) |
| Prompt input | `PromptInput::previous_complex_emotion_narrative_hint` | `&str` |

## 2. Call order (single turn `process_message` / co-present)

1. `load_recent_context`
2. Read `stored_complex_emotion_narrative_hint(srid)` → this turn's `ComplexEmotionInput.previous_narrative_hint`
3. `ComplexEmotionProvider::resolve_turn` → `ComplexEmotionOutput`
4. `build_prompt` (`previous_complex_emotion_narrative_hint` = step 2 snapshot)
5. Main dialogue LLM
6. `set_stored_complex_emotion_narrative_hint(srid, complex_emotion_out.narrative_hint)`

**Invariant**: Step 4 uses the **previous turn's** hint, not the hint just computed this turn.

## 3. Prompt injection (`PromptBuilder`)

- When `previous_complex_emotion_narrative_hint.trim().is_empty()`: **do not** emit 【复杂情感叙事提示】 section.
- When non-empty: fixed title line + trimmed body + double newline, then `用户说:` section.
- Title copy (ZH in product): narrative hint from prior-turn built-in analysis; do not recite meta to the user.

## 4. Automated verification

| Case | Location |
|------|----------|
| First turn: no narrative section in main prompt | `distros/desktop-tauri/tests/narrative_hint_contract_audit.rs` |
| Second turn injects prior hint | Same + `narrative_hint_prompt_roundtrip.rs` |
| Third consecutive turn contains section | `narrative_hint_contract_audit.rs` |
| Empty / special chars don't break structure | `oclive_kernel_runtime` `prompt_builder` unit tests |

## 5. Remote sidecar

Remote `complex_emotion.resolve_turn` JSON must match `ComplexEmotionOutput`; on degradation `degraded_to_builtin: true`. Sidecar errors: [ERROR_CODES.md § layered boundary](../getting-started/ERROR_CODES.md).
