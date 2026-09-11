# 03 · Glossary

Field names and runtime strategies below describe the **current reference Host**, not mandatory minimal Kernel fields. Responsibilities: [MODULE_MAP](../handoff/MODULE_MAP_AND_HANDOFF.md#kernel-responsibilities); names: [NAMING](../creator-docs-en/NAMING_CONVENTIONS.md).

| Term | Meaning |
|------|---------|
| **mrid** | Historical shorthand for the reference Host's role ID, resolved according to the loaded format; not every pack must contain a manifest |
| **srid** | Session role id — `conversation_state_role_id(mrid, session_id)` |
| **pl** | Resolved plugins for the current turn |
| **Minimal tool kernel** | Six-slot capability contracts and necessary public legality/causal/error boundaries; not concrete scheduling or domain commits, and not yet a separately compiled crate |
| **Distro Host** | Runtime composition, bounded scheduling, and domain application; not just a transport bridge or OS process |
| **Adapter** | Protocol conversion or execution of authorized operations, without independent domain decision authority |
| **Complete reference runtime** | Current `oclive_kernel_host::OcliveKernel` facade plus SQLite, Event Ring, concrete implementations, facilities, and HTTP dependencies |
| **Six slots** | memory · emotion · event · prompt · llm · agent |
| **plugin_backends** | Legacy six-key disk structure plus the folded runtime `PluginBackends` view; not the current blueprint write authority |
| **slot_registry** | Reference-Host v2/v3/v4 blueprint instance registry; new Stable packs in that format family use v4. Not a minimal-role prerequisite |
| **reply** | API response field (**not** `response`) |
| **OOCP** | OCLive Open Chat Protocol — HTTP black-box tests |
| **Event Ring** | Bounded event-routing facility in the reference runtime; not a minimal-core requirement, seventh slot, or database bus |
| **Explicit / implicit assembly** | Mixable strategies: contract-shaped shared state vs model inference from context; neither is mandatory |
| **State candidate** | Target form for semantic interpretations with source, confidence, TTL, and scope; not yet implemented uniformly across DTOs |
| **[Runtime Event Stream](../creator-docs-en/rfc/RFC_RUNTIME_EVENT_STREAM.md)** | Peripheral cross-turn/channel direction; trace/shadow and later experiments do not equal a Production Stream. Consult the RFC and technical debt for implemented slices, gaps, and pauses; do not conflate it with the bounded Ring |
| **legacy `event` slot** | Backend module 3; dialogue event-impact estimation only |
| **EventDraft / EventEnvelope** | Untrusted proposal content / Ring-issued event with trusted source, weight, order, and causation |
| **EventEmitter** | Source-bound handle obtained through registration; emission scope is declared |
| **influence weight** | Base proposal influence for a decision module; not priority or final authority |
| **Event decision module** | Admits/rejects a subscribed proposal; Rust remains overall turn authority |
| **TurnOrigin / TurnInput** | Typed source (`user`/`sensor`/`system`) and input (`UserMessage`/`ExternalObservation`) |
| **ProactiveTurnPermit** | One-shot host permit produced after authoritative Event admission |
| **Turn Thinking** | Fast/Deep turn policy (**not** a seventh slot) |

Chinese SSOT: [human-docs/03_GLOSSARY.md](../human-docs/03_GLOSSARY.md)
