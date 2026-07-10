"# Application Scenario Matrix (Kernel Capability Proof)

This document enumerates concrete product forms that the oclive kernel (`process_message` + six host slots + facility sub-modules + OOCP/MCP) can support. **The goal is to prove: one kernel, infinite scenarios. Role packs are kernel capability.**

Alongside [VISION_OPEN_LAB.md](VISION_OPEN_LAB.md), this is a pillar of the \"open lab\" thesis. It aligns with the monthly roadmap in [VISION_ROADMAP_MONTHLY.md](VISION_ROADMAP_MONTHLY.md) without binding to specific delivery dates.

[中文](../../creator-docs/roadmap/APPLICATION_SCENARIOS.md)

---

## Core Principle

```
One kernel + Different role packs + Different surface shells = Radically different products
```

| What changes | What stays the same |
|-------------|---------------------|
| Surface form (desktop window / VS Code extension / voice speaker / mobile app / smart home panel) | `chat_engine::process_message` orchestration |
| Personality (role pack manifest.json + prompts/ + personality archive) | PluginHost six slots |
| Interaction protocol (Tauri IPC / OOCP HTTP / MCP stdio) | Complex emotion facility sub-module |
| External tools (file ops / game engine / home control) | Role pack contract (manifest + pipeline.ocblueprint) |

---

## Scenarios at a Glance

| # | Scenario | Form | Key Kernel Capabilities |
|---|----------|------|------------------------|
| S1 | **Coding Companion** | VS Code distribution (official shell) | emotion slot monitors editing behavior / personality engine adjusts tone |
| S2 | **Character Casino** | Multi-character game desktop | CoPresent multi-character / agent slot game rules engine / memory for game history |
| S3 | **AI Theatre** | Multi-character story generator | scene mode / complex emotion facility sub-module / blueprint groups |
| S4 | **All-Day Embedded Companion** | Smart speaker / robot / wearable | Full six slots + MCP device control + `--template robot-soul` |
| S5 | **Desktop Character Chat** (existing) | Tauri desktop app | Standard 1v1 CoPresent |
| S6 | **Headless HTTP API** (existing) | Server deployment | `--api` + `POST /chat` + OOCP |
| S7 | **Mobile Companion** | Mobile app (Tauri mobile / PWA) | Same as S5, different shell |
| S8 | **Smart Home Hub** | Gateway / home hub | `--template robot-gateway` + MCP home toolchain |
| S9 | **Robot Emotion Kernel** | Embedded in robot OS | `library-embed` mode / agent slot sensor input |
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
| **Slot 2 (emotion)** | MCP server reads VS Code diagnostics (error count, code change frequency, editing duration), produces `emotion_output` |
| **Personality engine** | Decides tone direction (mock/care/encourage) based on emotion output + role pack personality archive |
| **Slot 4 (prompt)** | Injects current file context + errors + personality tone instructions |
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

**Surface**: Multi-character scene performance, characters take turns speaking according to blueprint `groups`.

**Example**: Three characters meet in a tavern, complex emotion facility drives plot direction.

**How the kernel does it**:

| Kernel capability | What it does in this scenario |
|-------------------|------------------------------|
| **scene mode** | Defines tavern scene, time, character list |
| **blueprint groups** | Defines inter-character relationships (who knows whom, initial affinity) |
| **Complex emotion facility** | Generates narrative hints (\"the atmosphere grows tense\", \"she hesitates to speak\"), guiding plot development |
| **Slot 1 (memory)** | Cross-character shared tavern memory (\"what did that bartender just say\") |

**New adapter layer needed**: Theatre-specific frontend UI (multi-character bubbles, scene background). Kernel capabilities are ready.

---

### S4 · All-Day Embedded Companion

**Scenario narrative**:

> In the morning, the voice assistant wakes you up. On the subway, you chat on your phone. At work, it keeps you company in VS Code. At night, the smart home reminds you to go to bed.

**One role pack**, running on different devices at different times. One kernel, different surfaces.

| Time | Device | Surface | What the kernel does |
|------|--------|---------|---------------------|
| 7:00 AM | Smart speaker | Voice TTS/STT | agent slot calls alarm API + character personality greeting (tsundere/gentle/energetic) |
| 8:00 AM commute | Phone app | Mobile chat UI | Standard CoPresent 1v1 chat, memory continues morning context |
| 10:00 AM–6:00 PM work | VS Code | Coding companion panel | S1 scenario, same role pack running continuously |
| 10:00 PM | Smart home | Voice reminder | agent slot checks if still staying up → character personality urges bedtime |

**Key capability**: Role pack **session continuity**—the character remembers what was said in the morning, what was discussed on the subway, how much code was written during the day, and what tone to use when urging bedtime now.

| Kernel capability | What it does in cross-device scenario |
|-------------------|--------------------------------------|
| **Slot 1 (memory)** | STM shared across devices (via unified SQLite / optional cloud sync), LTM records long-term habits |
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

**Kernel difference**: Zero. Shares the same kernel with desktop (compiled via `library-embed` mode as `.so`/`.dylib` for mobile invocation, or calling desktop/server kernel instance via OOCP HTTP).

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

### S9 · Robot Emotion Kernel

**Surface**: Emotion processing unit for physical robots (humanoid/desktop type).

**How the kernel does it**:

| Kernel capability | What it does in a robot |
|-------------------|------------------------|
| **Slot 2 (emotion)** | Sensor input (vision/audio/touch) → emotion analysis |
| **Complex emotion facility** | Body language narrative hints (\"she tilts her head slightly, fingers unconsciously twisting together\") → corresponding servo motor actions |
| **Slot 3 (event)** | Physical events (being touched, face recognized, falling) as event estimation |
| **Slot 6 (agent)** | MCP controls motors, plays speech, switches expression panel |
| **`library-embed`** | Compiled as `.so`, embedded in robot main controller ROS/RTOS |

**Difference from typical \"AI robots\"**: Most robots just attach a generic LLM + voice. oclive provides a **six-slot deep-processing emotion pipeline**: sensor input → emotion analysis → narrative hint → memory storage → prompt assembly → response generation → motor action, fully replaceable throughout.

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
| [VISION_OPEN_LAB.md](VISION_OPEN_LAB.md) | This document is its scenario-based expansion: the open lab is not an abstract slogan, but the sum of these concrete product forms |
| [VISION_ROADMAP_MONTHLY.md](VISION_ROADMAP_MONTHLY.md) | Monthly roadmap focuses on kernel engineering milestones; this document focuses on product-level application narrative |
| [OCLIVE_ARCHITECTURE_OVERVIEW.md](../getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md) | Terms used here (\"six slots\", \"facility sub-modules\", \"OOCP/MCP\") are authoritatively defined in that document |
| [KERNEL_FACTORY_VISION.md](../getting-started/KERNEL_FACTORY_VISION.md) | S8/S9/S10 corresponding templates (`robot-gateway`/`robot-soul`/`library-embed`) are already implemented by factory CLI |
| [APPLICATION_SCENARIOS.md](../../creator-docs/roadmap/APPLICATION_SCENARIOS.md) | Chinese version |

---

*This document is updated continuously as new scenarios are explored. Whenever the kernel is proven capable of supporting a new product form, add it to the matrix above.*"