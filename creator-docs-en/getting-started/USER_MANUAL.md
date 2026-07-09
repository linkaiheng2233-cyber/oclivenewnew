# oclive user manual

[中文](../../creator-docs/getting-started/USER_MANUAL.md)

For people who **only use the app** (no role-pack authoring, no plugin development). Error triage: [ERROR_CODES.md](ERROR_CODES.md). Build/install from source: root [README.md](../../README.md).

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
- **Residual UI** may still be tracked under checklist §A6 ([PRODUCT_AND_KERNEL_GAP_CHECKLIST](../../handoff/archive/PRODUCT_AND_KERNEL_GAP_CHECKLIST.md)).  
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
