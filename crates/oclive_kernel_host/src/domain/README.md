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

| Counter | Baseline (2026-06-10) | Meaning |
|---------|----------------------|---------|
| `use crate::infrastructure` imports | **4** (all `#[cfg(test)]`) | Top-level `use` lines |
| `crate::infrastructure::` FQ refs (production) | **1** | Fully-qualified paths outside test cfg |

**Do not increase either counter.** Ratchet down only when extracting ports.

### Production FQ-path refs (1)

| File | Refs | Notes |
|------|------|-------|
| `user_llm_env.rs` | 1× `db_ports::DbSettingsPort` | Wave 1 port adapter (D-ERR-01 remainder) |

`startup_health.rs` uses **`DbHealthPort`** ([`infrastructure/db_ports.rs`](../infrastructure/db_ports.rs)). `role_manager.rs` resolves plugins via constructor injection; `test_plugin_host` lives under `#[cfg(test)]` only.

Turn hot-path persistence now goes through **`domain/ports/`** traits with implementations in [`infrastructure/turn_ports.rs`](../infrastructure/turn_ports.rs):

- `ChatTurnPersistencePort` → `DbChatTurnPersistencePort`
- `TurnPoliciesPort` → `AppTurnPoliciesPort`
- `ConversationPersistPort` → `StoreConversationPersistPort`

`turn_pipeline/persistence.rs` consumes these ports; `post.rs` still calls `state.policies_for_scene` in-domain (D-LAYER-05b follow-up).

### `use`-import adapter layer (4, test-only)

These files still call `use crate::infrastructure::…` under `#[cfg(test)]` only:

- `event_impact_ai.rs`、`event_estimator.rs`、`complex_emotion_store.rs`、`mutable_profile_llm.rs`（`MockLlmClient` / `test_db`）

Plugin host / Remote / directory / reply post-processor factories live in `infrastructure/*_wiring` + `domain/ports`. **Prefer extending `domain/ports` traits** for new features; do not add new `domain → api` references.

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
