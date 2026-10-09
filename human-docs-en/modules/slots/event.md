# Slot pack · `event` (EN summary)

> Full checklist (ZH): [`human-docs/modules/slots/event.md`](../../../human-docs/modules/slots/event.md)
> Scope: developer entry points and checklist. Public entry points: [MODULE_MAP §3.1.1](../../../handoff/MODULE_MAP_AND_HANDOFF.md#311-base-实现者接入点清单一页); reference Host definitions: [§6](../../../handoff/MODULE_MAP_AND_HANDOFF.md#6-第-3-模块--event).
> Updated: 2026-10-09.

## Choose your entry point

Start below for basic on-demand analysis. Use **Reference Host** for `EventEstimator`, impact estimation or the legacy bridge; its state application and policies are not automatically prerequisites for every Base implementation.

### EventBase: check analysis cost and result scope

1. Find `EventBase::analyze` in the [public contract](../../../kernel/crates/oclive_kernel_contracts/src/slot_base.rs). The existing [`LlmEventAnalyzer`](../../../kernel/crates/oclive_kernel_runtime/src/domain/base_event.rs) borrows a caller-selected `LlmBase` and calls it once per analysis. Its private input/reply agreement is not a general Base protocol; choose a compatible generator.
2. Follow independent Event delegation in [`MinimalRoleBaseConsumer`](../../../kernel/crates/oclive_kernel_runtime/src/domain/minimal_role_consumer.rs). The reference Host exposes [`OcliveKernel::event_analysis_base()`](../../../kernel/crates/oclive_kernel_host/src/role_kernel.rs), wired in [`minimal_event.rs`](../../../kernel/crates/oclive_kernel_host/src/domain/chat_engine/minimal_event.rs). Constructing the view starts no analysis; ordinary chat does not automatically add analysis because the view exists.
3. Read [`base_only_fixture.rs`](../../../kernel/crates/oclive_kernel_contracts/tests/base_only_fixture.rs) and [`minimal_role_host`](../../../kernel/crates/oclive_kernel_runtime/examples/minimal_role_host.rs) for existing implementation and consumption examples. Select and schedule for a concrete need, not to run all six slots.

**Base checklist**:

- [ ] State the implementation's agreement and call cost; distinguish normal absence of analysis text from failure, keeping complete generator errors and format failures.
- [ ] The Host chooses result use. Base analysis does not publish Ring events, apply role state or grant proactive-turn permits.

Definitions remain in the [entry-point checklist](../../../handoff/MODULE_MAP_AND_HANDOFF.md#311-base-实现者接入点清单一页). The reference analyzer does not establish real model format compliance or analysis quality.

## Reference Host

**You plug in**: blueprint `slot_registry.type: event` → folded `PluginBackends.event` (legacy v1: `settings.json.plugin_backends.event`) · trait `EventEstimator` · hook `co_present/run_middle.rs` `EventEstimate` → `publish_legacy_event_impact` → `PersonalityEngine::evolve_by_event`.

This slot estimates dialogue event impact only. It does not own Event Ring envelopes, decision modules, or proactive authorization. See [EVENT_RING](../../../creator-docs-en/plugin-and-architecture/EVENT_RING.md).

**Two paths**: Rule table `EventDetector` vs LLM `estimate_event_impact`. LLM switch is **`HostProfile.event_impact_llm`** — not a slot key. See [DISTRO_CAPABILITY_PROFILE](../../../creator-docs/kernel/DISTRO_CAPABILITY_PROFILE.md).

**Do**: Rule / LLM builtin paths · `remote` / `directory` backends · distro `distro.oclive.toml` for defaults.

**Don't**: Call this slot the whole Event Ring · issue proactive permits · register Turn Thinking as a seventh slot · force the LLM event path on Fast turns.

**Read next**: [MODULE_MAP §12 `event_impact_llm`](../../../handoff/MODULE_MAP_AND_HANDOFF.md) · [turn-thinking](../orchestration/turn-thinking.md) · [RFC_TURN_THINKING](../../../creator-docs/rfc/RFC_TURN_THINKING_PERSISTENCE.md).
