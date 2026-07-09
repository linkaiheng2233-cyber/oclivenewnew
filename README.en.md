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
| Full index | [DOCUMENTATION_INDEX.md](creator-docs/getting-started/DOCUMENTATION_INDEX.md) |

---

## 🤖 AI / Agent section

> **Humans**: use [human-docs/](human-docs/) first. **Agents**: this section + [AGENTS.md](AGENTS.md)—link to SSOT, do not duplicate long tables.

| Key | Value |
|-----|-------|
| Reply field | **`reply`** (not `response`) |
| Orchestration SSOT | `kernel/crates/oclive_kernel_host/.../process_message.rs` |
| Blueprint `steps[]` | **Not** on hot path |
| Six slots | `memory` · `emotion` · `event` · `prompt` · `llm` · `agent` |
| Side channels | e.g. `voice.asr` — **not** in six slots |

**Before coding**: [AI_CHANGE_BOUNDARIES.md](handoff/AI_CHANGE_BOUNDARIES.md) (G1–G16) · [MODULE_MAP_AND_HANDOFF.md](handoff/MODULE_MAP_AND_HANDOFF.md) · [BUS_FACTOR_NOTES.md](handoff/BUS_FACTOR_NOTES.md)

**Tests**: OOCP S0–S12 · invoke hot path **13** · Dimension 5 **15/14** · daily `npm run check:rust` (no doctest) · release `npm run check:release`

---

*License: Apache-2.0 · [SECURITY.md](SECURITY.md) · [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)*
