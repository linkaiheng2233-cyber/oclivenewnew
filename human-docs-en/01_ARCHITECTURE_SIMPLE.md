# 01 · Architecture (simple)

> **Last updated:** 2026-09-11
> **Next:** [02 Thirty-minute start](02_THIRTY_MINUTE_START.md) · **Full human edition (CN):** [human-docs/01](../human-docs/01_ARCHITECTURE_SIMPLE.md) · **Module registry:** [MODULE_MAP](../handoff/MODULE_MAP_AND_HANDOFF.md)

Keep two layers separate: the **minimal conceptual core** is the six-slot capability contract and necessary public legality boundary. The flow below is the **complete reference runtime**, so it also shows Event Ring, persistence, facilities, and hosts.

The Host is more than a communication bridge: the small Kernel owns the six-slot contracts, necessary public legality constraints, and error boundaries; the trusted Rust Host in the current reference implementation performs product-state commits and resource operations within those constraints. Tauri APIs and HTTP are transport adapters. The current `oclive_kernel_host::OcliveKernel` is a complete reference-runtime facade that still physically assembles state, SQLite, plugins, and facilities; this does not mean a small Core crate has been extracted. The unique responsibility SSOT is [MODULE_MAP_AND_HANDOFF.md](../handoff/MODULE_MAP_AND_HANDOFF.md).

For Chat Pro, Tauri API/HTTP is the transport surface while the runtime assembles `AppState`, state, and permissions; `distros/desktop-tauri/src/api/chat_backend.rs` identifies the loopback kernel as the single authoritative writer. This example does not imply one Host per distro.

Start with the [Kernel/Host responsibilities and candidate boundary summary](../handoff/MODULE_MAP_AND_HANDOFF.md#kernel-responsibilities), then use the [focused source comparison](../handoff/MODULE_MAP_AND_HANDOFF.md#kernel-source-map) to separate responsibility targets from current implementation. The flows here teach the current ChatPro Host.

## One turn (current co-present Host path)

The [SVG](../human-docs/assets/oclive-architecture-learning-map.svg) / [PNG](../human-docs/assets/oclive-architecture-learning-map.png) remain learning snapshots of the **complete reference Host**. Older “kernel” labels there do not put orchestration, SQLite, memory, or Event Ring inside the small Kernel. The assets were not regenerated in this documentation alignment.

```
UI → Tauri/HTTP → process_message → optional Agent shortcut
   → turn_pipeline (pre → middle → LLM → post) ↔ Event Ring
   ← PluginHost (six slots)
```

- **Orchestration SSOT:** `kernel/crates/oclive_kernel_host/.../process_message.rs`
- **Blueprint `steps[]` does not schedule** the first turn — the current Rust Host does.
- **Agent shortcut:** `handled=true` sends this Host to minimal-response construction instead of the ordinary main chain; it alone does not establish the end of a public invocation. Continuing the ordinary branch does not prove that no tools ran earlier.

Main source anchors: `turn_pipeline/pre.rs` → `turn_pipeline/co_present/run_middle.rs` → `turn_pipeline/post.rs` → `turn_pipeline/post/post_llm.rs`. This is the current reference Host path, not a fixed pipeline required of every Host.

## Context and authority

Prompt context combines the typed current input, static role-pack content, runtime character state, retrieved memory/knowledge, and this turn's derived affect/event/recollection results. Module 4 assembles facts, candidates, and committed state admitted for this turn; admission does not make every semantic interpretation true. Module 5 generates the reply; post-processing edits the final output rather than re-deciding memory or events.

For the current reference runtime: **six slots provide capabilities; Event Ring carries trusted events; decision modules admit proposals; Rust Host orchestration applies domain results.** Domain conditions and commit responsibility belong to the Host; public invocation constraints are not a guarantee of domain-commit success. Transport and UI do not gain domain authority.

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

This is the principal chain of the current reference runtime; direct dependencies on types, validation, and other support crates are omitted. It is not a physical extraction diagram or a new future scheduling contract for the minimal Core.

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
