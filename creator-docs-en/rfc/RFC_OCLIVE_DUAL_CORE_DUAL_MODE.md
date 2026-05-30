# RFC: Runtime dual-core dual-mode (Stable · Experimental)

| Field | Value |
|-------|--------|
| Status | **Opt-in Beta (default off)** — P2–P5 path is merged; Stable remains the default delivery path |
| Entry | **`oclive init --dual-core`** (opt-in; **off by default**) |
| vs Monolith | **Orthogonal**: Monolith = compile-time weld; dual-core = runtime dual pipelines + rollback |

[中文全文](../rfc/RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md) · [Cursor handoff (progress)](../../handoff/DUAL_CORE_CURSOR_HANDOFF.md) · [Alignment quick ref](../../handoff/DUAL_CORE_ALIGNMENT.md)

---

## Terminology

| Term | Layer | Today |
|------|--------|--------|
| **Single-kernel dual-mode build** | Compile-time | **Yes** — PluginHost vs `monolith` feature ([RFC_OCLIVE_MONOLITH_MODE.md](RFC_OCLIVE_MONOLITH_MODE.md)) |
| **Dual-core dual-mode (this RFC)** | **Runtime** | **Opt-in Beta** — Stable + Experimental pipelines (default off) |

Do not conflate **build modes** with **runtime cores**.

---

## Summary

- **Stable core**: Only six slot `type`s (`memory` … `agent`); **fixed** stage order (today’s co-present path).
- **Experimental core**: Arbitrary `type`s (e.g. `intent_recognition`); order from `pipeline.experimental` + **`depends_on` DAG** (validated at load).
- **One blueprint**: `slot_registry` is the **master table** (not split per core); `zone` is a string or **array** — an instance **may belong to both** stable and experimental.
- **DualPipelineRunner**: Experimental first with **`SessionState` snapshot**; on failure, restore and run `pipeline.stable`; degradation **reuses** Remote→builtin fallback patterns (no new error framework).
- **Shared backend pool**: No core-specific backends; register traits in `slot_registry`, both cores may use the same instance.
- **Default off**: No `--dual-core` ⇒ zero behavior change.
- **Monolith**: `--monolith` without dual-core = zero dual-pipeline overhead (shipping minimal kernel); `--monolith --dual-core` = welded pipelines + runner (dev high-perf lab).

**Q15–Q20 (decided)**: `runtime_config.dual_core.enabled`; schema 2/3 split load; P1 registry-key-only; migration tool deferred; empty `pipeline.stable` → `co_present`; P4 seven PluginHost types only.

**Progress**: P1 validation in `oclive_validation`; P2+ scheduler not wired. See [DUAL_CORE_CURSOR_HANDOFF.md](../../handoff/DUAL_CORE_CURSOR_HANDOFF.md) · [ROLE_PACK_BOUNDARY.md](../../handoff/ROLE_PACK_BOUNDARY.md).

Current delivery stays on **v2 single stable path** until host integrates dual-core.
