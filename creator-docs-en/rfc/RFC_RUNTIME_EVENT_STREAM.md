# RFC: Runtime Event Stream — English summary

[中文](../../creator-docs/rfc/RFC_RUNTIME_EVENT_STREAM.md)

**Scope:** architecture and authority summary for the future Runtime Event Stream. The Chinese RFC is the contract SSOT. Current Event Ring wire and proactive authorization remain defined by [EVENT_RING](../plugin-and-architecture/EVENT_RING.md).

**Last updated:** 2026-09-01.

**Status:** **Draft v0.6 · boundaries agreed · B0 trace-only plus S0, S1.1, S1.2, and S1.3 synthetic samples implemented · Production Stream not implemented**.

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
- The optional B0 shadow records redacted headers from successful Ring dispatches only. Its S0/S1.1/S1.2/S1.3 commands create synthetic, Git-ignored structural, fault, bounded-load, and recorder duplicate-accounting evidence only. None of these capabilities claims that a durable Stream, consumer cursor, replay, multi-I/O scheduling, or a productized proactive bot exists today.

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

### Current B0 trace-only slice

Commit `d2320596` adds one independently rollbackable observer, not a Production Stream:

- trace is off by default and creates no file; opt-in uses a separate SQLite database and refuses the main kernel database path;
- Event Ring calls the observer only after an entire dispatch succeeds and enters bounded Ring history; failed dispatches are absent;
- the hot path uses bounded `try_send`; open, schema, queue, and write failures update redacted diagnostics/drop counters and never change `EventDispatchResult`;
- append-only records have a store-owned `position` plus event identity, kind, source, registry weight, correlation, causation, Ring sequence/depth, and occurrence/ingestion times;
- payload, metadata, and `stream_key` are not stored; no read API, consumer, checkpoint, replay, retention executor, Prompt input, or proactive trigger exists;
- `runtime_event_trace_diagnostics()` exposes health and counters, not the database path or event content.

The durable position proves restart-safe trace append only. It is not a Session consumer cursor, and Ring `sequence` remains non-durable ordering evidence.

### B0 synthetic sample collection (implemented, evidence-only)

Commit `6bb3e1f7` adds the explicit developer command `npm run event:trace-shadow-samples`. It collects reproducible structural evidence without reading real user data:

- a versioned contract fixes five scenarios and seven records: a root event, a derived causation chain, admitted and rejected proactive proposals, and a kernel restart against the same trace database;
- only synthetic Event Ring modules and identifiers are used. The sampler does not request a model reply and does not pass an admitted Permit into a proactive turn;
- normalized JSON contains only scenario-local indexes, kind, source, registry weight, durable position, Ring sequence, causation index, and depth. Validation recursively rejects payload, metadata, `stream_key`, event IDs, timestamps, observations, and user-message fields;
- SQLite, JSON, and Markdown artifacts are written only under the Git-ignored `target/oclive-event/trace-shadow-samples/` directory. They record source commit, worktree state, and contract SHA-256; the raw SQLite artifact is not committed by default;
- the current sample observes durable positions 1–7 across two kernel starts while the in-process Ring sequence resets to 1. This is B0 structural evidence, not proof of Session cursors, consumer recovery, or a Production Stream.

Further collection follows this admission ladder:

| Level | Scope | Admission boundary | Status |
|-------|-------|--------------------|--------|
| **S0 · fixed synthetic structure** | Versioned repository contract for success, derivation, decision, and restart structure | Explicit local command; ignored output only; no behavior or training input | **Implemented** |
| **S1 · extended synthetic failure matrix** | Reproducible queue-full, write-failure, duplicate, concurrency, and soak scenarios | Still no real users, role memories, or model bodies; freeze expectations and privacy fields first | **Partial: S1.1 queue-full/write-failure + S1.2 concurrency/short soak + S1.3 recorder duplicate-header idempotency/accounting** |
| **S2 · explicitly consented local aggregates** | Minimal counts, latency, drop rate, and event-kind distributions | Requires prior opt-in notice, consent, retention, deletion, redaction, budget, and export review; no body collection or automatic upload by default | Not authorized |

Commit `d03655b5` implements the S1.1 command `npm run event:trace-shadow-fault-samples` without changing production Trace, Ring, or public APIs:

- `queue_saturation_fail_open` holds a writer lock on a temporary trace database and performs 640 contract-bound synthetic Ring dispatches. Admission freezes only the invariants that all Ring dispatches succeed, enqueue/drop accounting balances, `queue_full` is observed, all admitted work drains, and the main database remains healthy. Scheduler-dependent exact enqueue/drop counts are evidence, not product contract;
- `post_start_write_failure_fail_open` removes only a temporary trace table and performs one synthetic dispatch. Ring must succeed, diagnostics must report `write_failed`, no dispatch may be rejected by Trace, and the main database must remain healthy;
- JSON and Markdown are written only under Git-ignored `target/oclive-event/trace-shadow-fault-samples/`. No failed SQLite database is exported, and validation rejects content, identities, event IDs, correlation, runtime paths, and event timestamps;
- duplicate delivery, concurrency, and soak remained later S1 slices at S1.1. Any duplicate-event probe that requires an internal replay entry must first be reviewed as a possible source-binding bypass.

