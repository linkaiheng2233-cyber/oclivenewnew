# English documentation (oclive / oclivenewnew)

[中文总索引](../creator-docs/getting-started/DOCUMENTATION_INDEX.md)

This tree mirrors **`creator-docs/`** with hand-maintained English pages. **Normative contracts remain Chinese SSOT** — English pages link back with **`[中文](…)`** on every mirrored topic.

## Mirror policy

| Principle | Detail |
|-----------|--------|
| **SSOT** | Simplified Chinese under `creator-docs/` (and `human-docs/` for the human ladder) |
| **English goal** | Full mirror by directory, phased — not a permanent “minimal subset” |
| **Page types** | **Full mirror** · **Summary + ZH link** · **Index-only (pending)** — see [coverage matrix](#mirror-coverage-matrix) |
| **Fallback** | No English file → open the linked Chinese page; do not treat English README tables as normative if ZH differs |
| **AI agents** | Use [handoff/AI_READING_INDEX.md](../handoff/AI_READING_INDEX.md) + [AGENTS.md](../AGENTS.md); this tree is for human readers and integrators |

## Navigate by role

| Who you are | Start here |
|-------------|------------|
| **End users** (desktop app only; no pack/plugin authoring) | [getting-started/USER_MANUAL.md](getting-started/USER_MANUAL.md) ([中文](../creator-docs/getting-started/USER_MANUAL.md)) |
| **Role pack authors** | [role-pack/CREATOR_LEARNING_PATH.md](role-pack/CREATOR_LEARNING_PATH.md) · [role-pack/ROLE_PACK_SPEC.md](role-pack/ROLE_PACK_SPEC.md) |
| **Plugin authors** | [plugin-and-architecture/PLUGIN_AUTHOR_LEARNING_PATH.md](plugin-and-architecture/PLUGIN_AUTHOR_LEARNING_PATH.md) · [plugin-and-architecture/PLUGIN_V1.md](plugin-and-architecture/PLUGIN_V1.md) · [plugin-and-architecture/PLUGIN_PLACEMENT_GUIDE.md](plugin-and-architecture/PLUGIN_PLACEMENT_GUIDE.md) |
| **Kernel / hardware integrators** | [getting-started/KERNEL_INTEGRATOR_LEARNING_PATH.md](getting-started/KERNEL_INTEGRATOR_LEARNING_PATH.md) · [kernel/DISTRO_CAPABILITY_PROFILE.md](kernel/DISTRO_CAPABILITY_PROFILE.md) · [kernel/DISTRO_KERNEL_LIFECYCLE.md](kernel/DISTRO_KERNEL_LIFECYCLE.md) |
| **Maintainers** (breaking changes, critical-path handoff) | [../handoff/BREAKING_CHANGE_PROCESS.md](../handoff/BREAKING_CHANGE_PROCESS.md) · [../handoff/BUS_FACTOR_NOTES.md](../handoff/BUS_FACTOR_NOTES.md) |

## Sync rules

When you change runtime or author-facing contracts (slots, `plugin_backends`, bridge, OOCP, pack schema):

1. If an **English mirror exists** for that Chinese file → update it in the **same PR**.
2. If no mirror yet → update Chinese SSOT + add a CHANGELOG note, or add the English mirror in the same release train.
3. Update the [coverage matrix](#mirror-coverage-matrix) row when a directory moves from **pending** to **mirrored** or **summary**.
4. Run **`node scripts/check-doc-mirror.mjs`** (also in `npm run check:rust` and dimension5 CI) before merge.

**Governance**: module definitions → [MODULE_MAP_AND_HANDOFF.md](../handoff/MODULE_MAP_AND_HANDOFF.md). Doc layer map → [handoff/README.md §文档分责](../handoff/README.md). Human ladder progress → [human-docs/README.md §文档包进度](../human-docs/README.md#文档包进度与-ai-包同步--2026-06-26). AI doc rules G10–G16 → [AI_CHANGE_BOUNDARIES.md](../handoff/AI_CHANGE_BOUNDARIES.md).

---

## Mirror coverage matrix

Last reviewed: **2026-07-10**. Counts are `*.md` files per directory (approximate).

| Directory | ZH files | EN status | Notes |
|-----------|----------|-----------|-------|
| `getting-started/` | ~22 | **Mirrored** | Hub + CREATOR_GOLDEN_PATH + learning paths + error codes |
| `plugin-and-architecture/` | ~12 | **Mirrored** | PLUGIN_V1, market submission, placement, agent remote |
| `kernel/` | 5 | **Mirrored** | HostProfile, lifecycle, app data, none semantics |
| `role-pack/` | ~16 | **Mirrored** | Spec + creator deep guides + cross-host + versioning |
| `testing/` | ~8 | **Mirrored** | OOCP, overview, fuzzing, narrative_hint, L03 |
| `security/` | 3 | **Mirrored** | KNOWN_VULN, audit scope, supply chain |
| `guides/` | 3 | **Mirrored** | Configuration, regression QA, mumu checklist |
| `roadmap/` | ~7 | **Mirrored** | Vision + APPLICATION_SCENARIOS |
| `rfc/` | ~12 | **Partial** | Full mirror for Monolith/dual-core/Studio; long RFCs → `*_SUMMARY.md` + ZH |
| `cli/` | 2 | **Mirrored** | CLI guide + settings reference |
| `storage/` | 1 | **Mirrored** | STORAGE_BACKEND_GUIDE |
| `legal/` | 1 | **Mirrored** | DISCLAIMER |
| `studio/` | 1 | **Mirrored** | USER_GUIDE |
| `development/` | 2 | **Mirrored** | RELEASE_VERSIONING, LIGHTWEIGHT_PROFILE |
| `dual-core/` | 2 | **Mirrored** | DEVELOPER_GUIDE, METHOD_REGISTRY |
| `video-script/` | 1 | **Pending** | Chinese-only script (index-only) |
| `architecture/` | 1 | **Mirrored** | [`DESIGN_DECISIONS`](../creator-docs/architecture-en/DESIGN_DECISIONS.md) (EN lives under `creator-docs/architecture-en/`) |
| Root | 3 | **Mirrored** | FAQ, LICENSE_POLICY, COMPATIBILITY, NAMING_CONVENTIONS |

**Legend**: **Mirrored** = English file per topic (full or intentional summary with ZH link). **Partial** = high-traffic paths done; long-tail creator/RFC pages may be ZH-only. **Pending** = index points to Chinese; mirror not committed yet.

---

## Getting started

| Topic | English |
|-------|---------|
| Documentation hub | [getting-started/DOCUMENTATION_INDEX.md](getting-started/DOCUMENTATION_INDEX.md) |
| User manual | [getting-started/USER_MANUAL.md](getting-started/USER_MANUAL.md) |
| Status & alignment | [getting-started/PROJECT_STATUS_AND_ALIGNMENT.md](getting-started/PROJECT_STATUS_AND_ALIGNMENT.md) |
| Current status | [getting-started/PROJECT_CURRENT_STATUS.md](getting-started/PROJECT_CURRENT_STATUS.md) |
| Project overview | [getting-started/PROJECT_OVERVIEW.md](getting-started/PROJECT_OVERVIEW.md) |
| Kernel-centric diagram | [getting-started/KERNEL_AND_MODULES_ARCHITECTURE.md](getting-started/KERNEL_AND_MODULES_ARCHITECTURE.md) |
| Error codes | [getting-started/ERROR_CODES.md](getting-started/ERROR_CODES.md) · [KERNEL_ERROR_CODE_CONVENTION.md](getting-started/KERNEL_ERROR_CODE_CONVENTION.md) |

---

## Kernel

| Topic | English |
|-------|---------|
| Distro capability profile (`distro.oclive.toml`) | [kernel/DISTRO_CAPABILITY_PROFILE.md](kernel/DISTRO_CAPABILITY_PROFILE.md) |
| Kernel lifecycle (attach / spawn / replace) | [kernel/DISTRO_KERNEL_LIFECYCLE.md](kernel/DISTRO_KERNEL_LIFECYCLE.md) |
| Default plugin matrix per distro | [kernel/DISTRO_DEFAULT_PLUGINS.md](kernel/DISTRO_DEFAULT_PLUGINS.md) |
| Six-slot `none` semantics | [kernel/MODULE_NONE_SEMANTICS.md](kernel/MODULE_NONE_SEMANTICS.md) |
| Cross-host `OCLIVE_APP_DATA` | [kernel/OCLIVE_APP_DATA.md](kernel/OCLIVE_APP_DATA.md) |

---

## Testing

| Topic | English |
|-------|---------|
| Where tests live | [testing/OVERVIEW.md](testing/OVERVIEW.md) |
| OOCP HTTP suite (S0–S12) | [testing/OOCP_TEST_SUITE.md](testing/OOCP_TEST_SUITE.md) |
| Test output schema | [testing/TEST_OUTPUT_SCHEMA.md](testing/TEST_OUTPUT_SCHEMA.md) |
| Swappable backend integration tests | [testing/ADAPTING_TEST_PLUGIN.md](testing/ADAPTING_TEST_PLUGIN.md) |
| Fuzzing (proptest + cargo-fuzz) | [testing/FUZZING.md](testing/FUZZING.md) |
| `narrative_hint` contract | [testing/NARRATIVE_HINT_CONTRACT.md](testing/NARRATIVE_HINT_CONTRACT.md) |
| L03 generation cancel (planned) | [testing/L03_GENERATION_CANCEL.md](testing/L03_GENERATION_CANCEL.md) |

---

## Role pack

| Topic | English |
|-------|---------|
| On-disk format | [role-pack/ROLE_PACK_SPEC.md](role-pack/ROLE_PACK_SPEC.md) |
| Community index JSON | [role-pack/ROLE_PACK_INDEX.md](role-pack/ROLE_PACK_INDEX.md) |
| Versioning & compatibility | [role-pack/PACK_VERSIONING.md](role-pack/PACK_VERSIONING.md) |
| Cross-host memory (L1/L2/L3) | [role-pack/CROSS_HOST_MEMORY.md](role-pack/CROSS_HOST_MEMORY.md) |
| v1 → v2 migration | [role-pack/V1_TO_V2_MIGRATION.md](role-pack/V1_TO_V2_MIGRATION.md) |

---

## Security

| Topic | English |
|-------|---------|
| Known vulnerabilities | [security/KNOWN_VULNERABILITIES.md](security/KNOWN_VULNERABILITIES.md) |
| Audit scope | [security/SECURITY_AUDIT_SCOPE.md](security/SECURITY_AUDIT_SCOPE.md) |
| Supply chain | [security/SUPPLY_CHAIN.md](security/SUPPLY_CHAIN.md) |

---

## Plugin architecture & contracts

| Topic | English |
|-------|---------|
| PLUGIN_V1 | [plugin-and-architecture/PLUGIN_V1.md](plugin-and-architecture/PLUGIN_V1.md) |
| **Placement guide** (decision tree) | [plugin-and-architecture/PLUGIN_PLACEMENT_GUIDE.md](plugin-and-architecture/PLUGIN_PLACEMENT_GUIDE.md) |
| Remote HTTP JSON-RPC | [plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md](plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md) |
| **Agent remote / directory** | [plugin-and-architecture/AGENT_REMOTE_PROTOCOL.md](plugin-and-architecture/AGENT_REMOTE_PROTOCOL.md) |
| Directory plugins | [plugin-and-architecture/DIRECTORY_PLUGINS.md](plugin-and-architecture/DIRECTORY_PLUGINS.md) |
| Bridge `invoke` reference | [plugin-and-architecture/BRIDGE_API_REFERENCE.md](plugin-and-architecture/BRIDGE_API_REFERENCE.md) |
| Extension points | [plugin-and-architecture/EXTENSION_POINTS.md](plugin-and-architecture/EXTENSION_POINTS.md) |
| Creator plugin architecture | [plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md](plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md) |
| Replace modules | [plugin-and-architecture/HOW_TO_REPLACE_MODULES.md](plugin-and-architecture/HOW_TO_REPLACE_MODULES.md) |
| Local plugin bridge | [plugin-and-architecture/LOCAL_PLUGIN_BRIDGE_SPEC.md](plugin-and-architecture/LOCAL_PLUGIN_BRIDGE_SPEC.md) |
| FAQ · Compatibility | [FAQ.md](FAQ.md) · [COMPATIBILITY.md](COMPATIBILITY.md) |

---

## Guides · Legal · CLI · RFC

| Topic | English |
|-------|---------|
| Configuration paths | [guides/CONFIGURATION_FILES.md](guides/CONFIGURATION_FILES.md) |
| Regression QA (complex emotion) | [guides/REGRESSION_COMPLEX_EMOTION_QA.md](guides/REGRESSION_COMPLEX_EMOTION_QA.md) |
| License policy | [LICENSE_POLICY.md](LICENSE_POLICY.md) |
| `oclive-cli` | [cli/OCLIVE_CLI_GUIDE.md](cli/OCLIVE_CLI_GUIDE.md) · [cli/SETTINGS_REFERENCE.md](cli/SETTINGS_REFERENCE.md) |
| Monolith RFC | [rfc/RFC_OCLIVE_MONOLITH_MODE.md](rfc/RFC_OCLIVE_MONOLITH_MODE.md) |
| Turn Thinking summary | [rfc/RFC_TURN_THINKING_PERSISTENCE_SUMMARY.md](rfc/RFC_TURN_THINKING_PERSISTENCE_SUMMARY.md) |
| Lightweight profile | [development/LIGHTWEIGHT_PROFILE.md](development/LIGHTWEIGHT_PROFILE.md) |
| Naming conventions | [NAMING_CONVENTIONS.md](NAMING_CONVENTIONS.md) |
| Creator golden path | [getting-started/CREATOR_GOLDEN_PATH.md](getting-started/CREATOR_GOLDEN_PATH.md) |
| Plugin market submission | [plugin-and-architecture/PLUGIN_MARKET_SUBMISSION.md](plugin-and-architecture/PLUGIN_MARKET_SUBMISSION.md) |
| Dual-core developer guide | [dual-core/DEVELOPER_GUIDE.md](dual-core/DEVELOPER_GUIDE.md) |
| Release versioning | [development/RELEASE_VERSIONING.md](development/RELEASE_VERSIONING.md) |

---

## Full Chinese corpus

- [creator-docs/getting-started/DOCUMENTATION_INDEX.md](../creator-docs/getting-started/DOCUMENTATION_INDEX.md)
- [human-docs-en/README.md](../human-docs-en/README.md) — human learning ladder (English)

---

## Application README

- [README.en.md](../README.en.md) (English) · [README.md](../README.md) (中文)

---

[中文](../creator-docs/README.md)
