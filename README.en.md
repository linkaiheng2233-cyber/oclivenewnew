# A.I.Live — Pluggable Role Artery Loom

> Repository **oclivenewnew** (codename **oclive**) · Open source · Local-first · **Tauri + Vue 3 + Rust**

[中文](README.md)

[![CI](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/workflows/ci.yml/badge.svg)](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/workflows/ci.yml)

**Release**: desktop host **0.5.2** · see [CHANGELOG.en.md](CHANGELOG.en.md)

## Start here

> **New to the project? Read [What is OCLive (5 min)](human-docs/START_HERE.md) first** (Chinese).
> That page covers what it is, who it is for, what it is not, and which path to take.

| Reader | Entry |
|---|---|
| **First time here** | **[What is OCLive (5 min)](human-docs/START_HERE.md)** |
| Human contributor | [Learning path](human-docs-en/README.md) |
| Rust Base module author | [Kernel foundation setup](creator-docs-en/cli/OCLIVE_CLI_GUIDE.md#kernel-base-start) → [Memory example](human-docs-en/modules/slots/memory.md) |
| Role / plugin author or integrator | [Documentation index](creator-docs-en/getting-started/DOCUMENTATION_INDEX.md) |
| AI agent / maintainer | [AGENTS](AGENTS.md), then the task-specific [AI index](handoff/AI_READING_INDEX.md) |
| Checking document authority or status | [Documentation layers and status](handoff/README.md#documentation-status) |

---

## What is this?

**A.I.Live (OCLive)** is not “yet another fixed AI chat app,” and a centralized platform is not its essence. It is first an **embeddable tool kernel that organizes AI-character capabilities through stable contracts**:

- The small Kernel defines six-slot capability contracts, necessary causality, and legality/error boundaries; distro Hosts own composition, scheduling, and domain application ([responsibility SSOT](handoff/MODULE_MAP_AND_HANDOFF.md#kernel-responsibilities))
- **Six stable capability ports**—memory, emotion, legacy event impact, prompt, LLM, and agent—accept replaceable implementations
- **Role packs** (persona, scenes, prompts) ship independently
- **Local-first** by default; cloud APIs optional (BYOK)

This repository also ships a complete reference runtime: concrete slot implementations, SQLite, Event Ring, facilities, HTTP/Tauri, and Chat Pro. Built-in roles such as `mumu` are examples. The runtime, distros, editor, and market can use the kernel, but none defines the minimal core.

> **One line**: OCLive = **one tool kernel + six stable capability ports + optional reference assemblies**. Use the whole runtime or take only the interfaces, contracts, and modules you need. “cargo + docker-compose for characters” is an assembly analogy, not platform lock-in.
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

Reference Hosts, the editor, and CLI reuse loading/validation capabilities, subject to each entry's supported formats. `manifest.json` / `pipeline.ocblueprint` are not mandatory files for every minimal role. The minimal definition expresses persona + visual asset references; local loading preparation does not mean runtime activation is wired ([role-pack boundary](handoff/ROLE_PACK_BOUNDARY.md)). Sharing **`OCLIVE_ROLES_DIR`** / **`app.db`** between desktop and VS Code is a reference-Host continuity arrangement, not an implemented independent Portable Memory contract: [CROSS_HOST_MEMORY.md](creator-docs-en/role-pack/CROSS_HOST_MEMORY.md).

### Example 4 · Module author: new capability only

Fork `examples/directory-plugin-minimal` or `examples/voice-loop-minimal`, implement **one slot** or a **side channel** (e.g. TTS), and reuse the reference host/toolchain for persona, UI, chat loop, validation, and packaging. Directory plugins declaring `process:spawn` / `network:*` / `mcp:*` require **explicit user consent** before execution (otherwise degrade, no silent escalation).

Start: [PLUGIN_AUTHOR_LEARNING_PATH.md](creator-docs-en/plugin-and-architecture/PLUGIN_AUTHOR_LEARNING_PATH.md) · permissions: [PLUGIN_V1.md](creator-docs-en/plugin-and-architecture/PLUGIN_V1.md)

---

## How is this different?

| | LangChain / AI SDK | EchoVessel / vertical engines | **OCLive** |
|--|-------------------|-------------------------------|------------|
| What you get | Blocks + glue — **write code** | A **finished dish** — fixed memory/affect | **Standard kitchen + plating rules** — **assemble and pack** your engine |
| Swappable modules | Yes, no role-domain contract | Mostly fixed | **Six slots + builtin/remote/directory** unified contract |
| Role content distribution | DIY | Bound to product | **`.ocpak` / zip**, editor export, deep-link install |
| Ceiling | Your code | Vendor implementation | **Not welded to one builtin implementation**; swap modules and models by contract |

**SillyTavern**-class “frontend shell + many backends” (common question):

| | SillyTavern class | **OCLive** |
|--|-------------------|------------|
| Core deliverable | Chat UI + API/extensions | **Six-slot contract + blueprint + pack format + cross-host validation** |
| Module semantics | Extensions ad hoc | **builtin / remote / directory** unified backend surface |
| Distribution | Community cards/files | **`.ocpak` / zip · SHA-256 · `oclive://` deep links** + market site |
| Orchestration SSOT | Often frontend/extensions | Rust **`process_message`** in the current reference Host, not a universal Host pipeline |

---

## Three distros (one kernel · different HostProfile)

Current reference distros reuse **`process_message`** and **`distro.oclive.toml` HostProfile**. This is a reference-Host maintenance boundary, not a requirement for third-party Hosts to use the same orchestration. Host scheduling must still satisfy capability contracts and necessary causality.

| Distro | `distro_id` | Shape | Status |
|--------|-------------|-------|--------|
| **A.I.Live Chat Pro** | `desktop` | This repo Tauri desktop (Release hero) | **0.5.2** main path |
| **VS Code Flash** | `vscode` | Sister repo [oclive-vscode](https://github.com/linkaiheng2233-cyber/oclive-vscode) | Penetration **pluginized**; core = lightweight chat host |
| **AI Theater** | `theater` | `distros/theater/` + theater profile | Bundled; mode 2 playtest **unfrozen** |
| **dev lab** | `desktop-chat` | Experimental profile | Daily dev / low-latency trials |

Profile SSOT: [THREE_DISTRO_KERNEL_CLOSURE.md](handoff/THREE_DISTRO_KERNEL_CLOSURE.md) · [DISTRO_CAPABILITY_PROFILE.md](creator-docs-en/kernel/DISTRO_CAPABILITY_PROFILE.md)

---


## Architecture (one-paragraph version)

OCLive optimizes for **orthogonal layers**, not for stuffing every capability into one app: swapping the LLM does not touch the persona, adding voice does not pollute the main chain, and freezing experimental features does not affect Stable releases.

The **kernel** specifies the shape of six capability ports plus a small set of baselines every Host must honour (how failures and errors are expressed, which combinations are illegal); the **Host** decides how to wire them into one turn; the **distro** decides what is installed and enabled by default. The six ports are `memory` / `emotion` / `event` / `prompt` / `llm` / `agent` — **all optional capabilities, not a fixed six-step pipeline**.

| For deeper reading | Read |
|---|---|
| How the four layers are cut, and why | [What is OCLive (5 min)](human-docs/START_HERE.md) (Chinese) |
| How one turn runs in code | [human-docs/01 simple architecture](human-docs/01_ARCHITECTURE_SIMPLE.md) (Chinese) |
| External architecture narrative, module numbering, facilities | [OCLIVE_ARCHITECTURE_OVERVIEW](creator-docs/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md) |
| **Module definitions, slot relations, change constraints (single SSOT)** | [MODULE_MAP](handoff/MODULE_MAP_AND_HANDOFF.md) |
| Role content / blueprint / distro separation | [ROLE_PACK_BOUNDARY](handoff/ROLE_PACK_BOUNDARY.md) |
| Single-kernel dual-mode (Monolith) | [RFC_OCLIVE_MONOLITH_MODE](creator-docs/rfc/RFC_OCLIVE_MONOLITH_MODE.md) |
| Distribution mechanics (OOCP, signing, `oclive://` deep links, cross-host memory, kernel factory) | [OCLIVE_ARCHITECTURE_OVERVIEW](creator-docs/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md) · [CROSS_HOST_MEMORY](creator-docs/role-pack/CROSS_HOST_MEMORY.md) |
| Roadmap and vision | [VISION_ROADMAP_MONTHLY](creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md) · [VISION_OPEN_LAB](creator-docs/roadmap/VISION_OPEN_LAB.md) |
| Current progress and outstanding work | [PROJECT_CURRENT_STATUS](creator-docs/getting-started/PROJECT_CURRENT_STATUS.md) · [TECHNICAL_DEBT_INVENTORY](handoff/TECHNICAL_DEBT_INVENTORY.md) |

**The front page carries positioning and entry points only; it does not restate architecture tables** (G14: link instead of copy). Module definitions live only in MODULE_MAP, public contracts only in creator-docs, progress only in the debt ledger.

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
| Node.js 22+, Rust stable | Windows also needs **VS Build Tools (MSVC)** |
| Ollama | **Optional** for build; needed for local LLM chat |
| Cargo artifacts | Default outside repo: `../oclive-dev-artifacts/oclivenewnew-cargo-target/` |

**Installers**: GitHub [Releases](https://github.com/linkaiheng2233-cyber/oclivenewnew/releases) provides the **A.I.Live Chat Pro 0.5.0 beta** Windows x64 MSI / NSIS installers (unsigned and without automatic updates). The current main branch is **0.5.2** and may be ahead of the published installer; clone the repo and follow [CONTRIBUTING.en.md](CONTRIBUTING.en.md) when you need the latest source build.

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

## AI / Agent onboarding

**This homepage is for humans.** Cursor, Codex, and other agents should use the dedicated reading index:

| Doc | Purpose |
|-----|---------|
| **[handoff/AI_READING_INDEX.md](handoff/AI_READING_INDEX.md)** | **Categorized SSOT index** (architecture · contracts · code anchors · task paths) |
| [AGENTS.md](AGENTS.md) | **Quick gate** before editing code (task-specific change boundaries) |
| [human-docs/ai-package/README.md](human-docs/ai-package/README.md) | AI package layout vs human docs |

---

## Support · license

- **Issues**: [GitHub Issues](https://github.com/linkaiheng2233-cyber/oclivenewnew/issues) (prefix `[bug]` / `[feat]` / `[support]`)
- **License**: Apache-2.0 · [LICENSE](LICENSE) · [LICENSE_POLICY.md](creator-docs-en/LICENSE_POLICY.md)
- **Code of conduct**: [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) · **Security**: [SECURITY.md](SECURITY.md)

---

*Human learning ladder: [human-docs-en/README.md](human-docs-en/README.md)*