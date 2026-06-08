# `oclive_kernel_host/src/domain` dependency rules

Orchestration and business-policy layer. New code should follow these directions (aligned with [ARCHITECTURE_LAYERING.md](../../../../handoff/ARCHITECTURE_LAYERING.md)).

Chinese handoff notes: [COMMENT_ENGLISH_MIGRATION_PLAN.md](../../../../handoff/COMMENT_ENGLISH_MIGRATION_PLAN.md).

## Allowed / forbidden

| Module | May depend on | Must not depend on |
|--------|---------------|-------------------|
| `domain/` | `domain/`, `models/`, `error/`, `domain/ports/`, `oclive_kernel_*` | `api/` |
| `infrastructure/` | `domain/`, `infrastructure/`, `models/` | `api/` |
| `api/` | `domain/`, `infrastructure/`, `state/` | — |

## Layering ratchet (D-LAYER-05)

`node scripts/check-domain-layering.mjs` enforces two counters under `domain/**/*.rs` (baseline: [LAYERING_BASELINE.json](../../../../handoff/LAYERING_BASELINE.json)):

| Counter | Baseline (2026-06-08) | Meaning |
|---------|----------------------|---------|
| `use crate::infrastructure` imports | **4** (all `#[cfg(test)]`) | Top-level `use` lines |
| `crate::infrastructure::` FQ refs (production) | **5** | Fully-qualified paths outside test cfg |

**Do not increase either counter.** Ratchet down only when extracting ports.

### Production FQ-path refs (5)

| File | Refs | Notes |
|------|------|-------|
| `user_llm_env.rs` | 3× `DbManager` | Env provider DB reads; candidate for a small port |
| `startup_health.rs` | 1× `DbManager` | `run_global_db_ping` |
| `plugin_host/mod.rs` | 1× (doc link) | Module docs only; may be skipped by heuristic |

Turn hot-path persistence now goes through **`domain/ports/`** traits with implementations in [`infrastructure/turn_ports.rs`](../infrastructure/turn_ports.rs):

- `ChatTurnPersistencePort` → `DbChatTurnPersistencePort`
- `TurnPoliciesPort` → `AppTurnPoliciesPort`
- `ConversationPersistPort` → `StoreConversationPersistPort`

`turn_pipeline/persistence.rs` consumes these ports; `post.rs` still calls `state.policies_for_scene` in-domain (D-LAYER-05b follow-up).

### `use`-import adapter layer (4, test-only)

These files still call `use crate::infrastructure::…` directly (constructing `PluginHost`, Remote HTTP, directory child processes, etc.):

- `ports/plugin_host/`、`role_manager.rs`、`agent.rs`、`slot_resolver.rs`、`role_manifest_validate.rs`

**Reason**: implementations are not fully moved into `infrastructure/*_wiring` + port factories yet. **Prefer extending `domain/ports` traits** for new features; do not add new `domain → api` references.

## Orchestration entry points

| File | Role |
|------|------|
| `chat_engine/process_message.rs` | Single-message dispatch (health check, dual-core gate, remote branches); builds `EffectiveSessionConfig`, `TurnPrefetch`, `RoleRuntimeSnapshot` once per turn |
| `chat_engine/turn_pipeline/` | Co-present turn execution |
| `chat_engine/turn_prefetch.rs` | Shared prefetch (recent context, user identity, complex emotion hint) |
| `role_runtime_snapshot.rs` | Single DB read for hot `role_runtime` fields |
| `dual_pipeline.rs` | Dual-core experimental + stable fallback |
| `slot_runner.rs` | Multi-instance slot merge and invocation |
| `ports/plugin_host/` | `plugin_backends` → `Arc<dyn …>` |

Naming SSOT: [NAMING_CONVENTIONS.md](../../../../creator-docs/NAMING_CONVENTIONS.md).
