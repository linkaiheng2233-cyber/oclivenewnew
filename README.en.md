# A.I.Live — Pluggable Role Artery Loom

> Repository **oclivenewnew** (codename **oclive**) · Open source · Local-first · **Tauri + Vue 3 + Rust**

[中文](README.md)

[![CI](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/workflows/ci.yml/badge.svg)](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/workflows/ci.yml)

**Release**: desktop host **0.4.0** · see [CHANGELOG.en.md](CHANGELOG.en.md)

---

## What is this?

**A.I.Live (OCLive)** is an **assemble–contract–pack–distribute** platform for AI characters and agents—not a fixed vertical chat app:

- **Six swappable slots** (memory, emotion, event, prompt, LLM, agent) compose your role runtime
- **Role packs** (persona, scenes, prompts) ship independently
- **Local-first** by default; cloud APIs optional (BYOK)

Built-in roles (e.g. `distros/chat-pro/roles/mumu`) are **official examples**. Community packs and module ecosystems define the ceiling.

Positioning: [handoff/OCLIVE_POSITIONING_DIFFERENTIATION.md](handoff/OCLIVE_POSITIONING_DIFFERENTIATION.md)

---

## Three quick examples

| Scenario | What you do |
|----------|-------------|
| **Creator** | Use [oclive-pack-editor](https://github.com/linkaiheng2233-cyber/oclive-pack-editor) → write `prompts/system.md` → `npm run tauri:dev` |
| **Developer** | Swap **slot 5 (llm)** in `pipeline.ocblueprint` from Ollama to remote/directory—persona unchanged |
| **Integrator** | Same role pack validated by desktop, headless `--api`, editor WASM, and `oclive-cli` |

30-minute creator path: [CREATOR_GOLDEN_PATH.md](creator-docs/getting-started/CREATOR_GOLDEN_PATH.md) · Human ladder: [human-docs-en/README.md](human-docs-en/README.md)

---

## Why this is hard to clone as “just another chat UI”

Cross-cutting engineering, not a single feature:

- **`oclive_validation`** — same contract in runtime, editor WASM, CLI
- **`process_message` + PluginHost** — stable turn semantics; swap backends, not orchestration
- **Three memory stores** — chat log vs STM vs LTM (deleting chat ≠ wiping AI memory)
- **G1–G16 boundaries + CI** — OOCP S0–S12, Dimension 5 (15 registered / 14 in CI), layering ratchet
- **Role pack vs blueprint split** — creators don’t touch `slot_registry`

Details: [OCLIVE_ARCHITECTURE_OVERVIEW.md](creator-docs/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md) · [MODULE_MAP_AND_HANDOFF.md](handoff/MODULE_MAP_AND_HANDOFF.md)

---

## Architecture (why assembly stays sane)

Design goal: **orthogonal layers**—swap LLM without touching persona; add voice without polluting the main chain; freeze experimental cores without blocking Stable releases.

| Category | In six `plugin_backends` keys? | Examples |
|----------|----------------------------------|----------|
| **Backend modules (slots 1–6)** | **Yes** | memory · emotion · event · prompt · llm · agent |
| **Facility submodules** | **No** (in orchestration) | complex emotion · expert routing · portrait · visual stage |
| **Side channels** | **No** (own resolver) | user identity · reply post-process · **voice.asr** |
| **Backend plugins** | Attached to a slot | directory LLM plugin · remote sidecar |

**Six-slot decoupling**: compile-time traits + `PluginHost` (fixed `process_message` order) · config-time `slot_registry` multi-instance fold · runtime session override (not persisted).

**Orthogonal config layers**: role pack (creator content) → blueprint (`slot_registry`, **`steps[]` not on hot path**) → distro HostProfile → session DB.

**Single kernel, dual build modes**: **outer core** (default PluginHost, swappable backends) vs **Monolith macro core** (compile-time weld via `monolith.toml` for embedded/perf). Not runtime hot-switch.

**Experimental core (`dual_core`)**: mechanism wired, **default off**—`dual_core` Cargo feature + blueprint opt-in; expert routing frozen per [TECHNICAL_DEBT_INVENTORY.md §2](handoff/TECHNICAL_DEBT_INVENTORY.md).

Human 45-min guide: [human-docs-en/01_ARCHITECTURE_SIMPLE.md](human-docs-en/01_ARCHITECTURE_SIMPLE.md) · [human-docs/01_ARCHITECTURE_SIMPLE.md](human-docs/01_ARCHITECTURE_SIMPLE.md)

---

## 30-minute contributor path

```bash
npm install
npm run tauri:dev
npm run check
```

Windows needs **VS Build Tools (MSVC)**. Ollama is optional for build. Step-by-step: [human-docs-en/02_THIRTY_MINUTE_START.md](human-docs-en/02_THIRTY_MINUTE_START.md)

---

## Documentation by role

| Role | Start here |
|------|------------|
| End user | [USER_MANUAL.md](creator-docs-en/getting-started/USER_MANUAL.md) |
| Human developer | [human-docs-en/README.md](human-docs-en/README.md) |
| Contributors | [CONTRIBUTING.en.md](CONTRIBUTING.en.md) |
| Full index (ZH SSOT) | [DOCUMENTATION_INDEX.md](creator-docs/getting-started/DOCUMENTATION_INDEX.md) |
| English contract mirror | [creator-docs-en/README.md](creator-docs-en/README.md) |

### English mirror policy

**Chinese (`creator-docs/`, `human-docs/`) is the single source of truth** for normative contracts and role-pack wording. English trees (`creator-docs-en/`, `human-docs-en/`) are **hand-maintained mirrors** filled in by phase — not a second SSOT.

| Tree | Role | When English is missing |
|------|------|-------------------------|
| [creator-docs-en/](creator-docs-en/) | Contracts, kernel, plugin, role-pack, testing | Follow `[中文](…)` links on each page, or the [coverage matrix](creator-docs-en/README.md#mirror-coverage-matrix) |
| [human-docs-en/](human-docs-en/) | Human learning ladder L0–L8 | Fall back to [human-docs/README.md](human-docs/README.md) |

**Maintenance**: when you change a Chinese page that already has an English mirror, update the mirror in the **same change-set** (or note Chinese-only in CHANGELOG). See [creator-docs-en/README.md § Sync rules](creator-docs-en/README.md#sync-rules).

---

## AI / Agent onboarding

**This homepage is for humans.** Cursor, Codex, and other agents should use the dedicated reading index:

| Doc | Purpose |
|-----|---------|
| **[handoff/AI_READING_INDEX.md](handoff/AI_READING_INDEX.md)** | **Categorized SSOT index** (architecture · contracts · code anchors · task paths) |
| [AGENTS.md](AGENTS.md) | **Quick gate** before editing code (G1–G16 summary) |
| [human-docs/ai-package/README.md](human-docs/ai-package/README.md) | AI package layout vs human docs |

---

*License: Apache-2.0 · [SECURITY.md](SECURITY.md) · [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)*
