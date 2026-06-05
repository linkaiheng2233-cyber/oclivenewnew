# Optimization Phase 5 — Design Decisions (2026-06-05)

Aligned during the oclive optimization plan (phases 1–4). **No code changes required** unless a future milestone explicitly adopts an outcome below.

## dual_core — promotion / retirement criteria

**Current state:** ~1.5k lines feature-gated behind `dual_core` (default **off**); CI S13/S14 in OOCP suite when `--include-dual-core`.

**Decision:** Keep the feature gate until **one** of the following is true (whichever comes first):

| Criterion | Target |
|-----------|--------|
| **Product opt-in** | Stable channel ships an explicit “Experimental dual pipeline” toggle with docs; ≥2 release cycles without P0 regressions in S13/S14 |
| **CoPresent parity** | `dual_pipeline*` paths reach functional parity with `CoPresent` for: memory write, favorability, event log, and prompt assembly (checklist in `handoff/DUAL_CORE_CURSOR_HANDOFF.md`) |
| **Retirement** | No opt-in usage telemetry / issue mentions for **6 months** *and* roadmap defers dual-core → **remove gate** in a breaking minor, document in `CHANGELOG` |

**Until then:** Do not expand co_present-only behavior without checking dual_core compile (`cargo build -p oclivenewnew-tauri --features dual_core`).

**Owner doc:** [`creator-docs/rfc/RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md`](../creator-docs/rfc/RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md) · progress [`handoff/DUAL_CORE_CURSOR_HANDOFF.md`](DUAL_CORE_CURSOR_HANDOFF.md)

---

## `ports/` ceremony — SlotRegistryResolver vs PluginHostPort

**Current state:**

- `PluginHostPort` — used for test doubles / HTTP attach boundaries; **keep** as `dyn` port.
- `SlotRegistryResolver` — implemented on `SlotResolver` but **never invoked through `dyn SlotRegistryResolver`**; all call sites use `SlotResolver::resolve*` statically.

**Decision:**

1. **Short term (done in spirit):** Treat `SlotResolver` as a plain module; do not add new `dyn SlotRegistryResolver` consumers.
2. **Optional cleanup (non-blocking):** In a dedicated refactor PR, remove `impl SlotRegistryResolver for SlotResolver` and the re-export from `domain/ports/` if `oclive_kernel_runtime` contract no longer requires it for external crates.
3. **Do not** remove `PluginHostPort` — it remains the stable seam for kernel attach and integration tests.

**Rationale:** Reduce indirection where no runtime substitution exists; preserve ports that genuinely support mocking and cross-crate boundaries.

---

## Related completed items (phases 1–4)

- Agent `directory` / `remote`: validation downgrade to `builtin` + audit logs (`oclive_validation::sanitize_unimplemented_agent_backend`).
- Directory HTTP clients: per-`(module, plugin_id)` cache in `BackendRegistry`.
- Memory merge: pool path wrapped in transaction; cross-host duplicate risk documented in `memory_merge.rs`.
- `event_type` persistence: unified on `EventType::as_ref()` (canonical `"Praise"`, not `Debug` formatting).
