# A.I.Live — Pluggable Role Artery Loom

> Repository **oclivenewnew** (codename **oclive**) · Open source · Local-first · **Tauri + Vue 3 + Rust**

[中文](README.md)

[![CI](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/workflows/ci.yml/badge.svg)](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/workflows/ci.yml)

**Release**: desktop host **0.5.0** · see [CHANGELOG.en.md](CHANGELOG.en.md)

---

## What is this?

**A.I.Live (OCLive)** is not “yet another fixed AI chat app.” It is an **assemble–contract–pack–distribute** platform for AI characters and agents:

- **Six swappable slots** (memory, emotion, event, prompt, LLM, agent) compose your role runtime
- **Role packs** (persona, scenes, prompts) ship independently
- **Local-first** by default; cloud APIs optional (BYOK)

Built-in roles (e.g. `distros/chat-pro/roles/mumu`) are **official examples**. Community packs and module ecosystems define the ceiling.

> **One line**: OCLive = **cargo + docker-compose** for AI characters — an open, local-first thin kernel with swappable, validatable, packagable modules. Assemble and distribute your own role runtime in ~30 minutes. **Ceiling = module ecosystem ceiling** (competitors’ strengths can become a slot backend).
>
> Deep positioning: [handoff/OCLIVE_POSITIONING_DIFFERENTIATION.md](handoff/OCLIVE_POSITIONING_DIFFERENTIATION.md)

---

## Four quick examples (30 seconds)

### Example 1 · Creator: a talkable OC

