# oclive user manual

[中文](../../creator-docs/getting-started/USER_MANUAL.md)

**SSOT scope:** installation, daily chat, settings, and self-service troubleshooting for people who use the app without authoring role packs or plugins.
**Last updated:** 2026-08-24.
**Audience:** A.I.Live Chat Pro users.

Error triage: [ERROR_CODES.md](ERROR_CODES.md). Build/install from source: root [README.md](../../README.md).

> **Experimental features (0.5.2):** **R18** and **voice (ASR/TTS)** are still under development and are provided only as optional experiments. Their UI, configuration, model compatibility, and runtime behavior may change between releases. Neither feature is required for normal text chat.

---

## 1. Install and first launch

### 1.1 System requirements

| Topic | Notes |
|-------|--------|
| **OS** | Follow the release notes for your platform; **Windows** and **Linux** are common CI targets. |
| **Hardware** | Local LLMs need **RAM and disk**; cloud sidecars follow their own docs. |
| **Ollama** | For **local Ollama**, install [Ollama](https://ollama.com/) and run **`ollama pull`** for at least one chat model. The model name may come from settings or the role pack. |

### 1.2 Install the app

- Use the **offline installer** from your distribution channel (see root README “Observability & releases”).  
- If antivirus/firewall prompts appear on first launch, allow as appropriate so local services can connect.

### 1.3 First launch checklist

1. Change UI language if needed: **Settings → General → Language**.  
2. Ensure a **role pack** is available: use in-app **import**, or install a zip via [oclive-launcher](https://github.com/linkaiheng2233-cyber/oclive-launcher/blob/main/README.md) (see [CREATOR_WORKFLOW.md](CREATOR_WORKFLOW.md) for paths like `OCLIVE_ROLES_DIR`).  
3. Run **Settings → General → Environment check** to confirm **Ollama**, roles directory, and app data writability.

### 1.4 If Ollama is unreachable

1. Confirm the Ollama service is running and reachable at the URL shown in the environment check.  
2. Run **`ollama pull <model>`** in a terminal.  
3. If it still fails: see [ERROR_CODES.md](ERROR_CODES.md) **§1.5**, and attach the **environment check summary** when opening an issue (no API keys).

---

## 2. Role packs

### 2.1 Import a role pack

1. Use the in-app **import** entry for **`.ocpak` / `.zip`** or a folder, following on-screen text.  
2. If the pack **ID collides** with an existing role, confirm **overwrite** only when intended.  
3. Wait for **progress** to finish on long imports.

### 2.2 Switch roles

- Pick another role from the **role list / top bar** (exact control depends on layout).  
- The first message after a switch may run **startup health** checks; follow prompts or [ERROR_CODES.md](ERROR_CODES.md).

### 2.3 Where is the “market”?

- **In-app**: if a **community index** is wired, you will see refresh/browse style actions; offline mode may use **cached index** with a banner.  
- **Index format** (reference): [../role-pack/ROLE_PACK_INDEX.md](../role-pack/ROLE_PACK_INDEX.md).  
- **Launcher zip install**: [oclive-launcher README](https://github.com/linkaiheng2233-cyber/oclive-launcher/blob/main/README.md).

---

## 3. Daily chat

### 3.1 UI overview

- **Main area**: message list + composer.  
- **Sidebar**: role info, scenes, favorability, etc. (depends on `ui.json` and mode).  
- **Co-presence / remote**: extra bars or notices may appear when the pack enables them.

### 3.2 Common shortcuts

| Shortcut | Action |
|----------|--------|
| **Ctrl+Shift+S** | Open **Settings** |
| **Ctrl+Shift+F** | Open **plugin manager** (V1 vs V2 preview depends on experimental toggle in Settings) |
| **Ctrl+Shift+D** | Toggle **debug panel** (power-user) |
| **Hold Ctrl ~1s** | May open **shortcut help** (if enabled in your build) |

Treat the in-app shortcut overlay as authoritative.

### 3.3 Message history

- **Scroll up** in the thread; if **“older messages”** blocks exist, expand per UI hints.  
- Limits depend on product settings and scene.

### 3.4 User identity (v0.3)

When a role pack includes **`user_identities/`** (e.g. `mumu`), open the **debug panel** (**Ctrl+Shift+D**) → **Runtime** section → **User identity** dropdown to switch who the user is in the prompt (e.g. “classmate”). Takes effect on the **next** message; independent of the **Relation** dropdown.

### 3.5 Reply post-processor (creators · off by default)

Optional text shaping after the LLM reply (`config.json` → `reply_post_processor`). **Disabled by default**; when enabled, the debug runtime panel shows a read-only status line. See [ROLE_PACK_SPEC §9.7](../role-pack/ROLE_PACK_SPEC.md).

### 3.6 Voice (experimental · Windows · side-channel `voice.asr`)

> **Still under development:** speech recognition, auto-read, model import, and streaming synthesis are experimental. Test with short phrases first and record your engine/profile choices before upgrading. Voice failures should fall back to text chat rather than block sending or reading text messages.

> **Architecture:** voice does **not** enter six slots / main orchestration; it uses the official directory plugin [`com.oclive.voice.asr`](../../distros/chat-pro/plugins/com.oclive.voice.asr/). See [`TRACK_VOICE_RECOGNITION.md`](../../human-docs/team/TRACK_VOICE_RECOGNITION.md).

| Feature | Notes |
|---------|--------|
| **Hold to talk** | Chat toolbar mic; default **V hold** when the window is focused and the input is not |
| **ASR result** | Settings → Voice: send immediately or fill the input draft |
| **TTS playback** | Enable global voice expansion and auto-read, then enable individual roles in the role list; pick engine/profile in settings |
| **Role voice hints** | Auto-read requires both a per-role user toggle and that pack's `voice_profile.json`; an optional `synth_profile` overrides only that role's speak jobs and must not rewrite global settings on role switch |
| **Platform** | Current experiments focus on Windows; platform coverage and engine compatibility are still under development. Linux/macOS profiles may return `unsupported_platform` |

### 3.7 R18 (experimental · adults only)

> **Still under development:** R18 is an optional experiment, not a default chat capability. Content generation, beat queues, role-extension compatibility, and UI behavior may change. Keep it disabled when you do not need it.

- Adults only. The current build uses a **local self-declaration** and does not perform online identity verification.
- A role's adult extension is active only when the local adult confirmation, global R18 toggle, and current-role toggle are all enabled.
- Settings provides **“Reset adult confirmation and R18 settings”** to revoke confirmation and reset related toggles and queue settings. It does not automatically delete existing chats or memories.
- A broken or incompatible adult extension should be isolated with a visible error while the role remains available in normal mode.
- R18 is independent from ordinary relationship and identity settings. Disabling it does not remove non-explicit relationship behavior or the base persona.

---

## 4. Settings

### 4.1 General

- **Language**: Settings → General → **UI language**. Residual non-target language may come from **distros/chat-pro/plugins/packs** or **untranslated strings**; see FAQ below.  
- **Telemetry (Sentry)**: opt-out in Settings when DSN is present; see [ERROR_CODES.md](ERROR_CODES.md) §1.7 and [A3 closure](../../handoff/archive/A3_CLOSURE_SUMMARY.en.md).  
- **Environment check**: quick Ollama / paths / writability probe.

### 4.2 LLM backend

- Default is often **local Ollama**; follow the role pack and environment check.  
- **Remote sidecars / BYOK**: [SIDECAR_LLM_USER_GUIDE.md](SIDECAR_LLM_USER_GUIDE.md).

### 4.3 Fallback toggles

- e.g. **“fallback to built-in when remote fails”** (see [../guides/CONFIGURATION_FILES.md](../guides/CONFIGURATION_FILES.md)).  
- Grayed-out items may be **locked by environment variables**; read the inline hint.

---

## 5. FAQ

### 5.1 Why are replies slow?

- **Local models**: cold start, first pull, hardware, context size.  
- **Remote**: network and provider throttling.  
- **Debug panel** (if on) may show timing / fallback usage; see [ERROR_CODES.md](ERROR_CODES.md).

### 5.2 I switched to English but still see Chinese

- **Plugins / packs** ship their own strings.  
- **Residual UI** is tracked in the active [PRODUCT_LINE_TASK_BUCKETS.md](../../handoff/PRODUCT_LINE_TASK_BUCKETS.md) and debt inventory.
- File an issue with **screenshot + version** for host UI gaps.

### 5.3 How do I remove a plugin?

- **Plugin manager** (**Ctrl+Shift+F**): **disable** or **uninstall** per buttons shown.  
- **Directory plugins** may need an **app restart** after removal; see [../plugin-and-architecture/DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md).

---

## 6. More links

| Resource | Link |
|----------|------|
| Plugin/pack FAQ | [../FAQ.md](../FAQ.md) |
| Error codes | [ERROR_CODES.md](ERROR_CODES.md) |
| Config paths | [../guides/CONFIGURATION_FILES.md](../guides/CONFIGURATION_FILES.md) |
| Full doc index | [DOCUMENTATION_INDEX.md](DOCUMENTATION_INDEX.md) |
