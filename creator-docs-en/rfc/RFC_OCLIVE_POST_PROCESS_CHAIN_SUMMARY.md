# Post-process chain — English summary

[中文](../../creator-docs/rfc/RFC_OCLIVE_POST_PROCESS_CHAIN.md)

Full RFC (Chinese SSOT): [RFC_OCLIVE_POST_PROCESS_CHAIN.md](../../creator-docs/rfc/RFC_OCLIVE_POST_PROCESS_CHAIN.md).

**Status:** **Draft / not in v0.2 scope** — naming and boundaries only; no runtime chain in v0.2.x.

## Problem

After the LLM produces **`reply`** and before the user sees it, distros may need pluggable **post-processing** (formatting, safety filter, TTS segmentation, overlay, etc.). Today much logic is hard-coded in `turn_pipeline/post.rs` and facility modules; there is no unified extension point.

## Terminology (SSOT)

| Name | English | Meaning |
|------|---------|---------|
| **后处理链** | **post-process chain** | Ordered steps between LLM output and user-visible reply |
| **内置后处理** | **built-in post-process** | Non-disableable core logic in host `turn_pipeline/post.rs` |
| **发行版后处理 profile** | **distro post-process profile** | `distro.oclive.toml` → `[post_process].chain` (P4 draft field) |

## What it is **not**

- **Not** blueprint `pipeline.ocblueprint` `steps[]` DSL (deprecated as main scheduler)
- **Not** `dual_pipeline` experimental orchestration ([RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md](./RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md))
- **Not** six-slot `plugin_backends` / `slot_registry` backend replacement

## Layered boundary (draft)

| Layer | Responsibility | Config |
|-------|----------------|--------|
| `turn_pipeline/post.rs` | Session write, DTO fill, memory/favor side effects | Code |
| **Chain extension** (not implemented) | Pure steps: `reply` + context → `reply` | `distro.oclive.toml` / future blueprint read-only |
| Facility modules | In-turn narrative hints, pre-LLM / prompt stages | Blueprint / code |
| Experimental core | Pre-fallback experimental steps | `pipeline.experimental` |

Flow: `LLM reply` → built-in post → optional chain → `SendMessageResponse.reply`.

## Non-goals (v0.2)

- No new `post_process` trait or plugin slot
- No blueprint v3 `runtime_config` chain definition
- No change to `SendMessageResponse` shape

## Future PR prerequisites

1. Align schema with [DISTRO_CAPABILITY_PROFILE.md](../../creator-docs/kernel/DISTRO_CAPABILITY_PROFILE.md) §3 `[post_process]`
2. Merge priority: distro > role pack > session
3. OOCP: at least one “chain step fails → fall back to built-in” black-box case
4. Breaking process: [BREAKING_CHANGE_PROCESS.md](../../handoff/BREAKING_CHANGE_PROCESS.md)

## Code anchor (read-only)

- Built-in today: `kernel/crates/oclive_kernel_host/src/domain/chat_engine/turn_pipeline/post.rs`
- Delivered reply post-processor (separate side-channel): [RFC_USER_IDENTITY_AND_REPLY_POST_PROCESSOR_SUMMARY.md](./RFC_USER_IDENTITY_AND_REPLY_POST_PROCESSOR_SUMMARY.md)
