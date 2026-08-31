# RFC: Runtime Event Stream — English summary

[中文](../../creator-docs/rfc/RFC_RUNTIME_EVENT_STREAM.md)

**Scope:** architecture and authority summary for the future Runtime Event Stream. The Chinese RFC is the contract SSOT. Current Event Ring wire and proactive authorization remain defined by [EVENT_RING](../plugin-and-architecture/EVENT_RING.md).

**Last updated:** 2026-09-01.

**Status:** **Draft v0.1 · boundaries agreed · wire and implementation not frozen · not implemented**.

---

## Core decision

The “character runtime river” is provisionally named **Runtime Event Stream**. It is a future durable timeline for cross-turn, cross-channel, restartable event delivery and consumer progress. It is not a rename of Event Ring and not a loop that polls modules in turn.

The following constraints are fixed:

- User turns keep the single `process_message` entry. Non-user turns still require Event admission, a one-shot `ProactiveTurnPermit`, and `process_proactive_turn`.
- Event Ring keeps source binding, envelope identity, registry influence, and bounded routing for one dispatch.
- Session, Stream, consumers, plugins, and LLMs cannot commit authoritative character state. Rust orchestration remains the single application/commit boundary.
- A `Decision` is not a committed state. `State` may only describe a successful commit.
- Proposal influence remains in the existing trusted `EventModuleRegistryPolicy`. A future Stream consumer registry manages subscriptions, read scope, cursors, and backpressure only.
- Model tiers may change observation, query, and proposal budgets, never source or commit authority.
- This RFC does not claim that a durable Stream, replay, multi-I/O scheduling, or a productized proactive bot exists today.

---

## Five boundaries

| Layer | Owns | Must not own |
|-------|------|--------------|
| Session | Runtime identity, isolation, lifecycle, and stream partition binding | Proposal admission, persona mutation, or state commit |
| Runtime Event Stream | Durable append/read, partition position, checkpoints, retention, and recovery | Influence weights, reply decisions, turns, or state writes |
| Event Ring | Trusted envelopes, source binding, registry policy, and bounded dispatch | Persistence or final state authority |
| Event decision module | Admit, reject, or derive proposals in its declared event family | Source/weight forgery, DB writes, or unchecked host capabilities |
| Rust orchestration | Permission recheck, turn invocation, CAS/transactional state application, and post-commit facts | Delegating state authority to Session/Stream/LLM |

The Event decision module is not the legacy backend `event` slot. The legacy slot only estimates dialogue `event.impact`.

---

## Intended flow

```text
external input / kernel result
        ↓
source-bound ingress → Event Ring dispatch → Ring-issued EventEnvelope
                                             ↓ append
                                  Runtime Event Stream
                                             ↓ read
                                  declared consumer
                                             ↓ new EventDraft + causation
                                      Event Ring decision
                                             ↓
                         existing process_message / process_proactive_turn
                                             ↓
                           Rust commit → host-bound emitter → Ring
                                           └→ State / Output fact → Stream
```

Rules:

1. External producers cannot append arbitrary `EventEnvelope` values.
2. Reading an old event does not grant authority. A consumer creates a new draft and references the old event as its cause.
3. Event handlers cannot recursively start turns; orchestration consumes typed results only after dispatch returns.
4. Trace-only recording must not become a required dependency of the low-latency user path.
5. A future Stream-first adapter may route an eligible user input only to the same `process_message`; it cannot create another turn engine.

---

## Event taxonomy

| Type | Meaning | Authority limit |
|------|---------|-----------------|
| Fact | A source-attributed statement about something known or completed | Trusted source does not mean absolute truth; conflicts retain provenance |
| Observation | Raw or normalized input from a channel, sensor, game, or environment | May trigger query/proposal, never automatic state mutation or reply |
| Proposal | Suggested context, action, or state change | Must pass an Event decision; cannot self-admit or self-weight |
| Decision | Admission/rejection/derivation by the appropriate decision module | May still fail revision, permission, or transaction checks |
| State | Fact emitted after Rust successfully commits state | Never acts as the command that causes the commit |
| Output | Generated, attempted, delivered, or failed output result | Generation success must not be reported as delivery success |

Operational checkpoint, lease, and backpressure records are control-plane data and do not enter character context by default.

