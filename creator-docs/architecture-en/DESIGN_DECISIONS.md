# oclive architecture decision record (ADR summary)

Key trade-offs distilled for contributors and host integrators. Layering rules: [`handoff/ARCHITECTURE_LAYERING.md`](../../handoff/ARCHITECTURE_LAYERING.md).

---

## 1. Blueprint does not drive main orchestration order

| Decision | Rationale |
|----------|-----------|
| **No executable DSL from `pipeline.ocblueprint`** | Keeps on-disk flow and [`process_message`](../../crates/oclive_kernel_host/src/domain/chat_engine/process_message.rs) / [`co_present`](../../crates/oclive_kernel_host/src/domain/chat_engine/turn_pipeline/mod.rs) **in sync**; blueprint supplies `slot_registry` / `groups` only. |

---

## 2. Anti-corruption layer: traits in `oclive_kernel_contracts`

| Decision | Rationale |
|----------|-----------|
| **Ports live in contracts crate** | Hosts (desktop, headless, embedded) share the same abstractions; `src-tauri/domain/ports/` re-exports only. |

---

## 3. `module_relations` are derived, not authored

| Decision | Rationale |
|----------|-----------|
| **Do not persist `module_relations` in blueprint JSON** | Manual edges drift from `slot_registry`; frontend `buildBlueprintEdges` is the single source of truth. |

---

## 4. Blueprint `groups` are UI-only

| Decision | Rationale |
|----------|-----------|
| **Groups for creator UX** | Recreates v1 “six module” visual grouping; does not change resolver or merge order. |

---

## 5. Multi-instance merge strategies

| Slot | Strategy | Rationale |
|------|----------|-----------|
| memory | Serial merge + dedupe by id | Union of recalls, no duplicate injection |
| llm | Serial last-wins | One user-visible reply per turn |
| emotion / event / prompt / complex_emotion | Serial last-wins | State / final text semantics |
| agent (directory) | Merged in `PluginHost` | Combine tool sets when plugins are independent |

See [`slot_runner.rs`](../../crates/oclive_kernel_host/src/domain/slot_runner.rs).

---

## 6. C1 thin wrappers (session API transition)

| Decision | Rationale |
|----------|-----------|
| **Legacy command signatures delegate to slot overrides** | One release cycle for downstream launchers; prefer `set_session_slot_override` in new code. |

---

## 7. Blueprint load pipeline

```text
pipeline.ocblueprint → validate → Role.slot_registry → PluginHost → SlotResolver → SlotRunner
```

See [`storage.rs`](../../src-tauri/src/infrastructure/storage.rs) module docs.

---

[中文](../architecture/DESIGN_DECISIONS.md)
