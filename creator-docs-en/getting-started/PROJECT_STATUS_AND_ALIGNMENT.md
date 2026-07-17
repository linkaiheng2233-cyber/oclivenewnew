# Project status & goal alignment (maintainer hub)

**One page**: where we are, where the canonical docs live, what to treat as source of truth for “what’s next.” **Does not replace** long-form topic docs.

[中文](../../creator-docs/getting-started/PROJECT_STATUS_AND_ALIGNMENT.md)

---

## How this differs from nearby docs

| Doc | Focus |
|-----|--------|
| **[PROJECT_OVERVIEW.md](PROJECT_OVERVIEW.md)** | Three repos, commands, release habits |
| **[PROJECT_CURRENT_STATUS.md](PROJECT_CURRENT_STATUS.md)** | **Fact snapshot**: semver, what ships, bilingual CHANGELOG pointers, sister repos & i18n |
| **[KERNEL_IMPLEMENTATION_PLAN.md](KERNEL_IMPLEMENTATION_PLAN.md)** | Kernel milestones **K0–K5**, north star, **verification log** |
| **[../../creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md](../../creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md)** | Monthly vision |
| **[../../creator-docs/roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md](../../creator-docs/roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md)** | Experience backlog; **does not replace** the monthly roadmap |
| **[../../handoff/PRODUCT_LINE_TASK_BUCKETS.md](../../handoff/PRODUCT_LINE_TASK_BUCKETS.md)** | Current product engineering execution view; old gap lists are historical context |

---

## Doc map by purpose

### 1. User-facing / handbook

| Topic | Entry |
|-------|--------|
| Hub & quick links | [DOCUMENTATION_INDEX.md](DOCUMENTATION_INDEX.md) |
| **Current status snapshot (version, ships, changelogs)** | [PROJECT_CURRENT_STATUS.md](PROJECT_CURRENT_STATUS.md) |
| In-app FAQ (mumu slots, plugins, UI) | [../FAQ.md](../FAQ.md) |
| Error codes & triage | [ERROR_CODES.md](ERROR_CODES.md) · [KERNEL_ERROR_CODE_CONVENTION.md](KERNEL_ERROR_CODE_CONVENTION.md) · A3 [EN](../../handoff/archive/A3_CLOSURE_SUMMARY.en.md) / [ZH](../../handoff/archive/A3_CLOSURE_SUMMARY.md) |
| Local sidecar + BYOK | [SIDECAR_LLM_USER_GUIDE.md](SIDECAR_LLM_USER_GUIDE.md) |
| Config paths | [../guides/CONFIGURATION_FILES.md](../guides/CONFIGURATION_FILES.md) |
| mumu UI acceptance | [`handoff/distros/MUMU_UI_ACCEPTANCE_CHECKLIST.md`](../../handoff/distros/MUMU_UI_ACCEPTANCE_CHECKLIST.md) |
| Editor vs host compatibility | [../COMPATIBILITY.md](../COMPATIBILITY.md) |
| Open-source licensing (host & plugins) | [../LICENSE_POLICY.md](../LICENSE_POLICY.md) |
| Creator workflow | [CREATOR_WORKFLOW.md](CREATOR_WORKFLOW.md) |

### 2. Modules, contracts, architecture

| Topic | Entry |
|-------|--------|
| Kernel-centric diagram | [KERNEL_AND_MODULES_ARCHITECTURE.md](KERNEL_AND_MODULES_ARCHITECTURE.md) |
| `plugin_backends` | [../plugin-and-architecture/PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md) |
| Directory plugins + manager | [../plugin-and-architecture/DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md) |
| Bridge `invoke` | [../plugin-and-architecture/BRIDGE_API_REFERENCE.md](../plugin-and-architecture/BRIDGE_API_REFERENCE.md) |
| Remote JSON-RPC | [../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md](../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md) |
| Extension styles overview | [../plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md](../plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md) |
| Extension points index | [../plugin-and-architecture/EXTENSION_POINTS.md](../plugin-and-architecture/EXTENSION_POINTS.md) |
| Replace modules (builtin / remote / directory) | [../plugin-and-architecture/HOW_TO_REPLACE_MODULES.md](../plugin-and-architecture/HOW_TO_REPLACE_MODULES.md) |
| `memory = local` bridge | [../plugin-and-architecture/LOCAL_PLUGIN_BRIDGE_SPEC.md](../plugin-and-architecture/LOCAL_PLUGIN_BRIDGE_SPEC.md) |
| On-disk pack / RobotSoulPack | [../role-pack/ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md) |

### 3. Kernel, platform, headless, CLI

| Topic | Entry |
|-------|--------|
| Pure kernel boundary | [PURE_KERNEL_BOUNDARY.md](PURE_KERNEL_BOUNDARY.md) |
| K0–K5 + verification | [KERNEL_IMPLEMENTATION_PLAN.md](KERNEL_IMPLEMENTATION_PLAN.md) |
| Single developer path | [KERNEL_PLATFORM_DEVELOPER_PATH.md](KERNEL_PLATFORM_DEVELOPER_PATH.md) |
| Headless `--api` loop | [../../examples/headless-kernel-minimal/README.md](../../examples/headless-kernel-minimal/README.md) |
| CLI & Monolith RFC | [../cli/OCLIVE_CLI_GUIDE.md](../cli/OCLIVE_CLI_GUIDE.md) · [../rfc/RFC_OCLIVE_MONOLITH_MODE.md](../rfc/RFC_OCLIVE_MONOLITH_MODE.md) |
| Settings keys | [../cli/SETTINGS_REFERENCE.md](../cli/SETTINGS_REFERENCE.md) |

