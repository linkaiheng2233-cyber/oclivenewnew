# Post-process chain — English summary

[中文](../../creator-docs/rfc/RFC_OCLIVE_POST_PROCESS_CHAIN.md)

Full RFC (Chinese SSOT): [RFC_OCLIVE_POST_PROCESS_CHAIN.md](../../creator-docs/rfc/RFC_OCLIVE_POST_PROCESS_CHAIN.md).

**Status:** **Partially delivered** — the role-pack `reply_post_processor` hook and distro `standard` / `minimal` policy are implemented; an arbitrary multi-step chain remains Draft.

## Problem

After the LLM produces **`reply`** and before the user sees it, a role pack may select one `ReplyPostProcessor` (`builtin` / `remote` / `directory`), while the distro maps `post_process.chain` to the current `standard` / `minimal` policy. Formatting, safety filters, TTS segmentation, overlays, and other arbitrary ordered steps still lack a common step schema.

This RFC governs that future general chain. The delivered single RPP hook is not evidence that arbitrary step composition already exists.

## Terminology (SSOT)

| Name | English | Meaning |
|------|---------|---------|
| **后处理链** | **post-process chain** | Ordered steps between LLM output and user-visible reply |
| **内置后处理** | **built-in post-process** | Non-disableable generation, semantic/state, persistence, and response assembly logic in `turn_pipeline/post.rs`, `post/post_llm.rs`, and `persistence.rs` |
| **发行版后处理 profile** | **distro post-process profile** | `distro.oclive.toml` → `[post_process].chain`; currently `standard` / `minimal` |

## What it is **not**

- **Not** blueprint `pipeline.ocblueprint` `steps[]` DSL (deprecated as main scheduler)
- **Not** `dual_pipeline` experimental orchestration ([RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md](./RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md))
- **Not** six-slot `plugin_backends` / `slot_registry` backend replacement

## Layered boundary (draft)

| Layer | Responsibility | Config |
|-------|----------------|--------|
| `turn_pipeline/post.rs` + `post/post_llm.rs` + `persistence.rs` | Main LLM call; protocol stripping and semantic/state consumers; display/transcript persistence and DTO assembly | Code |
| **`reply_post_process` hook** (delivered) | One resolved `ReplyPostProcessor`: reply + context → display reply; failure falls back to unmodified text | Role-pack `reply_post_processor` + HostProfile policy |
| **General multi-step chain** (not implemented) | Arbitrary step schema, ordering, and per-step degradation | Future RFC / read-only config |
| Facility modules | Participate at their own explicit anchors; complex emotion spans pre / prompt / post but is not a generic text-chain step | Blueprint / code |
| Experimental core | Pre-fallback experimental steps | `pipeline.experimental` |

Current ordinary Stable flow: `LLM reply` → strip protocol / run authoritative state consumers → one optional RPP → display/transcript persistence → `SendMessageResponse.reply`. A future general chain would live inside that display transformation boundary. Agent shortcuts use `minimal_response` and skip this path.

## Non-goals before the general chain is delivered

- Do not describe the delivered single RPP as an arbitrary chain
- No new six-slot type for post-processing
- No blueprint v3 `runtime_config` chain definition
- No change to `SendMessageResponse` shape

## Future PR prerequisites

1. Align schema with [DISTRO_CAPABILITY_PROFILE.md](../../creator-docs/kernel/DISTRO_CAPABILITY_PROFILE.md) §3 `[post_process]`
2. Define merge rules per step/config family; do not assume one universal distro > role pack > session hierarchy
3. OOCP: at least one “chain step fails → fall back to built-in” black-box case
4. Breaking process: [BREAKING_CHANGE_PROCESS.md](../../handoff/BREAKING_CHANGE_PROCESS.md)

## Code anchor (read-only)

- Built-in today: `kernel/crates/oclive_kernel_host/src/domain/chat_engine/turn_pipeline/post.rs`, `post/post_llm.rs`, and `persistence.rs`
- Delivered reply post-processor (separate side-channel): [RFC_USER_IDENTITY_AND_REPLY_POST_PROCESSOR_SUMMARY.md](./RFC_USER_IDENTITY_AND_REPLY_POST_PROCESSOR_SUMMARY.md)
