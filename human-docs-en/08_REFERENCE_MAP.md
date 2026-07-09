# 08 · Reference map (by topic)

[中文](../human-docs/08_REFERENCE_MAP.md)

> **Audience**: Engineers after L0–L6 who need deep docs on demand.  
> **After reading**: Find SSOT in `creator-docs/` / `handoff/` by topic — not a flat link dump.  
> **Time**: On demand.  
> **Next**: [human-docs-en/README](README.md) or [ai-package/README](../human-docs/ai-package/README.md).

**English mirrors**: High-traffic contracts in [creator-docs-en/](../creator-docs-en/); long-tail may be Chinese-only — see [coverage matrix](../creator-docs-en/README.md#mirror-coverage-matrix).

---

## 1. Architecture

**Module picker** → [modules/README.md](modules/README.md) (links MODULE_MAP, no copied tables)

| Doc | Purpose |
|-----|---------|
| [OCLIVE_ARCHITECTURE_OVERVIEW](../creator-docs-en/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md) | Modules 1–6, facility submodules |
| [RFC Turn Thinking summary](../creator-docs-en/rfc/RFC_TURN_THINKING_PERSISTENCE_SUMMARY.md) | Fast/Deep · persistence ([full ZH RFC](../creator-docs/rfc/RFC_TURN_THINKING_PERSISTENCE.md)) |
| [kernel/crates/README](../kernel/crates/README.md) | Crate deps and where to edit |
| [DESIGN_DECISIONS](../creator-docs/architecture/DESIGN_DECISIONS.md) | Trade-off log (ZH) |
| [ARCHITECTURE_LAYERING](../handoff/ARCHITECTURE_LAYERING.md) | Layering ratchet |
| [ROLE_PACK_BOUNDARY](../handoff/ROLE_PACK_BOUNDARY.md) | Role vs blueprint |

---

## 2. Contracts and naming

| Doc | Purpose |
|-----|---------|
| [NAMING_CONVENTIONS](../creator-docs-en/getting-started/NAMING_CONVENTIONS.md) | Canonical names, imports |
| [dto.rs](../kernel/crates/oclive_kernel_types/src/models/dto.rs) | HTTP/IPC fields |
| [KERNEL_ERROR_CODE_CONVENTION](../creator-docs-en/getting-started/KERNEL_ERROR_CODE_CONVENTION.md) | Error JSON |
| [COMPATIBILITY](../creator-docs-en/COMPATIBILITY.md) | Version compatibility |
| [BREAKING_CHANGE_PROCESS](../handoff/BREAKING_CHANGE_PROCESS.md) | Breaking workflow |

---

## 3. Role pack

| Doc | Purpose |
|-----|---------|
| [ROLE_PACK_SPEC](../creator-docs-en/role-pack/ROLE_PACK_SPEC.md) | Pack spec |
| [CREATOR_LEARNING_PATH](../creator-docs-en/role-pack/CREATOR_LEARNING_PATH.md) | Creator path |
| [PACK_VERSIONING](../creator-docs-en/role-pack/PACK_VERSIONING.md) | Version rules |
| [CROSS_HOST_MEMORY](../creator-docs-en/role-pack/CROSS_HOST_MEMORY.md) | Cross-host memory |

---

## 4. Plugins

| Doc | Purpose |
|-----|---------|
| [PLUGIN_V1](../creator-docs-en/plugin-and-architecture/PLUGIN_V1.md) | Plugin contract |
| [PLUGIN_PLACEMENT_GUIDE](../creator-docs-en/plugin-and-architecture/PLUGIN_PLACEMENT_GUIDE.md) | Where to put capabilities |
| [PLUGIN_AUTHOR_LEARNING_PATH](../creator-docs-en/plugin-and-architecture/PLUGIN_AUTHOR_LEARNING_PATH.md) | Author path |
| [DIRECTORY_PLUGINS](../creator-docs-en/plugin-and-architecture/DIRECTORY_PLUGINS.md) | Directory plugins |
| [AGENT_REMOTE_PROTOCOL](../creator-docs-en/plugin-and-architecture/AGENT_REMOTE_PROTOCOL.md) | Agent sidecar |
| [REMOTE_PLUGIN_PROTOCOL](../creator-docs-en/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md) | HTTP JSON-RPC |
| [EXTENSION_POINTS](../creator-docs-en/plugin-and-architecture/EXTENSION_POINTS.md) | Extension index |

---

## 5. Kernel lifecycle

| Doc | Purpose |
|-----|---------|
| [DISTRO_KERNEL_LIFECYCLE](../creator-docs-en/kernel/DISTRO_KERNEL_LIFECYCLE.md) | attach / spawn |
| [KERNEL_SCHEDULER_RESCOPE](../handoff/KERNEL_SCHEDULER_RESCOPE.md) | Single kernel · fallback |
| [DISTRO_CAPABILITY_PROFILE](../creator-docs-en/kernel/DISTRO_CAPABILITY_PROFILE.md) | HostProfile · `[turn_thinking]` |
| [DISTRO_DEFAULT_PLUGINS](../creator-docs-en/kernel/DISTRO_DEFAULT_PLUGINS.md) | Distro plugin matrix |
| [OCLIVE_APP_DATA](../creator-docs-en/kernel/OCLIVE_APP_DATA.md) | Data directory |
| [KERNEL_INTEGRATOR_LEARNING_PATH](../creator-docs-en/getting-started/KERNEL_INTEGRATOR_LEARNING_PATH.md) | Integrators |
| [OCLIVE_CLI_GUIDE](../creator-docs-en/cli/OCLIVE_CLI_GUIDE.md) | CLI scaffold |
| [BUS_FACTOR_NOTES](../handoff/BUS_FACTOR_NOTES.md) | Critical path |
| [MODULE_MAP_AND_HANDOFF](../handoff/MODULE_MAP_AND_HANDOFF.md) | Module registry |

---

## 6. Testing and CI

| Doc | Purpose |
|-----|---------|
| [CONTRIBUTING.en § Tests](../CONTRIBUTING.en.md) | Local commands |
| [OOCP_TEST_SUITE](../creator-docs-en/testing/OOCP_TEST_SUITE.md) | HTTP black-box |
| [OVERVIEW](../creator-docs-en/testing/OVERVIEW.md) | Three test layers |
| [FUZZING](../creator-docs-en/testing/FUZZING.md) | Fuzz targets |
| [INVOKE_HOTPATH_MATRIX](../handoff/INVOKE_HOTPATH_MATRIX.md) | invoke matrix |
| [DIMENSION5_CLOSURE_SIGNOFF](../handoff/DIMENSION5_CLOSURE_SIGNOFF.md) | ratchet gates |

---

## 7. Release and security

| Doc | Purpose |
|-----|---------|
| [RELEASE_VERSIONING](../creator-docs/development/RELEASE_VERSIONING.md) | SemVer (ZH) |
| [KNOWN_VULNERABILITIES](../creator-docs-en/security/KNOWN_VULNERABILITIES.md) | Supply chain |
| [SUPPLY_CHAIN](../creator-docs-en/security/SUPPLY_CHAIN.md) | Trust model |
| [LIGHTWEIGHT_PROFILE](../creator-docs-en/LIGHTWEIGHT_PROFILE.md) | Binary size baseline |
| [CHANGELOG.en.md](../CHANGELOG.en.md) | User-visible changes |

---

## 8. handoff deep read (maintainers)

| Doc | Purpose |
|-----|---------|
| [handoff/README § doc layers](../handoff/README.md) | SSOT map · distro subdirs |
| [AI_CHANGE_BOUNDARIES](../handoff/AI_CHANGE_BOUNDARIES.md) | AI G1–G16; humans → [04 §8](04_ENGINEERING_RULES_SUMMARY.md#documentation-discipline) |
| [theater/](../handoff/theater/) | AI Theater |
| [vscode/](../handoff/vscode/) | VS Code Flash |
| [TECHNICAL_DEBT_INVENTORY](../handoff/TECHNICAL_DEBT_INVENTORY.md) | Active debt |
| [CHAT_STORAGE_ARCHITECTURE](../handoff/CHAT_STORAGE_ARCHITECTURE.md) | Chat vs memory |
| [DOCUMENTATION_INDEX](../creator-docs/getting-started/DOCUMENTATION_INDEX.md) | Full contract index (ZH hub) |

**Human pack progress**: [human-docs/README § progress](../human-docs/README.md#文档包进度与-ai-包同步--2026-06-26)

---

## Checklist

- [ ] When changing plugins, open §4 first — not whole-repo search
- [ ] Normative contracts: Chinese `creator-docs/` when EN/ZH differ
