# 01 · Architecture (simple)

> **Last updated:** 2026-09-04
> **Next:** [02 Thirty-minute start](02_THIRTY_MINUTE_START.md) · **Full human edition (CN):** [human-docs/01](../human-docs/01_ARCHITECTURE_SIMPLE.md) · **Module registry:** [MODULE_MAP](../handoff/MODULE_MAP_AND_HANDOFF.md)

Keep two layers separate: the **minimal conceptual core** is one turn/lifecycle orchestration and authority boundary plus six stable capability ports. The flow below is the **complete reference runtime**, so it also shows Event Ring, persistence, facilities, and hosts.

## One turn (co-present path)

```
UI → Tauri/HTTP → process_message → optional Agent shortcut
   → turn_pipeline (pre → middle → LLM → post) ↔ Event Ring
   ← PluginHost (six slots)
```

- **Orchestration SSOT:** `kernel/crates/oclive_kernel_host/.../process_message.rs`
- **Blueprint `steps[]` does not schedule** the first turn — Rust code does.

Main source anchors: `turn_pipeline/pre.rs` → `turn_pipeline/co_present/run_middle.rs` → `turn_pipeline/post.rs` → `turn_pipeline/post/post_llm.rs`.

## Context and authority

Prompt context combines the typed current input, static role-pack content, runtime character state, retrieved memory/knowledge, and this turn's derived affect/event/recollection results. Module 4 assembles facts, candidates, and committed state admitted for this turn; admission does not make every semantic interpretation true. Module 5 generates the reply; post-processing edits the final output rather than re-deciding memory or events.

Remember: **six slots provide capabilities; Event Ring carries trusted events; decision modules admit proposals; Rust orchestration applies and persists results.**

## Three memory stores (do not conflate)

| Store | Purpose | In prompt? |
|-------|---------|------------|
| Chat log (`chat_messages`) | UI history, export, replay source | **No** |
| Short-term (`short_term_memory`) | Recent-turn buffer | **Yes** |
| Long-term (`long_term_memory`) | AI archive, decay | **Yes** |

Deleting chat **does not** clear memory tables. Details: [CHAT_STORAGE_ARCHITECTURE](../handoff/CHAT_STORAGE_ARCHITECTURE.md).

## Six slots + non-slots

| Slots | `memory` · `emotion` · legacy dialogue-impact `event` · `prompt` · `llm` · `agent` |
| Facilities | Complex emotion, expert routing, portrait, visual stage — **not** slot keys |
| Side channels | User identity, reply post-process, theater director API |

Per-slot definitions: [MODULE_MAP §4–§9](../handoff/MODULE_MAP_AND_HANDOFF.md).

The six slots are stable categories, not a claim that all six implementations run on every path. Today the co-present health gate requires `prompt + llm`; memory, emotion, event, and agent may use their defined `none` / Noop behavior. An ordinary extension does not become “slot 7”; changing this taxonomy is a versioned breaking contract revision.

## Configuration authority

```text
role content (persona, scenes)
  → blueprint slot_registry (six slots and engine policy)
    → distro.oclive.toml (HostProfile and capability ceiling)
      → SessionCache overrides (temporary; no pack write and no SQLite persistence)
```

These are first of all four maintenance surfaces, not four copies of identical fields that always override each other. Role content and the in-pack blueprint have separate responsibilities inside one role pack. Overlapping values follow the effective-resolution chain, and no later layer can exceed host capability or safety limits. Basic role creators edit only role content. Advanced authors or host administrators own the blueprint. See [ROLE_PACK_BOUNDARY](../handoff/ROLE_PACK_BOUNDARY.md).

## Crate layers (principal dependency direction)

```text
desktop-tauri / oclive-kernel-server
  → depends on oclive_kernel_host
    → depends on oclive_kernel_runtime
      → depends on oclive_kernel_contracts
        → depends on oclive_kernel_types
```

This is the principal chain; direct dependencies on types, validation, and other support crates are omitted.

## Event Ring (not a seventh slot)

Event Ring is a reusable facility in the current complete reference runtime, not a prerequisite of the minimal core. Registered modules submit `EventDraft` proposals. Event Ring assigns trusted source, registry influence, order, correlation, and causation in an `EventEnvelope`. Influence is a proposal's base input to a decision module, not execution priority or automatic admission.

The “ring” is the routing boundary of one dispatch, not a permanent loop that polls modules in turn. A future cross-turn, cross-channel [Runtime Event Stream](../creator-docs-en/rfc/RFC_RUNTIME_EVENT_STREAM.md) must not replace Event decisions, Rust state-commit authority, or the Stable turn pipeline.

Two current chains are: legacy event impact → Ring → personality/relation preview; and memory candidate → Event decision → optional one-turn recollection context. A trusted sensor/system proposal may produce a one-shot `ProactiveTurnPermit`, but external observations do not impersonate user messages or gain user-persistence semantics. Contract: [EVENT_RING](../creator-docs-en/plugin-and-architecture/EVENT_RING.md).

## Turn Thinking (orchestration, not a slot)

Each co-present turn picks **Fast** or **Deep** before the main LLM chain (`turn_thinking.rs` + `co_present`).

| Layer | Config | Effect |
|-------|--------|--------|
| **Wave E** | `distro.oclive.toml` → `[turn_thinking] fast_persistence` | `strong_only`: Fast casual chat skips LTM / favor / profile evolution; strong events still persist |
| **Wave F** | `config.json` → `turn_thinking` | OR/AND Deep rules, latch, ephemeral situation summary (TTL) |

Chat log rows are **always** written. No player Fast/Deep toggle. Details: [RFC summary](../creator-docs-en/rfc/RFC_TURN_THINKING_PERSISTENCE_SUMMARY.md) · [Chinese RFC](../creator-docs/rfc/RFC_TURN_THINKING_PERSISTENCE.md).

## Key IDs

| Term | Meaning |
|------|---------|
| `mrid` | Manifest role id (pack folder) |
| `srid` | Session namespace id |
| `reply` | Response field name (not `response`) |

Glossary: [03_GLOSSARY.md](03_GLOSSARY.md)
