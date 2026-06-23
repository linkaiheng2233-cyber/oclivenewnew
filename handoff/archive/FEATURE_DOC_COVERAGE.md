# Feature ↔ documentation coverage

**Date:** 2026-05-20  
**Index entry:** [DOCUMENTATION_INDEX.md](../creator-docs/getting-started/DOCUMENTATION_INDEX.md)

| Feature area | Doc entry | Covered |
|--------------|-----------|---------|
| Kernel orchestration | [KERNEL_AND_MODULES_ARCHITECTURE.md](../creator-docs/getting-started/KERNEL_AND_MODULES_ARCHITECTURE.md), [PURE_KERNEL_BOUNDARY.md](../creator-docs/getting-started/PURE_KERNEL_BOUNDARY.md), [architecture/DESIGN_DECISIONS.md](../creator-docs/architecture/DESIGN_DECISIONS.md) | ✓ blueprint load via `storage.rs` / merge in DESIGN_DECISIONS |
| Blueprint & role pack | [ROLE_PACK_SPEC.md](../creator-docs/role-pack/ROLE_PACK_SPEC.md), [V1_TO_V2_MIGRATION.md](../creator-docs/role-pack/V1_TO_V2_MIGRATION.md) | ✓ `groups`, `slot_registry` |
| Plugin system | [PLUGIN_V1.md](../creator-docs/plugin-and-architecture/PLUGIN_V1.md), [DIRECTORY_PLUGINS.md](../creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md) | ✓ `complex_emotion` + `provides` |
| Monolith build | [RFC_OCLIVE_MONOLITH_MODE.md](../creator-docs/rfc/RFC_OCLIVE_MONOLITH_MODE.md) | ✓ |
| oclive-cli | [OCLIVE_CLI_GUIDE.md](../creator-docs/cli/OCLIVE_CLI_GUIDE.md), [SETTINGS_REFERENCE.md](../creator-docs/cli/SETTINGS_REFERENCE.md) | ✓ 21 commands; `--smart`, `--deny`, `--oocp` |
| Studio | [`handoff/studio/USER_GUIDE.md`](../studio/USER_GUIDE.md) | ✓ create mode, trial chat, diagnostics, export; undo/redo = editor UX (studio repo) |
| Testing | [TESTING_GUIDE.md](../creator-docs/testing/TESTING_GUIDE.md), [PERFORMANCE.md](../creator-docs/getting-started/PERFORMANCE.md) | ✓ unit / e2e / OOCP paths |
| Security | [DISCLAIMER.md](../creator-docs/legal/DISCLAIMER.md), [KNOWN_VULNERABILITIES.md](../creator-docs/security/KNOWN_VULNERABILITIES.md), [SECURITY_AUDIT_SCOPE.md](../creator-docs/security/SECURITY_AUDIT_SCOPE.md) | ✓ `cargo-audit` baseline; `cargo deny` in CLI `lint --deny` + DISCLAIMER §4 |
