# `oclive_kernel_host/src/domain` dependency rules

Orchestration and business-policy layer. New code should follow these directions (aligned with [ARCHITECTURE_LAYERING.md](../../../../handoff/ARCHITECTURE_LAYERING.md)).

Chinese handoff notes: [COMMENT_ENGLISH_MIGRATION_PLAN.md](../../../../handoff/COMMENT_ENGLISH_MIGRATION_PLAN.md).

## Allowed / forbidden

| Module | May depend on | Must not depend on |
|--------|---------------|-------------------|
| `domain/` | `domain/`, `models/`, `error/`, `domain/ports/`, `oclive_kernel_*` | `api/` |
| `infrastructure/` | `domain/`, `infrastructure/`, `models/` | `api/` |
| `api/` | `domain/`, `infrastructure/`, `state/` | — |

## Known adapter layer (being tightened)

These files still call `use crate::infrastructure::…` directly (constructing `PluginHost`, Remote HTTP, directory child processes, etc.):

- `ports/plugin_host/`、`role_manager.rs`、`agent.rs`、`slot_resolver.rs`、`role_manifest_validate.rs`

**Reason**: implementations are not fully moved into `infrastructure/*_wiring` + port factories yet. **Prefer extending `domain/ports` traits** for new features; do not add new `domain → api` references.

## Orchestration entry points

| File | Role |
|------|------|
| `chat_engine/process_message.rs` | Single-message dispatch (health check, dual-core gate, remote branches) |
| `chat_engine/turn_pipeline/` | Co-present turn execution |
| `dual_pipeline.rs` | Dual-core experimental + stable fallback |
| `slot_runner.rs` | Multi-instance slot merge and invocation |
| `ports/plugin_host/` | `plugin_backends` → `Arc<dyn …>` |

Naming SSOT: [NAMING_CONVENTIONS.md](../../../../creator-docs/NAMING_CONVENTIONS.md).