### 4. Testing & quality

| Topic | Entry |
|-------|--------|
| Test layers | [../testing/OVERVIEW.md](../testing/OVERVIEW.md) |
| OOCP suite | [../testing/OOCP_TEST_SUITE.md](../testing/OOCP_TEST_SUITE.md) |
| Output schema | [../testing/TEST_OUTPUT_SCHEMA.md](../testing/TEST_OUTPUT_SCHEMA.md) |
| Supply chain / lightweight | [../../creator-docs/development/LIGHTWEIGHT_PROFILE.md](../../creator-docs/development/LIGHTWEIGHT_PROFILE.md) |
| Known CVEs | [../security/KNOWN_VULNERABILITIES.md](../security/KNOWN_VULNERABILITIES.md) |

### 5. Vision & ecosystem

| Topic | Entry |
|-------|--------|
| Open lab | [../../creator-docs/roadmap/VISION_OPEN_LAB.md](../../creator-docs/roadmap/VISION_OPEN_LAB.md) |
| Monthly roadmap | [../../creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md](../../creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md) |
| Experience backlog | [../../creator-docs/roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md](../../creator-docs/roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md) |
| Market / launcher / web | under `../../creator-docs/roadmap/` |

### 6. Handoff (`handoff/`)

| Topic | Entry |
|-------|--------|
| Product execution view | [../../handoff/PRODUCT_LINE_TASK_BUCKETS.md](../../handoff/PRODUCT_LINE_TASK_BUCKETS.md) · [TECHNICAL_DEBT_INVENTORY.md](../../handoff/TECHNICAL_DEBT_INVENTORY.md) |
| Four-repo i18n baseline | [../../handoff/I18N_FOUR_REPO_BASELINE.md](../../handoff/I18N_FOUR_REPO_BASELINE.md) |

---

## Status snapshot (aligned with goals)

- **Kernel K0–K5**: closed in plan except **P2 (OTA / remote logs)** — see [KERNEL_IMPLEMENTATION_PLAN.md](KERNEL_IMPLEMENTATION_PLAN.md) and [AGENTS.md](../../AGENTS.md) for CI.
- **Role pack blueprint v2 (P0–P8)**: `pipeline.ocblueprint` is the recommended SSOT; **`pack validate` defaults to v2**; golden pack `distros/chat-pro/roles/mumu`; architecture graph **`save_role_slot_registry`**. See [BLUEPRINT_V2_IMPLEMENTATION_PLAN.md](../../handoff/BLUEPRINT_V2_IMPLEMENTATION_PLAN.md).
- **Product launch**: current checks are [PROJECT_OVERVIEW.md §8](PROJECT_OVERVIEW.md#8-minimal-pre-release-checklist), the CI workflow, and the active debt inventory. **A1 (CI-attainable slice)** is closed (**A1.1a** HTTP restart, **A1.1b** `vite preview` + Playwright, **A1.2** nine `invoke` hot-path `*_impl` chains). **Default next engineering focus** is **A2.2 / A2.3 / A4.2** and **A1.1c (native installer / Tauri-window E2E)** — see [PRODUCT_LINE_TASK_BUCKETS.md](../../handoff/PRODUCT_LINE_TASK_BUCKETS.md) **§四 (Hard items)** — **one issue per item**. The old gap checklist remains historical context only.
- **Experience**: use monthly vision + backlog; do not merge into the kernel milestone table.
- **Creator docs bilingual (`creator-docs-en/`)**: hub + plugin contracts + `guides/` etc. are aligned with the Chinese corpus for a **closure baseline**; long-form vision under `creator-docs/roadmap/` stays **Chinese-first** until a release needs EN. Update mirrors in the same change-set as contract changes, or note Chinese-only doc updates in CHANGELOG — see [Documentation bilingual closure baseline](../README.md#documentation-bilingual-closure-baseline) in `creator-docs-en/README.md`.

---

## Where “future goals” are defined

| Track | Source of truth |
|-------|-----------------|
| Kernel follow-up (incl. P2) | [KERNEL_IMPLEMENTATION_PLAN.md](KERNEL_IMPLEMENTATION_PLAN.md) |
| Product P0–P2 | [PRODUCT_LINE_TASK_BUCKETS.md](../../handoff/PRODUCT_LINE_TASK_BUCKETS.md) · [TECHNICAL_DEBT_INVENTORY.md](../../handoff/TECHNICAL_DEBT_INVENTORY.md) |
| Vision / narrative | [../../creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md](../../creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md) · [../../creator-docs/roadmap/VISION_OPEN_LAB.md](../../creator-docs/roadmap/VISION_OPEN_LAB.md) |

**Habit**: before release or contract changes, run checks in [PROJECT_OVERVIEW.md](PROJECT_OVERVIEW.md); when changing kernel boundaries, sync [PURE_KERNEL_BOUNDARY.md](PURE_KERNEL_BOUNDARY.md) and the kernel plan.

---

## Index dedupe note

In [DOCUMENTATION_INDEX.md](DOCUMENTATION_INDEX.md), duplicate FAQ rows point to a **single** [../FAQ.md](../FAQ.md) row. Directory plugins + manager shortcuts remain one canonical [../plugin-and-architecture/DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md).
