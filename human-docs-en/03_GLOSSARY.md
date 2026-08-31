# 03 · Glossary

| Term | Meaning |
|------|---------|
| **mrid** | Manifest role id — pack folder / manifest role id |
| **srid** | Session role id — `conversation_state_role_id(mrid, session_id)` |
| **pl** | Resolved plugins for the current turn |
| **Six slots** | memory · emotion · event · prompt · llm · agent |
| **plugin_backends** | Legacy/runtime six-key backend selection |
| **slot_registry** | Blueprint multi-instance slot registry (admin layer) |
| **reply** | API response field (**not** `response`) |
| **OOCP** | OCLive Open Chat Protocol — HTTP black-box tests |
| **Event Ring** | Bounded in-process authoritative event routing; not a seventh slot or database bus |
| **legacy `event` slot** | Backend module 3; dialogue event-impact estimation only |
| **EventDraft / EventEnvelope** | Untrusted proposal content / Ring-issued event with trusted source, weight, order, and causation |
| **EventEmitter** | Source-bound handle obtained through registration; emission scope is declared |
| **influence weight** | Base proposal influence for a decision module; not priority or final authority |
| **Event decision module** | Admits/rejects a subscribed proposal; Rust remains overall turn authority |
| **TurnOrigin / TurnInput** | Typed source (`user`/`sensor`/`system`) and input (`UserMessage`/`ExternalObservation`) |
| **ProactiveTurnPermit** | One-shot host permit produced after authoritative Event admission |
| **Turn Thinking** | Fast/Deep turn policy (**not** a seventh slot) |

Chinese SSOT: [human-docs/03_GLOSSARY.md](../human-docs/03_GLOSSARY.md)