1. Clone [oclive-pack-editor](https://github.com/linkaiheng2233-cyber/oclive-pack-editor) (role pack editor)
2. New pack: write `prompts/system.md`, save under `distros/chat-pro/roles/your_role_id/`
3. In this repo: `npm run tauri:dev` → pick role → chat

**Do not touch** blueprint `slot_registry` or six slots on this path. 30-minute guide: [CREATOR_GOLDEN_PATH.md](creator-docs-en/getting-started/CREATOR_GOLDEN_PATH.md).

### Example 2 · Developer: swap LLM only

In `pipeline.ocblueprint`, change **slot 5 (llm)** from `ollama` to `remote` or a **directory plugin** — persona, memory, and prompt formula stay the same. Ollama, llama.cpp sidecars, and OpenAI-compatible APIs are **different plugs on the same slot**.

### Example 3 · Integrator: one pack, many hosts

The same `manifest.json` + `pipeline.ocblueprint` is validated by **desktop Tauri**, **headless HTTP `--api`**, **editor WASM**, and **oclive-cli** — format SSOT lives in `oclive_validation`, not in one app. Desktop and VS Code can share **`OCLIVE_ROLES_DIR`** and **`app.db`** (L1 pack + L3 continuity): [CROSS_HOST_MEMORY.md](creator-docs-en/role-pack/CROSS_HOST_MEMORY.md).

### Example 4 · Module author: new capability only

Fork `examples/directory-plugin-minimal` or `examples/voice-loop-minimal`, implement **one slot** or a **side channel** (e.g. TTS) — persona, UI, chat loop, validation, and packaging come from the platform. Directory plugins declaring `process:spawn` / `network:*` / `mcp:*` require **explicit user consent** before execution (otherwise degrade, no silent escalation).

Start: [PLUGIN_AUTHOR_LEARNING_PATH.md](creator-docs-en/plugin-and-architecture/PLUGIN_AUTHOR_LEARNING_PATH.md) · permissions: [PLUGIN_V1.md](creator-docs-en/plugin-and-architecture/PLUGIN_V1.md)

---

## How is this different?

| | LangChain / AI SDK | EchoVessel / vertical engines | **OCLive** |
|--|-------------------|-------------------------------|------------|
| What you get | Blocks + glue — **write code** | A **finished dish** — fixed memory/affect | **Standard kitchen + plating rules** — **assemble and pack** your engine |
| Swappable modules | Yes, no role-domain contract | Mostly fixed | **Six slots + builtin/remote/directory** unified contract |
| Role content distribution | DIY | Bound to product | **`.ocpak` / zip**, editor export, deep-link install |
| Ceiling | Your code | Vendor implementation | **Union of module ecosystem** |

**SillyTavern**-class “frontend shell + many backends” (common question):

| | SillyTavern class | **OCLive** |
|--|-------------------|------------|
| Core deliverable | Chat UI + API/extensions | **Six-slot contract + blueprint + pack format + cross-host validation** |
| Module semantics | Extensions ad hoc | **builtin / remote / directory** unified backend surface |
| Distribution | Community cards/files | **`.ocpak` / zip · SHA-256 · `oclive://` deep links** + market site |
| Orchestration SSOT | Often frontend/extensions | Rust **`process_message`** fixed turn semantics |

---

## Three distros (one kernel · different HostProfile)

Orchestration is **one** (`process_message`); differences are **`distro.oclive.toml` HostProfile** and host UI — **not** a second chat engine.

| Distro | `distro_id` | Shape | Status |
|--------|-------------|-------|--------|
| **A.I.Live Chat Pro** | `desktop` | This repo Tauri desktop (Release hero) | **0.5.0** main path |
| **VS Code Flash** | `vscode` | Sister repo [oclive-vscode](https://github.com/linkaiheng2233-cyber/oclive-vscode) | Penetration **pluginized**; core = chat platform |
| **AI Theater** | `theater` | `distros/theater/` + theater profile | Bundled; mode 2 playtest **unfrozen** |
| **dev lab** | `desktop-chat` | Experimental profile | Daily dev / low-latency trials |

Profile SSOT: [THREE_DISTRO_KERNEL_CLOSURE.md](handoff/THREE_DISTRO_KERNEL_CLOSURE.md) · [DISTRO_CAPABILITY_PROFILE.md](creator-docs-en/kernel/DISTRO_CAPABILITY_PROFILE.md)

---

## Why this is hard to clone as “just another chat UI”

Cross-cutting engineering, not a single feature:

| Asset | Meaning |
|-------|---------|
| **`oclive_validation`** | Same contract in runtime, editor WASM, CLI — no silent format drift |
| **`process_message` + PluginHost** | Stable turn semantics; swap backends, not orchestration |
| **Three memory stores** | Chat log / STM / LTM decoupled (deleting chat ≠ wiping AI memory) |
| **G1–G16 + CI gates** | OOCP S0–S12, Dimension 5 **15** registered / **14** in CI, layering ratchet, doc registry |
| **Role pack vs blueprint split** | Creators don’t touch `slot_registry`; admins don’t pollute content packs |
| **Side channels (e.g. voice.asr)** | Voice/TTS **outside six slots** — does not pollute `process_message` |

Details: [OCLIVE_ARCHITECTURE_OVERVIEW.md](creator-docs-en/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md) · [MODULE_MAP_AND_HANDOFF.md](handoff/MODULE_MAP_AND_HANDOFF.md)

---

## Architecture (why assembly stays sane)

Design goal: **orthogonal layers** — swap LLM without touching persona; add voice without polluting the main chain; freeze experimental cores without blocking Stable releases.

### Four module categories (map first)

| Category | In six `plugin_backends` keys? | Examples |
|----------|----------------------------------|----------|
| **Backend modules (slots 1–6)** | **Yes** | memory · emotion · event · prompt · llm · agent |
| **Facility submodules** | **No** (in orchestration) | complex emotion · expert routing · portrait · visual stage |
| **Side channels** | **No** (own resolver) | user identity · reply post-process · **voice.asr** · theater director API |
| **Backend plugins** | Attached to a slot | directory LLM plugin · remote sidecar |

**Discipline**: plugins do **not** get a “slot 7” number; facilities do **not** become six-slot keys. Per-slot definitions → [MODULE_MAP §2–§10](handoff/MODULE_MAP_AND_HANDOFF.md).

**Six-slot decoupling**: compile-time traits + `PluginHost` (fixed `process_message` order) · config-time `slot_registry` multi-instance fold · runtime session override (not persisted).

**Orthogonal config layers**: role pack (creator content) → blueprint (`slot_registry`, **`steps[]` not on hot path**) → distro HostProfile → session DB.

**Single kernel, dual build modes**: **outer core** (default PluginHost, swappable backends) vs **Monolith macro core** (compile-time weld via `monolith.toml` for embedded/perf). Not runtime hot-switch.

**Experimental core (`dual_core`)**: mechanism wired, **default off** — `dual_core` Cargo feature + blueprint opt-in; expert routing frozen per [TECHNICAL_DEBT_INVENTORY.md §2](handoff/TECHNICAL_DEBT_INVENTORY.md).

Human 45-min guide: [human-docs-en/01_ARCHITECTURE_SIMPLE.md](human-docs-en/01_ARCHITECTURE_SIMPLE.md)

---

## Distribution · contracts · cross-host

| Mechanism | Human-readable | Integration anchor |
|-----------|----------------|-------------------|
| **OOCP black-box** | Swap backend, same turn semantics | S0–S12 · `examples/oocp-test-suite/` |
| **Pack signing** | Editor export zip / `.ocpak`, optional SHA-256 sidecar | `api/plugin_pack.rs` |
| **Deep-link install** | Market → `oclive://` → host installs plugin/pack | [oclive-plugin-market](https://github.com/linkaiheng2233-cyber/oclive-plugin-market) |
| **Cross-host memory** | Same roles dir + shared `app.db` → desktop ↔ VS Code continuity | L1/L2/L3 · [CROSS_HOST_MEMORY.md](creator-docs-en/role-pack/CROSS_HOST_MEMORY.md) |
| **Kernel factory** | `oclive-cli init` → standalone `cargo build` skeleton | `kernel/crates/oclive-cli` |

---

## Roadmap · open lab

Product axis: **local-first, swappable modules, role pack as the only integration surface** — an **open experiment harness**. Researchers/developers **write new modules**, plug into a slot, and try them in a full role; persona, storage, UI, and turn loop come from the platform.

| Phase | Highlights |
|-------|------------|
| **Shipped** | Six slots + directory/Remote plugins · OOCP · three distro profiles · streaming `/chat/stream` · Turn Thinking Fast/Deep |
| **In progress / wired** | Plugin market + launcher · Theater mode 2 · portrait/visual facility RFCs |
| **Default off (not “shipping soon”)** | `dual_core` experimental core · expert routing · blueprint v3 |

Vision: [VISION_OPEN_LAB.md](creator-docs-en/roadmap/VISION_OPEN_LAB.md) · monthly: [VISION_ROADMAP_MONTHLY.md](creator-docs-en/roadmap/VISION_ROADMAP_MONTHLY.md)

---

## 30-minute contributor path

```bash
git clone https://github.com/linkaiheng2233-cyber/oclivenewnew.git
cd oclivenewnew
npm install
npm run tauri:dev    # desktop client
npm run check        # daily gates (build + fmt + clippy + test --lib)
```

| Prerequisite | Notes |
|--------------|-------|
| Node.js 18+, Rust stable | Windows also needs **VS Build Tools (MSVC)** |
| Ollama | **Optional** for build; needed for local LLM chat |
| Cargo artifacts | Default outside repo: `../oclive-dev-artifacts/oclivenewnew-cargo-target/` |

**Installers**: GitHub [Releases](https://github.com/linkaiheng2233-cyber/oclivenewnew/releases) currently emphasize **role packs**; desktop client requires **clone + local build** (`npm run tauri:dev` / release flow in [CONTRIBUTING.en.md](CONTRIBUTING.en.md)).

Step-by-step: [human-docs-en/02_THIRTY_MINUTE_START.md](human-docs-en/02_THIRTY_MINUTE_START.md)

---

## Ecosystem (sister repos)

| Repo | Purpose |
|------|---------|
| **This repo** | Desktop runtime, Rust kernel, Chat Pro / Theater distros |
| [oclive-pack-editor](https://github.com/linkaiheng2233-cyber/oclive-pack-editor) | Visual role pack editor · export zip / `.ocpak` |
| [oclive-vscode](https://github.com/linkaiheng2233-cyber/oclive-vscode) | In-editor companion (penetration pluginized) |
| [oclive-launcher](https://github.com/linkaiheng2233-cyber/oclive-launcher) | Multi-distro entry · market integration (roadmap) |
| [oclive-plugin-market](https://github.com/linkaiheng2233-cyber/oclive-plugin-market) | Plugin/pack discovery · **`oclive://` deep links** |

---

## Documentation by role

| Who you are | Start here |
|-------------|------------|
| **End user** (install → import pack → chat) | [USER_MANUAL.md](creator-docs-en/getting-started/USER_MANUAL.md) |
| **Human developer** (no Cursor) | **[human-docs-en/README.md](human-docs-en/README.md)** · L0–L2 ~1 hour |
| **Role pack creator** | [CREATOR_LEARNING_PATH.md](creator-docs-en/role-pack/CREATOR_LEARNING_PATH.md) |
| **Plugin / module author** | [PLUGIN_AUTHOR_LEARNING_PATH.md](creator-docs-en/plugin-and-architecture/PLUGIN_AUTHOR_LEARNING_PATH.md) |
| **Kernel integrator** | [KERNEL_INTEGRATOR_LEARNING_PATH.md](creator-docs-en/getting-started/KERNEL_INTEGRATOR_LEARNING_PATH.md) |
| **Code contributor** | [CONTRIBUTING.en.md](CONTRIBUTING.en.md) · [Good first issues](https://github.com/linkaiheng2233-cyber/oclivenewnew/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22) |

Full index: [DOCUMENTATION_INDEX.md](creator-docs-en/getting-started/DOCUMENTATION_INDEX.md)

### English mirror policy

**Chinese (`creator-docs/`, `human-docs/`) is SSOT** for normative contracts. English trees are **hand-maintained mirrors** — not a second SSOT.

| Tree | Role | When English is missing |
|------|------|-------------------------|
| [creator-docs-en/](creator-docs-en/) | Contracts, kernel, plugin, role-pack, testing | Follow `[中文](…)` links or [coverage matrix](creator-docs-en/README.md#mirror-coverage-matrix) |
| [human-docs-en/](human-docs-en/) | Human learning ladder L0–L8 | Fall back to [human-docs/README.md](human-docs/README.md) |

**Maintenance**: update the English mirror in the **same change-set** when you change a mirrored Chinese page. See [creator-docs-en/README.md § Sync rules](creator-docs-en/README.md#sync-rules).

---

## AI / Agent onboarding

**This homepage is for humans.** Cursor, Codex, and other agents should use the dedicated reading index:

| Doc | Purpose |
|-----|---------|
| **[handoff/AI_READING_INDEX.md](handoff/AI_READING_INDEX.md)** | **Categorized SSOT index** (architecture · contracts · code anchors · task paths) |
| [AGENTS.md](AGENTS.md) | **Quick gate** before editing code (G1–G16 summary) |
| [human-docs/ai-package/README.md](human-docs/ai-package/README.md) | AI package layout vs human docs |

---

## Support · license

- **Issues**: [GitHub Issues](https://github.com/linkaiheng2233-cyber/oclivenewnew/issues) (prefix `[bug]` / `[feat]` / `[support]`)
- **License**: Apache-2.0 · [LICENSE](LICENSE) · [LICENSE_POLICY.md](creator-docs-en/LICENSE_POLICY.md)
- **Code of conduct**: [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) · **Security**: [SECURITY.md](SECURITY.md)

---

*Human learning ladder: [human-docs-en/README.md](human-docs-en/README.md)*