The semantic type cannot be self-reported in ordinary payload or metadata. A host-owned event-kind registry or typed outer field must assign it and validate it together with each module's allowed emission scope.

---

## Identity and durable position

`EventEnvelope` remains the trusted event identity and causation contract. A future persistence layer should wrap it with Session partition and durable storage position rather than overload current fields.

```text
RuntimeEventRecord
├─ session_partition
├─ stream_position
├─ ingested_at
└─ envelope: EventEnvelope

ConsumerCheckpoint
├─ consumer_id
├─ session_partition
├─ next_position
└─ lease_epoch / revision
```

Current `EventEnvelope.sequence` is allocated by one in-process `AppState` and is **not** a durable cursor across restart. `occurred_at` is source time and may be skewed; recovery uses store-owned position. `stream_key` is not automatically a Session identity or access-control credential.

Current Ring handlers may replace payload or merge metadata while the primary event keeps the original emitter's `source`. That is acceptable for a bounded compatibility dispatch but insufficient durable provenance. A semantic transformation intended for Production Stream must emit a child event with the transforming module's own source and causation. In-place mutation remains dispatch-local; an append-only Stream corrects or retracts records through new events rather than overwriting old ones.

---

## Delivery and recovery baseline

- The first implementation uses at-least-once delivery; it does not claim end-to-end exactly-once.
- Consumers advance checkpoints only after a side effect succeeds or is durably staged.
- Duplicate delivery must be safe through `event_id`, a source-bound idempotency key, or a deterministic cause/action key.
- Ordering is guaranteed only within one Session partition; wall-clock last-write-wins is forbidden for concurrent sources.
- Rust applies expected revision/CAS or transaction checks. Stale decisions are rejected, recalculated, or marked superseded.
- Bounded in-flight work, retry backoff, lag diagnostics, and explicit sampling/coalescing policy are required.
- `Decision` and `State` must not be silently dropped. Production state commit plus event publication requires a transactional outbox/reconcile design.

---

## Memory, models, and proactive turns

The Stream may widen what memory modules can observe, but the whole timeline never enters Prompt directly:

```text
Stream Observation / State
  → memory query or recall/archive Proposal
  → Event decision
  → Rust fetches admitted IDs/safe evidence
  → bounded Prompt context
  → LLM
```

Memory bodies remain in the memory store. Models see only policy-approved, bounded context. Larger models may query more; smaller models receive compiled context; neither can emit authoritative `State`.

An Observation or high-influence Proposal does not automatically make the character speak. Productized proactive behavior still requires deduplication, TTL, cooldown, rate budget, user-input preemption, one-shot Permit, cancellation, an output port, and delivery results. Runtime Event Stream does not broaden the current non-user turn's persistence side effects.

---

## Trace, replay, privacy, and degradation

- Trace-only is independently disableable and behavior-neutral.
- Replay defaults to an isolated Session/dry-run with production state and output ports disabled.
- Recovery uses checkpoints and idempotency, not replaying all historical Output events.
- Credentials, full system prompts, raw long-term-memory bodies, and unconsented media must not enter generic payloads.
- Trace storage failure disables Trace without changing the user reply. A production Stream dependency enters explicit degraded/blocked state and must not bypass authority boundaries.
- Consumer failure does not advance its checkpoint or block unrelated consumers.

---

## Staged admission

1. Freeze taxonomy, Session mapping, outer record, checkpoint, and failure/recovery tests.
2. Add a disableable trace-only recorder; prove disabled parity, restart, redaction, and retention.
3. Add cursor, at-least-once, idempotency, backpressure, and an effect-free test consumer.
4. Integrate one low-risk real consumer through Draft → Ring → Decision → Rust application.
5. Add one real non-user adapter, scheduler state machine, and output port with TTL/cooldown/preemption/cancellation/recovery tests.

Real multi-I/O consumers and relevant reference projects must be studied before production implementation. They inform consumer and recovery semantics but cannot weaken the authority boundaries above.

Open implementation choices remain tracked by Chinese [`K-EVENT-STREAM-01`](../../handoff/TECHNICAL_DEBT_INVENTORY.md). The debt stays OPEN until code, recovery tests, and repository evidence exist.
