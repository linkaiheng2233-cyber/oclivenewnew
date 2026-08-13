# 07 · Common task recipes

[中文](../human-docs/07_COMMON_TASKS.md)

> **Audience**: Contributors who finished L5 and need to edit code.  
> **After reading**: Find primary files and docs/tests to sync per scenario.  
> **Time**: On demand.  
> **Next**: [08 Reference map](08_REFERENCE_MAP.md) · shortcut [07 First PR](07_FIRST_PR.md).

Full navigation table: [CONTRIBUTING.en.md § Code navigation](../CONTRIBUTING.en.md).

---

## 1. Add a Tauri command

| Step | Location |
|------|----------|
| Implement | `distros/desktop-tauri/src/api/<topic>.rs` |
| Register | `distros/desktop-tauri/src/lib.rs` → `generate_handler!` |
| Frontend | `distros/shared/src/api/*.ts` (**camelCase** keys, e.g. `pluginId`) |
| Business | Delegate to `oclive_kernel_host::service::*_impl` — **no orchestration in api** |

**Also sync**: DTO changes → `oclive_kernel_types` + [ERROR_CODES](../creator-docs-en/getting-started/ERROR_CODES.md)

---

## 2. Change Prompt sections

| Step | Location |
|------|----------|
| Section formulas | `kernel/crates/oclive_kernel_runtime/src/domain/prompt_builder/sections.rs` |
| Assembly order | `prompt_builder/mod.rs` |
| Input injection | `turn_pipeline/pre.rs` → `PromptInput` fields |
| Guardrails | **Kernel constant** `KERNEL_DIALOGUE_GUARDRAILS` — not replaceable by role pack |

**Constraint**: `build_prompt(&PromptInput)` returns `String`, not `Result`.

---

## 3. Add a `config.json` field

**Human module pack**: [modules/packs/role-pack-config.md](../human-docs/modules/packs/role-pack-config.md) (ZH)

| Step | Location |
|------|----------|
| Parse | `RoleStorage::load_role` / loaders |
| Validate | `kernel/crates/oclive_validation` |
| Doc | [ROLE_PACK_SPEC](../creator-docs-en/role-pack/ROLE_PACK_SPEC.md) |
| Use | Matching `*_engine` or `turn_pipeline`, **not** API layer |

---

## 4. Domain unit test (good first PR)

```rust
let state = AppState::new_in_memory_with_llm(/* … */).await;
// call domain fn, assert Result
```

References: `distros/desktop-tauri/tests/invoke_hotpath_matrix.rs`, `narrative_hint_prompt_roundtrip.rs`

**Command**: `cargo test -p oclivenewnew-tauri --test <name>` or `npm run check:release`

---

## 5. Change `plugin_backends` / slot resolution

| Step | Location |
|------|----------|
| Merge rules | `slot_runner.rs` (read file header) |
| Resolve | `slot_resolver.rs`, `plugin_host/resolver.rs` |
| Backend table | `infrastructure/backend_registry.rs` |
| Validate | `oclive_validation` + [PLUGIN_V1](../creator-docs-en/plugin-and-architecture/PLUGIN_V1.md) |

---

## 6. New persistence field

| Step | Location |
|------|----------|
| SQL | `kernel/crates/oclive_kernel_host/migrations/0NN_*.sql` |
| trait | `domain/repository.rs` |
| impl | `infrastructure/repositories.rs` |
| **Forbidden** | Inventing table names — follow `001_init.sql` + migrations |

---

## 7. Change HTTP / OOCP contract

| Step | Location |
|------|----------|
| Routes | `oclive_kernel_host/src/http_api/` |
| DTO | `oclive_kernel_types/src/models/dto/mod.rs` |
| Error codes | [KERNEL_ERROR_CODE_CONVENTION](../creator-docs-en/getting-started/KERNEL_ERROR_CODE_CONVENTION.md) |
| Black-box | `examples/oocp-test-suite/` |

---

## 8. Adjust co-present stage order (high risk)

| Step | Location |
|------|----------|
| Main orchestration | `turn_pipeline.rs` / `co_present.rs` |
| **Caution** | Core orchestration — needs OOCP / integration tests + [DESIGN_DECISIONS](../creator-docs/architecture/DESIGN_DECISIONS.md) (ZH) |

---

## Checklist

- [ ] Can open the primary file for a task type in 30 seconds
- [ ] New Tauri commands sync `distros/shared/src/api/` camelCase

---

## Deep links

- [EXTENSION_POINTS](../creator-docs-en/plugin-and-architecture/EXTENSION_POINTS.md)
- [BUS_FACTOR_NOTES](../handoff/BUS_FACTOR_NOTES.md)
