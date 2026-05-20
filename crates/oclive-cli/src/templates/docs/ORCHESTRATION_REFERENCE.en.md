# Orchestration reference (kernel developers)

This document describes the **six-stage logical pipeline** aligned with the main app’s `process_message`, and how **Monolith** welding can omit slots.

> **Desktop host**: Tauri / `--api` HTTP in oclivenewnew uses a **fixed** `process_message` path in `src-tauri/src/domain/chat_engine/mod.rs`. This file is for **headless / Monolith** customization only.

## Six main stages

1. **Load context** — role pack, session, recent turns (`load_context`)
2. **Emotion & events** — emotion analysis, event estimation (`analyze_emotion` / `detect_event`)
3. **Memory** — retrieval and ranking (`retrieve_memory`)
4. **Prompt** — assemble system/user messages (`build_prompt`)
5. **LLM** — primary generation (`call_llm`)
6. **Post-process** — persistence, narrative hints, etc. (`post_process`)

Generated **`src/process_message_monolith.rs`** is driven by `monolith.toml` and demonstrates seven slots (memory, emotion, event, prompt, llm, agent, complex_emotion) as static builtin calls or trait stubs.

## Steps that may be reordered safely

In a **custom** `process_message`, these may be swapped when data dependencies still hold:

| Group | Notes |
|-------|--------|
| `analyze_emotion` ↔ `detect_event` | Both mainly consume the current user turn |
| Memory vs emotion/event | If retrieval does not need emotion labels, memory can run earlier |

## Hard constraints

| Rule | Reason |
|------|--------|
| **`build_prompt` before `call_llm`** | LLM needs complete messages |
| **`load_context` before state-dependent steps** | Prompt needs role/session |
| **`post_process` after `call_llm`** | Writes depend on model output |

## Skipping a slot via `monolith.toml`

Under `[monolith]`:

- **`weld_modules`**: slots to **statically weld** at compile time.
- **`exclude`**: mutually exclusive with `weld_modules`; slots kept dynamic.

Init flags such as `--monolith-preset embedded` pre-fill `weld_modules`; edit the file then run **`oclive build`** to regenerate `process_message_monolith.rs`.

Slots in neither list use the trait stub path (smaller binary, higher per-call overhead).

## Edit points

| Artifact | Role |
|----------|------|
| `monolith.toml` | Source of weld plan |
| `src/process_message_monolith.rs` | Generated; do not hand-edit weld blocks |
| `vendor/oclive_monolith_builtin/` | Stubs; replace with real `oclive_*_builtin` crates |

See `creator-docs/rfc/RFC_OCLIVE_MONOLITH_MODE.md` in the oclivenewnew repo.
