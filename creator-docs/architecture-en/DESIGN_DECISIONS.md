# oclive architecture decision record (ADR summary)

Key trade-offs distilled for contributors and host integrators. Layering rules: [`handoff/ARCHITECTURE_LAYERING.md`](../../handoff/ARCHITECTURE_LAYERING.md).

---

## 1. Blueprint does not drive main orchestration order

| Decision | Rationale |
|----------|-----------|
| **No ordinary Stable execution DSL from `pipeline.ocblueprint`** | Keeps disk configuration and [`process_message`](../../kernel/crates/oclive_kernel_host/src/domain/chat_engine/process_message.rs) / [`co_present`](../../kernel/crates/oclive_kernel_host/src/domain/chat_engine/turn_pipeline/mod.rs) **in sync**. Blueprints provide `slot_registry`, read-only `groups`, and Stable-v4 `runtime_config` / `extensions`; only frozen-v3 dual-core Beta `pipeline.experimental` is a bounded experimental DAG. |

---

## 2. Anti-corruption layer: traits in `oclive_kernel_contracts`

| Decision | Rationale |
|----------|-----------|
| **Ports live in contracts crate** | Hosts (desktop, headless, embedded) share the same abstractions; `distros/desktop-tauri/domain/ports/` re-exports only. |

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
| llm | Non-streaming: default `ensemble` serial last-wins, `fastest`, or ordered `fallback`; streaming currently serial last-wins | Trade resource use against availability without interleaving streamed tokens |
| emotion / event / prompt / complex_emotion | Serial last-wins | State / final text semantics |
| agent (directory) | Execution merge not implemented | The host currently records merged directory ids for diagnostics, while `wrap_agent_if_merged` returns the original provider; see `K-AGENT-MERGE-01` |

See [`slot_runner.rs`](../../kernel/crates/oclive_kernel_host/src/domain/slot_runner.rs).

---

## 6. C1 thin wrappers (session API transition)

| Decision | Rationale |
|----------|-----------|
| **Legacy command signatures delegate to slot overrides** | One release cycle for downstream launchers; prefer `set_session_slot_override` in new code. |

---

## 7. Blueprint load pipeline

```text
pipeline.ocblueprint → exact v2/v3/v4 dispatch → Role configuration → PluginHost → SlotResolver → SlotRunner
```

See the [`storage` module](../../kernel/crates/oclive_kernel_host/src/infrastructure/storage/mod.rs) docs.

---

[中文](../architecture/DESIGN_DECISIONS.md)
