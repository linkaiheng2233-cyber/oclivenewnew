# Event Ring and proactive-turn contract

[中文](../../creator-docs/plugin-and-architecture/EVENT_RING.md)

**SSOT scope:** Event envelopes, registry policy, authority boundaries, integrated event kinds, and proactive-turn authorization. See [MODULE_MAP](../../handoff/MODULE_MAP_AND_HANDOFF.md) for module classification and [TECHNICAL_DEBT_INVENTORY](../../handoff/TECHNICAL_DEBT_INVENTORY.md) for implementation status and open gaps.

**Last updated:** 2026-09-02.

**Audience:** kernel integrators, Event module authors, directory-plugin authors, and maintainers.

---

## 1. Position in the architecture

Event Ring is a bounded, in-process, non-persistent event perimeter owned by each `AppState`. Registered modules may propose, enrich, and transform events. The ring assigns trusted identity, source, base influence, ordering, correlation, and causation.

“Perimeter” means the routing boundary of one dispatch, and “authority” covers envelope identity and registry policy only. Event Ring is not a scheduler that continuously polls modules, does not determine whether an external claim is absolutely true, and does not own proposal admission or character-state commits.

Event Ring is not:

- a seventh backend slot;
- a pipeline that invokes every module in a fixed round-robin order;
- chat history or a database event bus;
- a second reply engine that bypasses `process_message`;
- an unrestricted way for plugins to trigger proactive replies.

