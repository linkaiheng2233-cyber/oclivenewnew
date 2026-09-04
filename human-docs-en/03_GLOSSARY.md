# 03 · Glossary

| Term | Meaning |
|------|---------|
| **mrid** | Manifest role id — pack folder / manifest role id |
| **srid** | Session role id — `conversation_state_role_id(mrid, session_id)` |
| **pl** | Resolved plugins for the current turn |
| **Minimal tool kernel** | One turn/lifecycle orchestration, capability-call/merge rules, authoritative commits, errors/isolation, and six stable capability ports; not yet a separately compiled crate |
| **Complete reference runtime** | Current `oclive_kernel_host::OcliveKernel` facade plus SQLite, Event Ring, concrete implementations, facilities, and HTTP dependencies |
| **Six slots** | memory · emotion · event · prompt · llm · agent |
| **plugin_backends** | Legacy six-key disk structure plus the folded runtime `PluginBackends` view; not the current blueprint write authority |
| **slot_registry** | v2/v3/v4 blueprint multi-instance registry (admin layer); new Stable packs use v4 |
| **reply** | API response field (**not** `response`) |
| **OOCP** | OCLive Open Chat Protocol — HTTP black-box tests |
| **Event Ring** | Bounded event-routing facility in the reference runtime; not a minimal-core requirement, seventh slot, or database bus |
| **Explicit / implicit assembly** | Mixable strategies: contract-shaped shared state vs model inference from context; neither is mandatory |
| **State candidate** | Target form for semantic interpretations with source, confidence, TTL, and scope; not yet implemented uniformly across DTOs |
| **[Runtime Event Stream (planned)](../creator-docs-en/rfc/RFC_RUNTIME_EVENT_STREAM.md)** | Future durable cross-turn, cross-channel timeline; Production storage/readers/consumers are not implemented. Only an off-by-default, non-readable B0 trace-only shadow exists, and it is not the current Ring or a shipped Stream |
| **legacy `event` slot** | Backend module 3; dialogue event-impact estimation only |
| **EventDraft / EventEnvelope** | Untrusted proposal content / Ring-issued event with trusted source, weight, order, and causation |
| **EventEmitter** | Source-bound handle obtained through registration; emission scope is declared |
| **influence weight** | Base proposal influence for a decision module; not priority or final authority |
| **Event decision module** | Admits/rejects a subscribed proposal; Rust remains overall turn authority |
| **TurnOrigin / TurnInput** | Typed source (`user`/`sensor`/`system`) and input (`UserMessage`/`ExternalObservation`) |
| **ProactiveTurnPermit** | One-shot host permit produced after authoritative Event admission |
| **Turn Thinking** | Fast/Deep turn policy (**not** a seventh slot) |

Chinese SSOT: [human-docs/03_GLOSSARY.md](../human-docs/03_GLOSSARY.md)
