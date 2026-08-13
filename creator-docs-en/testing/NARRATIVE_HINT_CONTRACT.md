# `narrative_hint` end-to-end contract (Plan A / B M1)

[中文](../../creator-docs/testing/NARRATIVE_HINT_CONTRACT.md)

**Status**: Aligned with the co-present path in `oclive_kernel_host`, `oclive_kernel_runtime::PromptBuilder`, and desktop integration document v1.22.

## 1. Data shapes and storage

| Stage | Type / storage | Contract |
|-------|----------------|----------|
| Main LLM output | `[EMO]...[/EMO]` | `labels[]` plus optional `narrative_hint`; markers are removed before the reply reaches the user |
| Plugin fallback output | `ComplexEmotionOutput` | A remote or directory plugin supplies `labels[]` and `narrative_hint` when the marker is missing or invalid |
| Persistence | SQLite `complex_emotion_hint` plus session cache | Stored by `srid` with a 24-hour TTL; the cache is not the only source of truth |
| Prompt input | `PromptInput::previous_complex_emotion_narrative_hint` | Only the previously persisted turn hint is injected |

Length uses Unicode character count. The prompt asks the model to stay within 150 characters; the host enforces a **200-character** hard cap at marker parsing, plugin output handling, and final persistence without splitting a multibyte character.

## 2. Backend gate matrix

| Effective `complex_emotion` slot | Read / inject old hint | Write new hint | Emotion label source |
|----------------------------------|------------------------|----------------|----------------------|
| Omitted or `none` | No | No | Valid `[EMO]` labels may still update the bot's six-slot emotion state |
| `builtin` | Yes | Yes | Prefer a valid `[EMO]`; retain degraded output when invalid |
| `remote` / `directory` | Yes | Yes | Prefer a valid `[EMO]`; otherwise use plugin `labels[]` to update the bot's six-slot emotion state |

Duplicate slot declarations retain the registry's last-wins semantics. `none` disables complex-emotion hint reads and writes; it does not discard valid emotion labels produced by the main LLM.

## 3. Single-turn call order (`process_message` / co-present)

1. Determine the effective complex-emotion backend from the role's slots.
2. Read `stored_complex_emotion_narrative_hint(srid)` only for `builtin`, `remote`, or `directory`; treat expired data as empty.
3. `build_prompt` injects the prior-turn snapshot from step 2. Omitted or `none` always receives an empty value.
4. The main dialogue LLM produces reply text and an optional `[EMO]` marker.
5. Parse and remove all emotion markers from user-visible text. From an unclosed marker start through the end of the reply is also stripped to prevent internal protocol leakage.
6. A valid marker wins; otherwise remote or directory uses its plugin output. Effective `labels[]` drives current bot emotion and its event, and the effective hint is capped at 200 characters.
7. Persist the hint only for an enabled backend. Omitted or `none` must not read, inject, clear, or create a hint.

**Cross-turn invariant**: The current prompt may use only the previously stored hint, never the hint parsed from the current reply.

## 4. Parsing and degradation rules

- With multiple complete markers, the last valid marker wins; every marker is removed from the reply body.
- A trailing unclosed marker invalidates that marker attempt and strips the unclosed tail so `[EMO]` or JSON fragments cannot leak to the user.
- For an enabled backend, a missing or invalid marker retains the previous hint; a valid marker with an absent or blank `narrative_hint` clears the stored hint.
- Remote and directory plugin output shares the same final length and persistence boundaries as a main-LLM marker.
- The Fast path does not generate or write a new hint. When the backend is enabled, an existing hint may still be included in the prompt.

## 5. Prompt injection (`PromptBuilder`)

- When `previous_complex_emotion_narrative_hint.trim().is_empty()`, do not emit the 【复杂情感叙事提示】 section.
- Otherwise insert the fixed title, trimmed body, and a blank line before the `用户说:` section.
- Product title copy: `【复杂情感叙事提示】（上一回合内置分析输出；自然落实，勿向用户复述本段标题或元信息）`.

## 6. Automated verification

| Case | Location |
|------|----------|
| First-turn omission, prior-hint injection, three-turn propagation, empty and special-character handling | `distros/desktop-tauri/tests/narrative_hint_contract_audit.rs`, `narrative_hint_prompt_roundtrip.rs` |
| `none` does not read or write but labels still apply; remote labels drive six slots; plugin hint truncation | `distros/desktop-tauri/tests/complex_emotion_backend_contract.rs` |
| Unclosed-marker stripping, last-valid-marker selection, Unicode 200-character cap | Unit tests in `kernel/crates/oclive_kernel_host/src/domain/emo_marker.rs` |
| SQLite plus session cache, 24-hour TTL, defensive persistence cap | Unit tests in `kernel/crates/oclive_kernel_host/src/domain/complex_emotion_store.rs` |
| Prompt structure for empty and special-character values | `oclive_kernel_runtime` `prompt_builder` unit tests |

## 7. Remote sidecar

Remote `complex_emotion.resolve_turn` JSON must match `ComplexEmotionOutput`; on degradation it must set `degraded_to_builtin: true`. See [ERROR_CODES.md § layered boundary](../getting-started/ERROR_CODES.md) for sidecar errors.
