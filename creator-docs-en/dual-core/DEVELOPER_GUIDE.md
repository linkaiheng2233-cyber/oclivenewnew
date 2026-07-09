# Dual-core dual-mode developer guide

[中文](../../creator-docs/dual-core/DEVELOPER_GUIDE.md)

## Overview

**Dual-core dual-mode** provides two runtime paths:

| Core | Source | Behavior |
|------|--------|----------|
| **Stable core** | Always `process_co_present` | Today’s default single-core orchestration |
| **Experimental core** | `pipeline.experimental` | Runs implemented `action` steps in DAG order; on failure **silently falls back** to stable core |

Gating (all required):

```text
runtime_config.dual_core.enabled == true
AND pipeline.experimental is non-empty
```

Otherwise **zero difference** from dual-core off.

---

## Enabling dual-core

### Scaffold

```bash
cargo run -p oclive-cli -- init --dual-core --preset full -o ./my-kernel
```

Generates `distros/chat-pro/roles/default/pipeline.ocblueprint` (`schema_version: 3`, with `runtime_config` and `pipeline`).

### Hand-written blueprint

1. Use `schema_version: 3`.
2. Set `runtime_config.dual_core.enabled: true`.
3. Fill `pipeline.experimental` (see [METHOD_REGISTRY.md](./METHOD_REGISTRY.md)).
4. `pipeline.stable` may serve as documentation; **the host does not execute it**.

Validate:

```bash
cargo run -p oclive-cli -- pack validate --profile creator ./distros/chat-pro/roles/your_role
```

---

## Writing the experimental pipeline

1. In `slot_registry`, prepare a valid `registry_key` for each `action`.
2. Instance `type` must match the method (e.g. `emotion` + `analyze`).
3. Use `depends_on` for in-pipeline dependencies; no cycles.
4. End with `slot.<llm_key>.generate`, or let Agent `process` short-circuit.

Example:

```json
"pipeline": {
  "stable": [],
  "experimental": [
    { "action": "slot.emotion.analyze", "depends_on": [] },
    { "action": "slot.memory.retrieve", "depends_on": ["slot.emotion.analyze"] },
    { "action": "slot.llm.generate", "depends_on": ["slot.memory.retrieve"] }
  ]
}
```

---

## Registering new slot instances

Add instances in `slot_registry`; `action` references the **key name**, not `type`. v3 optional `zone: "experimental"` limits instances to the experimental pipeline (see blueprint v3 validation).

---

## Debugging and fallback

Experimental failure shows **no user-visible fallback**; use `tracing` to diagnose.

### Viewing fallback logs

PowerShell (desktop host or `oclivenewnew-tauri --api`):

```powershell
$env:RUST_LOG = "info,oclive_dual_core=info"
# or dual-core only: $env:RUST_LOG = "oclive_dual_core=info"
```

Typical log sequence (`target=oclive_dual_core`):

| Level | Meaning |
|-------|---------|
| `INFO` starting experimental core, `step_count=N` | Entering experimental pipeline |
| `INFO` experimental core succeeded | All experimental steps done (Agent short-circuit or handoff to stable core) |
| `WARN` experimental core failed at step X: …, falling back to stable core | `action` / method / co-present sub-stage failed; error stage `dual_core_experimental` |
| `INFO` stable core finished (fallback mode) | Snapshot rolled back; `co_present` returned reply |

Filter example:

```powershell
# With default tracing fmt layer, search terminal for:
# oclive_dual_core
```

- OOCP optional scenario `S13_dual_core_fallback` (`OCLIVE_OOCP_INCLUDE_S13=1`).

### Snapshot rollback (captured before experiment)

| Field | Description |
|-------|-------------|
| `narrative_hint` | Complex-emotion narrative cache |
| `emotion_state` | `get_current_emotion` |
| `active_scene_id` | `get_user_presence_scene` / `set_user_presence_scene` |

If experimental steps mutate these and then fail, state rolls back before stable core runs.

---

## Contributing a new method mapping

1. Register `(type, method)` in `kernel/crates/oclive_kernel_host/src/domain/dual_pipeline_registry.rs`.
2. Implement aligned co-present sub-step calls in `dual_pipeline_steps.rs`.
3. Update [METHOD_REGISTRY.md](./METHOD_REGISTRY.md) and `oclive explain`.
4. Add unit tests.

---

## FAQ

**Q: Does `pipeline.stable` run?**  
A: No. Stable core is always hard-coded `co_present`.

**Q: `enabled=true` but `experimental=[]`?**  
A: Treated as dual-core off; uses `co_present`.

**Q: Can Monolith builds use dual-core?**  
A: `oclive init --monolith --dual-core` writes `[dual_core]` in `monolith.toml`; when linked into the main repo, `DualPipelineRunner` still schedules.

**Q: Can creator packs enable dual-core by default?**  
A: Do not ship `enabled: true` alone in distribution packs; see [ROLE_PACK_BOUNDARY.md](../../handoff/ROLE_PACK_BOUNDARY.md).

---

## Related docs

- Method registry: [METHOD_REGISTRY.md](./METHOD_REGISTRY.md)
- RFC: [RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md](../rfc/RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md)