The legacy `event` slot remains backend module 3 and estimates dialogue `event.impact` only. A compatibility bridge submits that estimate to Event Ring. Module classification is maintained only in [MODULE_MAP §6](../../handoff/MODULE_MAP_AND_HANDOFF.md#6-第-3-模块--event).

### Boundary with a future Runtime Event Stream

The current Event Ring governs trusted event circulation within one dispatch. It is **not** the cross-turn, cross-channel, restartable Session timeline discussed as a continuous “character runtime river.” That future concept is provisionally named **Runtime Event Stream**. It may carry continuous facts and derived events from chat platforms, livestreams, games, sensors, memory, and agents. Stages A.1/A.2.1 freeze pure types and local persistence design; A.2.2.1 reviews OneBot v11 acknowledgement and uncertain-delivery semantics. There is still no Production database, read interface, consumer loop, real QQ adapter, or recovery implementation.

Only an off-by-default **B0 trace-only shadow** exists today. After a successful dispatch enters Ring history, redacted fact headers can be sent non-blockingly to an independent SQLite database. Failures affect diagnostics only. The shadow has no read, consumer, replay, Prompt, or proactive-trigger authority, so it is not Runtime Event Stream and does not alter the Ring authority defined here.

Any future design must preserve these boundaries:

- Runtime Event Stream does not replace Event Ring. The stream may own a durable timeline and consumer progress; the Ring keeps authority over trusted envelopes, registration policy, and bounded routing.
- Runtime Event Stream does not replace the Stable turn pipeline or create a second `process_message`.
- A Session is a role-runtime instance and isolation boundary. Dispatch responsibility does not grant admission, persona mutation, or state-commit authority.
- Consumers may observe, query, propose, and derive events, while state changes still pass through Event decisions and Rust orchestration.
- A large model may receive broader observation, query, and proposal rights, but cannot forge facts or commit authoritative state. Small models continue to consume bounded context selected by modules and compiled by the Prompt layer.
- B0 durable trace proves behavior-neutral append only. Its S0/S1 synthetic evidence is now closed and includes a fixed ten-minute developer soak, but that is not a production-duration or target-hardware soak. Replay, consumer cursors, Production backpressure/idempotency, retention, and the complete privacy contract remain future Stream/Trace work. Neither B0 nor bounded Ring diagnostics may be presented as those capabilities.

See the [`RFC_RUNTIME_EVENT_STREAM` summary](../rfc/RFC_RUNTIME_EVENT_STREAM.md). Design and implementation status remains tracked as `K-EVENT-STREAM-01` in the Chinese [TECHNICAL_DEBT_INVENTORY](../../handoff/TECHNICAL_DEBT_INVENTORY.md). Closing B0 S1 adds no read or behavior authority; until the stage A contract plus consumer and recovery contracts exist, the “river” is not a shipped capability.

---

## 2. Four authority levels

| Level | Authority it owns | Authority it does not own |
|-------|--------------------|---------------------------|
| Rust turn orchestration | Chooses when to dispatch and consume events, run Prompt/LLM, and commit state | Does not implement each capability slot's retrieval or generation algorithm |
| Event Ring | Assigns identity, source, registry weight, order, correlation, and causation; performs bounded routing | Does not decide what the character says and does not treat weight as execution order |
| Event decision module | Admits, rejects, or derives events for proposals it subscribes to | Cannot forge sources, raise its registry weight, or issue a host Permit |
| Capability module/slot | Produces domain evidence or proposals, such as event-impact estimates or memory candidates | Cannot declare its proposal admitted or bypass turn orchestration to commit final state |

An “authoritative event” in this document means that source, identity, causation, and the applicable decision passed their current boundary checks. It **does not mean that character state has already been committed**. A state change exists only after Rust orchestration applies it successfully; any future `State` event must describe a completed commit rather than act as the command that causes one.

Influence uses fixed-point basis points: `10_000 == 1.0`. It is the proposal's base influence for downstream decision modules, not module priority, invocation frequency, or a final admission result. Execution order is `(priority, module_id)`.

---

## 3. Data and registration contract

```text
EventModuleDeclaration              EventModuleRegistryPolicy
(module declaration)                (assigned by trusted host)
├─ module_id                         ├─ influence_weight_bps
├─ subscriptions                    └─ failure_mode
├─ emissions                            ├─ fail_fast
└─ priority                             └─ isolate
           ↓
module submits EventDraft
           ↓
source-bound EventEmitter → Event Ring → EventEnvelope
```

A module submits only `EventDraft { kind, payload, metadata }`. Event Ring supplies and keeps authority over these `EventEnvelope` fields:

- `event_id`, `source`, and `source_weight_bps`;
- `stream_key`, `correlation_id`, and `causation_id`;
- `sequence`, `depth`, and `occurred_at`.

A registered module may replace the current payload, merge metadata, or emit child events within its declaration. It cannot alter authoritative identity fields. Built-in modules default to `fail_fast`. Untrusted adapters should use `isolate`; their failing or unauthorized output is rejected atomically and the module is quarantined.

Public Rust contracts:

- data types: `oclive_kernel_types::event_ring`;
- ports: `oclive_kernel_contracts::event_ring`;
- host implementation: `oclive_kernel_host::domain::event_ring`.

---

## 4. Integrated event chains

### 4.1 Legacy dialogue-event compatibility bridge

```text
legacy event slot estimate
  → kernel.chat.event_impact.estimated
  → Event Ring modules may enrich or transform it within contract
  → Rust orchestration consumes the final EventImpactEstimate
  → personality, favor, and relationship preview
```

With no registered handler, the bridge preserves every field of the estimate.

### 4.2 Memory recollection

```text
memory retrieval candidates
  → kernel.memory.recall.candidate
  → builtin.event_decision
  → kernel.memory.recollection.activated (optional)
  → one-turn recollection Prompt section
```

The proposal carries memory IDs, confidence, and relevance, not memory text. Only admitted memories enter the dedicated reply context. Their current TTL is one turn. Text is fetched by ID outside the ring and passes through safe evidence extraction.

### 4.3 Directory plugins

Directory plugins join through manifest `eventRing` and JSON-RPC `event_ring.handle`. Manifest values are untrusted suggestions. The host owns the subscription allowlist, plugin emission namespace, weight cap, fixed priority, timeout, and `isolate` policy. The detailed wire is maintained only in the [Chinese directory-plugin contract](../../creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md#44-内核-event-ring-适配与-bridgeevents-无关).

Directory plugins currently cannot read or write proactive-turn kernel events, obtain a proactive emitter, or receive a `ProactiveTurnPermit`.

---

## 5. Authorized proactive turns

```text
trusted input adapter
  → kernel.proactive.turn.proposed
  → builtin.proactive_turn_decision
  → kernel.proactive.turn.authorized
  → one-shot ProactiveTurnPermit
  → process_proactive_turn
  → origin-aware co-present reply
```

Security boundaries:

- A proposal cannot declare its own source or base weight; registration binds both into its `EventEmitter`.
- `origin` may be `sensor` or `system`, never an impersonated `user`.
- Only an authoritative authorization event with valid source, causation, and score range can yield a Permit.
- A Permit is not publicly constructible, cloneable, or deserializable, and is consumed once.
- The host invokes the Turn Engine only after ring dispatch returns; an event handler must not recursively start a turn.
- External observations use `TurnInput::ExternalObservation`; `SendMessageRequest.user_message` stays empty.
- A non-user turn currently skips user emotion and the legacy event path. It does not write user chat, STM/LTM, relationship, personality, continuity, virtual time, or the legacy `events` table.
- The remote-life Prompt is not origin-aware yet, so non-user turns are temporarily forced through the co-present Prompt.

This executor is infrastructure, not a productized proactive bot. Deduplication, TTL, cooldown, rate limits, user-input preemption, proactive output ports, and third-party root-event ingress remain tracked only as [`K-PROACTIVE-01`](../../handoff/TECHNICAL_DEBT_INVENTORY.md).

---

## 6. Diagnostics and privacy

`EventRingDiagnostics` exposes a deterministic registry snapshot, module runtime state, bounded-history occupancy, sequence state, and routing/causation summaries. It excludes by default:

- payload content;
- metadata values;
- `stream_key`.

It is a read-only, eventually consistent kernel diagnostic, not an event persistence API.

---

## 7. Change checklist

- [ ] New event kinds use canonical dotted names and declare subscription/emission scope.
- [ ] Module declarations remain separate from trusted registry policy.
- [ ] Proposal, admission, and execution use distinct events or type boundaries.
- [ ] Influence weight is not treated as priority or final authority.
- [ ] External observations never enter the user-message field or gain user identity semantics.
- [ ] A new proactive entrypoint cannot bypass Permit, persistence boundaries, or the future scheduling state machine.
- [ ] Directory-plugin coverage includes normal, unauthorized, timeout, and isolation paths.
