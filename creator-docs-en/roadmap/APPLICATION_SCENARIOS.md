# Application Scenario Matrix (Tool-Kernel Assembly Hypotheses)

This document explores product forms that OCLive's minimal tool kernel—one orchestration/authority boundary plus six stable capability ports—and its reference runtime may support. The goal is to state which contracts can be reused and which adapters and evidence are still missing. Roadmap ideas are not shipped claims; role packs are portable assets, not kernel code.

This page expands open experimentation as one optional use of the kernel. It aligns with [VISION_ROADMAP_MONTHLY.md](VISION_ROADMAP_MONTHLY.md) without binding to delivery dates.

[中文](../../creator-docs/roadmap/APPLICATION_SCENARIOS.md)

---

## Core Principle

```
Stable kernel contracts + Scenario-specific capability assembly + Role assets + Host adapters = Different tools
```

| What changes | What stays the same |
|-------------|---------------------|
| Surface form (desktop / VS Code / speaker / mobile / home panel) | Turn/lifecycle and authoritative state boundary |
| Persona and content | Semantics of the six stable capability ports |
| Module thickness (small-model assistance vs thin strong-model assembly) | Distinction among facts, candidates, and committed results |
| Protocols and external tools | Do not duplicate a second `process_message` authority path |

Not every scenario needs all six concrete implementations, complex emotion, or Event Ring. The current co-present health gate requires at least `prompt + llm`; other capabilities are selected by scenario. “Same kernel” first means the same stable ports and authority boundary, not that today's entire host crate already runs unchanged on every device.

---

## Scenarios at a Glance

| # | Scenario | Form | Key Kernel Capabilities |
|---|----------|------|------------------------|
| S1 | **Coding Companion** | VS Code distribution (official shell) | Host supplies opt-in editor context; prompt/LLM generate the role reply; proactive observation needs a separate authorization path |
| S2 | **Character Casino** | Multi-character game desktop | CoPresent multi-character / agent slot game rules engine / memory for game history |
| S3 | **AI Theatre** | Multi-character story generator | scene mode / complex-emotion candidates / a separate future multi-character scheduler; blueprint `groups` only groups instances |
| S4 | **All-Day Embedded Companion** | Smart speaker / robot / wearable | Resource-aware slot profile + device adapters + `--template robot-soul` |
| S5 | **Desktop Character Chat** (existing) | Tauri desktop app | Standard 1v1 CoPresent |
| S6 | **Headless HTTP API** (existing) | Server deployment | `--api` + `POST /chat` + OOCP |
| S7 | **Mobile Companion** | Mobile app (Tauri mobile / PWA) | Same as S5, different shell |
| S8 | **Smart Home Hub** | Gateway / home hub | `--template robot-gateway` + MCP home toolchain |
| S9 | **Robot Role Runtime** | Embedded in robot OS | Rust `OcliveKernel` facade / typed external observations / agent outputs |
| S10 | **AI NPC Engine** | Game NPC dialogue backend | CoPresent + scene mode + low-latency Monolith build |

---

## Detailed Scenarios

### S1 · Coding Companion (VS Code Distribution)

**Surface**: VS Code extension with personality sidebar chat panel.

