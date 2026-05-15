# English documentation (oclive / oclivenewnew)

[中文总索引](../creator-docs/getting-started/DOCUMENTATION_INDEX.md)

This tree mirrors **`creator-docs/`** with hand-maintained English pages. For topics without an English file yet, use the Chinese hub or follow links inside each page.

---

## Getting started

| Topic | English |
|-------|---------|
| Documentation hub (quick links, RFC, reading order) | [getting-started/DOCUMENTATION_INDEX.md](getting-started/DOCUMENTATION_INDEX.md) |
| **Status & goal alignment (one-page hub)** | [getting-started/PROJECT_STATUS_AND_ALIGNMENT.md](getting-started/PROJECT_STATUS_AND_ALIGNMENT.md) |
| **Current status (version, ships, changelog)** | [getting-started/PROJECT_CURRENT_STATUS.md](getting-started/PROJECT_CURRENT_STATUS.md) |
| Project overview (repos, commands, checklist) | [getting-started/PROJECT_OVERVIEW.md](getting-started/PROJECT_OVERVIEW.md) |
| Kernel-centric module diagram (Mermaid + static figure) | [getting-started/KERNEL_AND_MODULES_ARCHITECTURE.md](getting-started/KERNEL_AND_MODULES_ARCHITECTURE.md) |
| Error codes & triage | [getting-started/ERROR_CODES.md](getting-started/ERROR_CODES.md) |

---

## Testing

| Topic | English |
|-------|---------|
| Where tests live (main repo vs pack editor) | [testing/OVERVIEW.md](testing/OVERVIEW.md) |
| OOCP HTTP suite (S0–S11) | [testing/OOCP_TEST_SUITE.md](testing/OOCP_TEST_SUITE.md) |
| Test output and contracts | [testing/TEST_OUTPUT_SCHEMA.md](testing/TEST_OUTPUT_SCHEMA.md) |
| Rust integration tests for swappable backends | [testing/ADAPTING_TEST_PLUGIN.md](testing/ADAPTING_TEST_PLUGIN.md) |

---

## Role pack

| Topic | English |
|-------|---------|
| On-disk role pack format | [role-pack/ROLE_PACK_SPEC.md](role-pack/ROLE_PACK_SPEC.md) |
| Community index JSON | [role-pack/ROLE_PACK_INDEX.md](role-pack/ROLE_PACK_INDEX.md) |

---

## Security

| Topic | English |
|-------|---------|
| Known vulnerabilities (`cargo-audit`) | [security/KNOWN_VULNERABILITIES.md](security/KNOWN_VULNERABILITIES.md) |
| Audit scope and limitations | [security/SECURITY_AUDIT_SCOPE.md](security/SECURITY_AUDIT_SCOPE.md) |

---

## CLI and RFC

| Topic | English |
|-------|---------|
| `oclive-cli` user guide | [cli/OCLIVE_CLI_GUIDE.md](cli/OCLIVE_CLI_GUIDE.md) |
| `settings.json` → `plugin_backends` reference | [cli/SETTINGS_REFERENCE.md](cli/SETTINGS_REFERENCE.md) |
| Monolith / high-coupling compile mode (RFC) | [rfc/RFC_OCLIVE_MONOLITH_MODE.md](rfc/RFC_OCLIVE_MONOLITH_MODE.md) |

---

## Release engineering baseline

| Topic | English |
|-------|---------|
| Lightweight profile, `cargo audit`, `cargo-bloat` | [LIGHTWEIGHT_PROFILE.md](LIGHTWEIGHT_PROFILE.md) |

---

## Plugin architecture & contracts

| Topic | English |
|-------|---------|
| PLUGIN_V1 — `plugin_backends` contract | [plugin-and-architecture/PLUGIN_V1.md](plugin-and-architecture/PLUGIN_V1.md) |
| Remote HTTP JSON-RPC (host ↔ sidecar) | [plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md](plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md) |
| Directory plugins (`plugins/`, manifest, shell, invoke) | [plugin-and-architecture/DIRECTORY_PLUGINS.md](plugin-and-architecture/DIRECTORY_PLUGINS.md) |
| Whole-shell bridge `invoke` reference | [plugin-and-architecture/BRIDGE_API_REFERENCE.md](plugin-and-architecture/BRIDGE_API_REFERENCE.md) |
| Extension points index (traits → source files) | [plugin-and-architecture/EXTENSION_POINTS.md](plugin-and-architecture/EXTENSION_POINTS.md) |
| Creator architecture (sidecar vs directory vs fork) | [plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md](plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md) |
| How to replace modules (builtin / remote / directory) | [plugin-and-architecture/HOW_TO_REPLACE_MODULES.md](plugin-and-architecture/HOW_TO_REPLACE_MODULES.md) |
| Local plugin bridge (`memory = local`, `_local_plugins`) | [plugin-and-architecture/LOCAL_PLUGIN_BRIDGE_SPEC.md](plugin-and-architecture/LOCAL_PLUGIN_BRIDGE_SPEC.md) |
| Plugin & pack FAQ | [FAQ.md](FAQ.md) |
| Pack editor vs host compatibility (`ui.json`) | [COMPATIBILITY.md](COMPATIBILITY.md) |

---

## Guides

| Topic | English |
|-------|---------|
| Configuration paths (`plugin_state`, `ui.json`, `{app_data}`) | [guides/CONFIGURATION_FILES.md](guides/CONFIGURATION_FILES.md) |
| mumu UI release checklist (`ui.json` + directory slots) | [guides/MUMU_UI_ACCEPTANCE_CHECKLIST.md](guides/MUMU_UI_ACCEPTANCE_CHECKLIST.md) |
| Regression: Plugin Manager V2 & Complex Emotion | [guides/REGRESSION_COMPLEX_EMOTION_QA.md](guides/REGRESSION_COMPLEX_EMOTION_QA.md) |

---

## Legal & licensing

| Topic | English |
|-------|---------|
| Open-source policy (host & plugins) | [LICENSE_POLICY.md](LICENSE_POLICY.md) |

---

## Full Chinese corpus

- [creator-docs/getting-started/DOCUMENTATION_INDEX.md](../creator-docs/getting-started/DOCUMENTATION_INDEX.md)

---

## Application README

- [README.en.md](../README.en.md) (English) · [README.md](../README.md) (中文)

---

## Contributing

- [CONTRIBUTING.en.md](../CONTRIBUTING.en.md) · [CONTRIBUTING.md](../CONTRIBUTING.md)

---

[中文](../creator-docs/README.md)
