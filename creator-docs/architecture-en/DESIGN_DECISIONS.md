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

<a id="full-resilience-deferred"></a>

## 8. Preserve Minimal resilience; defer Full (2026-10-08)

| Decision | Basis and frozen scope |
|----------|------------------------|
| **Keep existing Minimal; Full is Deferred** | The maintainer preserves the accepted bounded consolidation. An undefined Full label does not justify inventing a general layer. Deferred means neither completed nor permanently cancelled; this decision adds no implementation. Current status belongs to the [debt inventory](../../handoff/TECHNICAL_DEBT_INVENTORY.md). |
| **Preserve the small Kernel / Host authority boundary** | Follow the [existing responsibilities](../../handoff/MODULE_MAP_AND_HANDOFF.md#kernel-responsibilities): Kernel protects hard contract, authorization-validity, necessary causal and terminal boundaries; Host handles distro faults and environment adaptation within them. No new public API or architectural authority is introduced. |
| **Do not introduce execution policies automatically** | No new automatic retries, circuit breaking, module takeover or degradation. Existing Agent, tool, six-slot and accepted Remote behavior remains. Existing policies continue under their implementation and authorization; deferral does not remove them. |

**Reassessment conditions:** Full may be reassessed only for a concrete reproducible failure, a clear reliability requirement that existing Minimal cannot meet, and an identifiable goal, impact scope and semantic risk. Reassessment does not authorize implementation. New execution semantics, contracts or allocation of architectural authority still require maintainer confirmation. Candidates without these prerequisites remain recorded; closing more debt is not permission to resume Full.

See the [freeze entry and historical Minimal plan](../../handoff/debt-marathon/long-plans/K-RESILIENCE-01.md#full-deferred-freeze) and [DCL-65](../../handoff/debt-marathon/DEBT_CHANGELOG.md#dcl-20261008-65--保持-minimal并将-full-韧性暂缓) for execution and handoff.

---

<a id="emotion-memory-extension-deferred"></a>

## 9. Emotion-driven long-term memory: defer implementation, retain optional extensions (2026-10-09)

The maintainer keeps **K-EMO-06 Deferred**. Existing basic emotion and memory capabilities are considered sufficient in principle for the current-stage goals; the core project prioritizes their concrete defects rather than developing advanced emotion-driven long-term-memory policies. This is a scope decision, not evidence of complex-emotion quality, every scenario or long-term effects. Deferred means neither completed, implemented nor permanently cancelled. Current status belongs to the [debt inventory](../../handoff/TECHNICAL_DEBT_INVENTORY.md).

**Retained direction:** third-party modules or distros may explore emotion-based memory selection, importance or weighting as optional extensions. Whether intense content should be retained preferentially, or a temporary complaint should enter long-term memory, is an extension's product policy rather than a minimum obligation for every Memory or Emotion implementation. Extensions remain subject to existing public contracts and actual authorization. Recording this direction proves neither universal plug-in compatibility nor permission to read or write Host long-term storage automatically.

**Boundary:** this direction is outside the small Kernel's core responsibilities. It adds no mandatory six-slot Base methods, fields or default policies, and extension output acquires no domain-commit authority. Follow the existing [Kernel / Host responsibilities](../../handoff/MODULE_MAP_AND_HANDOFF.md#kernel-responsibilities) and [Base / Extension boundary](../../handoff/MODULE_MAP_AND_HANDOFF.md#six-slot-base-extension). Existing emotion, memory, Agent, tool and six-slot execution remains unchanged.

**Reassessment conditions:** a concrete reproducible integration case must demonstrate that an existing public contract obstructs a specific advanced-module or distro need. Record the obstructing interface, expected capability, impact scope and minimal counterexample before assessing a contract adjustment. Research value or a missing advanced algorithm alone does not unfreeze implementation. New execution semantics, authority or mandatory public obligations still require maintainer confirmation. Stop investigation once the direction, scope and counterexample suffice for a decision; do not enumerate every future policy.

---

[中文](../architecture/DESIGN_DECISIONS.md)