Commit `e17038a4` implements the S1.2 command `npm run event:trace-shadow-load-samples`, again without changing production Trace, Ring, or public APIs:

- `concurrent_burst_fail_open` uses eight concurrent workers with 64 source-bound synthetic dispatches each. The contract freezes only that all 512 Ring dispatches succeed, Trace enqueue/drop accounting balances, admitted work drains, Ring history remains bounded, and the only permitted result is no Trace error or `queue_full`;
- `bounded_short_soak_fail_open` uses four workers over 24 rounds, eight dispatches per worker per round, and a 50 ms inter-round pause for 768 bounded dispatches. The whole scenario has a 60-second ceiling and verifies complete rounds/workers, main-database health, and Trace drain;
- repeated local samples show that the burst's exact enqueue/drop split varies with scheduling while the short soak can drain fully. Elapsed time and exact ratios are Git-ignored machine evidence, not a performance SLA or product contract;
- JSON and Markdown are written only under `target/oclive-event/trace-shadow-load-samples/`. Shared validation rejects content, identities, event/correlation IDs, runtime paths, event timestamps, SQLite files, and any other unexpected output file;
- this slice adds no Replay, duplicate injection, consumer, checkpoint, or behavior-feedback entry. Consumer duplicate delivery and production-duration soak remain pending review and evidence.

Commit `20f44f47` implements the S1.3 command `npm run event:trace-shadow-duplicate-samples`. It compiles and runs only an internal B0 recorder test and changes no production Trace, Ring, or public API:

- the versioned contract contains only `duplicate_header_idempotency`. Under `#[cfg(test)]`, the test submits the same synthetic trace header twice to the private recorder without adding a production Replay or duplicate-injection entry;
- acceptance fixes two enqueued and processed internal record batches, one SQLite row, `duplicate_events = 1`, no drops, no Trace failure, and a worker that stops after explicit shutdown. This is not a production-path test of two real Ring dispatches;
- the Rust test prints normalized counts and booleans only. The collector recursively rejects bodies, metadata, `stream_key`, event/correlation IDs, paths, and event timestamps, writes JSON/Markdown only under `target/oclive-event/trace-shadow-duplicate-samples/`, and exports no SQLite file;
- this proves idempotent persistence and diagnostic accounting for the same event identity inside the B0 recorder only. It does **not** prove consumer duplicate delivery, at-least-once consumption, end-to-end idempotency, checkpoint recovery, or a Production Stream; those remain stage C or later work.

No sample level is an authoritative event store, behavior input, or training authorization. Real runtime collection must not be wired before S2 admission; content-level data would require a separate privacy and data-governance design rather than expanding this command.

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
- Current B0 persists only redacted headers from successful Ring dispatches. It has no Replay path and is neither an authoritative event store nor a behavior input.
- Replay defaults to an isolated Session/dry-run with production state and output ports disabled.
- Recovery uses checkpoints and idempotency, not replaying all historical Output events.
- Credentials, full system prompts, raw long-term-memory bodies, and unconsented media must not enter generic payloads.
- Trace storage failure disables Trace without changing the user reply. A production Stream dependency enters explicit degraded/blocked state and must not bypass authority boundaries.
- Consumer failure does not advance its checkpoint or block unrelated consumers.

---

## Staged admission

1. Freeze taxonomy, Session mapping, outer record, checkpoint, and failure/recovery tests.
2. Add a disableable trace-only recorder. B0 now proves disabled parity, restart-safe position, redaction, append-only storage, and fail-open degradation; S0 fixes five structural scenarios/seven records, S1.1 covers queue saturation and post-start write failure, S1.2 covers a concurrent burst plus bounded short soak, and S1.3 covers recorder duplicate-header idempotency/accounting. Retention and commit/output coverage remain open.
3. Add cursor, at-least-once, idempotency, backpressure, and an effect-free test consumer.
4. Integrate one low-risk real consumer through Draft → Ring → Decision → Rust application.
5. Add one real non-user adapter, scheduler state machine, and output port with TTL/cooldown/preemption/cancellation/recovery tests.

Real multi-I/O consumers and relevant reference projects must be studied before production implementation. They inform consumer and recovery semantics but cannot weaken the authority boundaries above.

Open implementation choices remain tracked by Chinese [`K-EVENT-STREAM-01`](../../handoff/TECHNICAL_DEBT_INVENTORY.md). The debt stays OPEN: B0 trace evidence does not satisfy Production Stream consumer, recovery, retention, and domain-loop acceptance.