**Personality examples**:
- **Teasing character (little sister type)**: Detects bad code → mocking tone (\"Pathetic~ you wrote *that* function?\"); detects good code → tsundere acknowledgment (\"Hmph, you can write something decent once in a while\")
- **Gentle big-sister type**: Coding > 2 hours → urges rest (\"Rest your eyes, I'll get you some water\"); frequent errors → encouraging tone (\"Don't rush, let's look at this bug together\")

**How the kernel does it**:

| Kernel capability | What it does in this scenario |
|-------------------|------------------------------|
| **Host context adapter** | Reads bounded file/language/selection/diagnostic facts only after user opt-in; it does not impersonate user emotion |
| **Slot 2 (emotion)** | Provides candidates from user text; it is not the general classifier for editor behavior |
| **Slot 4 (prompt)** | Injects admitted editor facts, persona material, and candidate hints |
| **Slot 6 (agent)** | Optional: autonomously suggest refactoring, auto-run tests, operate terminal |

**New adapter layer needed**:
- One MCP server (stdio) connecting to VS Code Extension API (read editor state, line numbers, diagnostics)
- The VS Code extension itself as surface UI (chat panel + notifications)

**Does NOT need changes to**: `process_message`, PluginHost, role pack format, any of the six slot implementations.

---

### S2 · Character Casino (Multi-Character Game)

**Surface**: Multiplayer desktop game, player character + AI characters interacting at the same table.

**Example**: Mai Sakurajima × Socrates playing Liar's Bar.

**How the kernel does it**:

| Kernel capability | What it does in this scenario |
|-------------------|------------------------------|
| **CoPresent multi-character** | Orchestrates multiple character turns within the same scene |
| **Slot 6 (agent)** | Connects to game rules engine (play card, call bluff), tool-calling to manipulate game state |
| **Slot 1 (memory)** | Records game history (who played what, who won which rounds), influences future strategy |
| **Slot 3 (event)** | Game events (got called out, won a round) → impact estimation → triggers personality reactions |
| **Complex emotion facility** | Narrative hints for tension/bluffing in-game, injected via prompt into character reactions |

**New adapter layer needed**:
- Game engine (directory plugin or MCP server) managing table state, rule adjudication
- Frontend game UI (card table, hand cards, character avatars)

---

### S3 · AI Theatre (Multi-Character Story)

**Surface**: Multi-character scene performance. Turn-taking requires a separate multi-character host/scheduler; blueprint `groups` only organizes same-type slot instances for the architecture view and is not a speaking-order DSL.

**Example**: Three characters meet in a tavern; the complex-emotion facility offers traceable state/narrative candidates, while the model and multi-character host still decide expression and turn order.

**How the kernel does it**:

| Kernel capability | What it does in this scenario |
|-------------------|------------------------------|
| **scene mode** | Defines tavern scene, time, character list |
| **blueprint `groups`** | Groups same-type slot instances for architecture/management UI only; it defines neither character relationships nor execution order |
| **Character relations / multi-character scheduling** | Role-pack relation data describes who knows whom; a dedicated host decides who speaks when (still a separate implementation) |
| **Complex emotion facility** | Forms a traceable hint from model output and fallback evidence (for example, lingering tension) as a next-turn/presentation candidate, not a plot authority |
| **Slot 1 (memory)** | Cross-character shared tavern memory (\"what did that bartender just say\") |

**New adapter layer needed**: Theatre-specific UI (multi-character bubbles, scene background) plus multi-character turn scheduling and isolation policy. The six-port/state contracts are reusable, but `groups` cannot stand in for a delivered multi-character executor.

---

### S4 · All-Day Embedded Companion

**Scenario narrative**:

> In the morning, the voice assistant wakes you up. On the subway, you chat on your phone. At work, it keeps you company in VS Code. At night, the smart home reminds you to go to bed.

One compatible role pack may be loaded by hosts on different devices. They reuse contracts and role assets; whether they share one process, database, or synchronization service is a deployment choice.

| Time | Device | Surface | What the kernel does |
|------|--------|---------|---------------------|
| 7:00 AM | Smart speaker | Voice TTS/STT | agent slot calls alarm API + character personality greeting (tsundere/gentle/energetic) |
| 8:00 AM commute | Phone app | Mobile chat UI | Standard CoPresent 1v1 chat, memory continues morning context |
| 10:00 AM–6:00 PM work | VS Code | Coding companion panel | S1 scenario, same role pack running continuously |
| 10:00 PM | Smart home | Voice reminder | agent slot checks if still staying up → character personality urges bedtime |

**Key capability**: Role pack **session continuity**—the character remembers what was said in the morning, what was discussed on the subway, how much code was written during the day, and what tone to use when urging bedtime now.

| Kernel capability | What it does in cross-device scenario |
|-------------------|--------------------------------------|
| **Slot 1 (memory)** | LTM may be shared under the cross-host contract; STM is a session buffer and is not advertised as a cross-device source of truth |
| **Slot 3 (event)** | \"User finished today's coding\" injected as event into memory |
| **OOCP** | Devices call the same kernel instance via HTTP (or sync database) |
| **MCP** | Speaker TTS/alarm, phone notifications, home lighting control—all are MCP tools |

---

### S5 · Desktop Character Chat (Existing, Delivered)

**Surface**: Tauri + Vue 3 desktop window, Ctrl+Shift+F/M for plugin and model management.

**Status**: Implemented, CI-covered. Serves as the \"standard build\" verification bed for all other scenarios.

---

### S6 · Headless HTTP API (Existing)

**Surface**: `--api --port 8420`, `POST /chat`.

**Use cases**: Server deployment, CI testing, pack editor trial chat, third-party app integration.

**Status**: OOCP S0–S12 integrated in CI.

---

### S7 · Mobile Companion

**Surface**: Mobile app (Tauri mobile or PWA), role packs usable across desktop and mobile.

**Target boundary**: stable ports and turn semantics stay the same, but the mobile host, permissions, storage, and resource budget still need adaptation. The current project exposes a Rust source-level `OcliveKernel` facade, not a stable C ABI; OOCP HTTP is the available integration route until a mobile library delivery is actually verified.

---

### S8 · Smart Home Hub (`robot-gateway`)

**Surface**: Embedded gateway device, voice-controlled home + character personality interaction.

**How the kernel does it**:

| Kernel capability | What it does in smart home |
|-------------------|---------------------------|
| **`--template robot-gateway`** | Generates MCP skeleton + fully welded Monolith build, adapted for embedded devices |
| **Slot 6 (agent)** | MCP tool-calling to control lights, AC, curtains |
| **Slot 1 (memory)** | Remembers user habits (\"you like dim lights on weekends\") |
| **Personality engine** | Announces status in character voice (\"Master, temperature set to 26°C~\" vs \"26 degrees. Happy now?\") |

**Status**: `oclive init --template robot-gateway` already generates MCP skeleton; needs adaptation to specific home protocols (Zigbee/HomeKit/MQTT).

---

### S9 · Robot Role Runtime

**Surface**: Role runtime embedded in a physical robot or its gateway.

**How the kernel does it**:

| Kernel capability | What it does in a robot |
|-------------------|------------------------|
| **Input adapter / ExternalObservation** | Types vision/audio/touch facts without impersonating a user message or directly committing character state |
| **Slot 2 (emotion)** | May provide candidates about human input; it is not the universal classifier for every sensor fact |
| **Event Ring (optional facility)** | Assigns trusted source and causation to physical-event proposals; decisions and a permit gate proactive turns |
| **Presentation/device adapter** | Maps committed expression or action intent to servos, panels, and speech; failures must not write false facts back |
| **Slot 6 (agent)** | MCP controls motors, plays speech, switches expression panel |
| **Rust `OcliveKernel` facade** | Supports current in-process Rust integration; ROS/other-language bindings, RTOS support, and a stable ABI remain separate deliverables |

**Differentiation hypothesis**: OCLive can coordinate input facts, state candidates, model expression, and device actions through stable contracts instead of one unstructured prompt. A robot does not have to enable a thick six-slot emotion pipeline; strong-model and low-resource deployments should both be able to choose thinner assemblies.

---

### S10 · AI NPC Engine

**Surface**: Game engine (Unity/Unreal) calling oclive kernel to drive NPC dialogue.

**How the kernel does it**:

| Kernel capability | What it does for game NPCs |
|-------------------|---------------------------|
| **scene mode** | NPC knows current game scene (location, time, weather, present characters) |
| **Slot 3 (event)** | Game events (player stole something, killed a guard) affect NPC attitude |
| **Slot 1 (memory)** | NPC remembers player's past behavior |
| **Monolith build** | Low-latency fully-welded compilation, suitable for game real-time response |
| **OOCP / library-embed** | Game engine calls kernel via HTTP or FFI |

**Difference from AI game NPC solutions**: Inworld, Convai, etc. are cloud-based closed services. oclive allows game developers to **self-host** the character engine locally, and freely swap memory/emotion/LLM backends.

---

## Scenario Capability Matrix

| Scenario | Slot 1 memory | Slot 2 emotion | Slot 3 event | Slot 4 prompt | Slot 5 llm | Slot 6 agent | Complex emotion | MCP extension | Role pack | Form |
|----------|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|------|
| S1 Coding companion | ✅ | ✅ ✅ | — | ✅ | ✅ | ✅ (opt) | — | VS Code API | ✅ | VS Code ext |
| S2 Character casino | ✅ ✅ | — | ✅ | ✅ | ✅ | ✅ ✅ | ✅ | Game engine | ✅ ✅ (multi) | Desktop game |
| S3 AI theatre | ✅ ✅ | — | — | ✅ | ✅ | — | ✅ ✅ | — | ✅ ✅ (multi) | Multi-char UI |
| S4 All-day | ✅ ✅ ✅ | ✅ | ✅ ✅ | ✅ | ✅ | ✅ ✅ | ✅ | TTS/alarm/home | ✅ | Multi-device |
| S5 Desktop chat | ✅ | ✅ | ✅ | ✅ | ✅ | — | ✅ | — | ✅ | Tauri |
| S6 Headless API | ✅ | ✅ | ✅ | ✅ | ✅ | — | — | External calls | ✅ | HTTP |
| S7 Mobile | ✅ | ✅ | ✅ | ✅ | ✅ | — | — | — | ✅ | Mobile app |
| S8 Home hub | ✅ | — | ✅ | ✅ | ✅ | ✅ ✅ | — | Zigbee/MQTT | ✅ | Embedded |
| S9 Robot | ✅ | ✅ ✅ ✅ | ✅ ✅ | ✅ | ✅ | ✅ ✅ | ✅ ✅ | Sensors/motors | ✅ | Embedded |
| S10 NPC | ✅ ✅ | ✅ | ✅ ✅ | ✅ | ✅ | — | ✅ | Game engine | ✅ | library |

> ✅ = light use | ✅ ✅ = heavy dependency | ✅ ✅ ✅ = core driver

---

## Relationship to Other Documents

| Document | Relationship |
|----------|-------------|
| [VISION_OPEN_LAB.md](VISION_OPEN_LAB.md) | This document expands open experimentation as one optional use, not the definition of every product form |
| [VISION_ROADMAP_MONTHLY.md](VISION_ROADMAP_MONTHLY.md) | Monthly roadmap focuses on kernel engineering milestones; this document focuses on product-level application narrative |
| [OCLIVE_ARCHITECTURE_OVERVIEW.md](../getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md) | Terms used here (\"six slots\", \"facility sub-modules\", \"OOCP/MCP\") are authoritatively defined in that document |
| [KERNEL_FACTORY_VISION.md](../getting-started/KERNEL_FACTORY_VISION.md) | S8/S9/S10 corresponding templates (`robot-gateway`/`robot-soul`/`library-embed`) are already implemented by factory CLI |
| [APPLICATION_SCENARIOS.md](../../creator-docs/roadmap/APPLICATION_SCENARIOS.md) | Chinese version |

---

*This document is updated continuously as new scenarios are explored. Whenever the kernel is proven capable of supporting a new product form, add it to the matrix above.*"
