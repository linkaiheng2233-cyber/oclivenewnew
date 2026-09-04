# RFC: Runtime dual-core dual-mode (Stable · Experimental)

[中文](../../creator-docs/rfc/RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md)

| Field | Value |
|-------|--------|
| Status | **Opt-in Beta (default off)** — P2–P5 path is merged; Stable remains the default delivery path |
| Entry | **`oclive init --dual-core`** (opt-in; **off by default**) |
| vs Monolith | **Orthogonal**: Monolith = compile-time weld; dual-core = runtime dual pipelines + rollback |

[中文全文](../../creator-docs/rfc/RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md) · [Cursor handoff (historical progress)](../../handoff/DUAL_CORE_CURSOR_HANDOFF.md) · [Archived alignment quick ref](../../handoff/archive/DUAL_CORE_ALIGNMENT.md)

---

## Terminology

| Term | Layer | Today |
|------|--------|--------|
| **Single-kernel dual-mode build** | Compile-time | **Yes** — PluginHost vs `monolith` feature ([RFC_OCLIVE_MONOLITH_MODE.md](RFC_OCLIVE_MONOLITH_MODE.md)) |
| **Dual-core dual-mode (this RFC)** | **Runtime** | **Opt-in Beta** — Stable + Experimental pipelines (default off) |

Do not conflate **build modes** with **runtime cores**.

---

## Summary

- **Stable core**: Only six slot `type`s (`memory` … `agent`); the authoritative executable path remains the host’s fixed co-present pre / middle / main-LLM / post lifecycle. `pipeline.stable` is not a second executable Stable pipeline.
- **Experimental core**: Arbitrary `type`s (e.g. `intent_recognition`); order from `pipeline.experimental` + **`depends_on` DAG** (validated at load).
- **One blueprint**: `slot_registry` is the **master table** (not split per core); `zone` is a string or **array** — an instance **may belong to both** stable and experimental.
- **DualPipelineRunner**: Experimental first with a bounded snapshot of narrative hint, current emotion, and presence scene. On failure it restores the previously present cache/SQLite-backed values and calls the host’s fixed Stable `co_present`; runtime does not execute `pipeline.stable` as a second pipeline. Prior `NULL` emotion/scene is not currently cleared (`K-DUAL-ROLLBACK-02`).
- **Shared backend pool**: No core-specific backends; register traits in `slot_registry`, both cores may use the same instance.
- **Default off**: No `--dual-core` ⇒ zero behavior change.
- **Monolith**: `--monolith` without dual-core = zero dual-pipeline overhead (shipping the single-pipeline runtime); `--monolith --dual-core` = welded pipelines + runner (dev high-perf lab).

**Q15–Q20 (decided)**: `runtime_config.dual_core.enabled`; exact v2/v3/v4 dispatch; P1 registry-key-only; migration tool deferred; `pipeline.stable` never executes and Stable always uses `co_present`; P4 supports the current seven `PluginHost` types (six slots plus the `complex_emotion` facility) only.

**Progress**: P1–P5 are merged, including validation, `DualPipelineRunner`, host gating, the CLI scaffold, OOCP coverage, and the Monolith template. The feature remains a **frozen v3 Beta, default off**. See [DUAL_CORE_CURSOR_HANDOFF.md](../../handoff/DUAL_CORE_CURSOR_HANDOFF.md) · [ROLE_PACK_BOUNDARY.md](../../handoff/ROLE_PACK_BOUNDARY.md).

Current reference-host blueprint delivery uses **Stable v4** for new packs in that format family, keeps v2 compatible, and reserves v3 for the opt-in dual-core Beta. This does not define the kernel-minimal role contract.
