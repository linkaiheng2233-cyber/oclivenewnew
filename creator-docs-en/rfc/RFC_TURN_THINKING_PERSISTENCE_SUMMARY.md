# Turn Thinking persistence & routing — English summary

[中文](../../creator-docs/rfc/RFC_TURN_THINKING_PERSISTENCE.md)

Full RFC (Chinese SSOT): [RFC_TURN_THINKING_PERSISTENCE.md](../../creator-docs/rfc/RFC_TURN_THINKING_PERSISTENCE.md).

## What it is

**Turn Thinking** is a **co-present orchestration policy** (not a seventh slot). Each turn picks **Auto → Fast or Deep** before the main LLM chain. It affects **latency** (skip extra LLM calls on Fast) and **what gets consolidated** into long-term memory, favor, and mutable personality.

Code: `kernel/crates/oclive_kernel_host/src/domain/turn_thinking.rs` · router in `turn_pipeline/co_present.rs`.

## Wave E — persistence split (host)

`distro.oclive.toml` → `[turn_thinking] fast_persistence`:

| Value | Fast casual chat |
|-------|------------------|
| `legacy` (default) | Full persistence (same as pre–Wave E) |
| `strong_only` | Skip LTM / favor / profile evolution; **Quarrel / Apology / Confession / Praise** still persist |

**Chat turns** (`chat_storage`) are **always** written every round.

Host profile SSOT: [DISTRO_CAPABILITY_PROFILE.md §3.2.1](../../creator-docs/kernel/DISTRO_CAPABILITY_PROFILE.md) (Chinese).

## Wave F — role-pack routing (pack)

`config.json` → `turn_thinking`:

- **`deep_when.or` / `deep_when.and`**: rule signals merged with host defaults (`or = host ++ pack`; pack `and` groups require all signals).
- **`latch`**: e.g. enter on `Quarrel`, exit on `Apology` — stay Deep until reconciliation (`role_runtime.deep_latch_active`).
- **`ephemeral_archive`**: rule-written **situation summary** (`ephemeral_personality`, TTL turns) injected as `【局面摘要】` on Fast **and** Deep; separate from `mutable_personality`.

Migration: `035_turn_thinking_runtime.sql`.

Pack schema: [ROLE_PACK_SPEC.md §9.11](../../creator-docs/role-pack/ROLE_PACK_SPEC.md#911-turn_thinkingwave-f) (Chinese).

## Product boundaries

- **No** player UI toggle for Fast/Deep.
- **No** seventh slot / `slot_registry` changes.
- Pack editor UI: **PE-TURN-01** (open; not blocking MVP).
