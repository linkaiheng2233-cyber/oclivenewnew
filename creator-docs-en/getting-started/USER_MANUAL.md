# oclive user manual

**SSOT scope:** installation, chat, settings, backup and troubleshooting for the current default Chat Pro role workspace; not role-pack or plugin contracts.
**Last updated:** 2026-10-27.
**Interface scope:** launch into the role library, then select/import a character to enter its workspace. Legacy Fluent / Tool menus and experiments may be absent.

[中文](../../creator-docs/getting-started/USER_MANUAL.md) · [Version and candidate status](PROJECT_CURRENT_STATUS.md) · [Error codes](ERROR_CODES.md)

## 1. Install and first launch

### 1.1 What to prepare

| Item | Notes |
|------|-------|
| Installer | Use a package for your system. This candidate provides Windows x64 installers; source/CI coverage of another platform does not establish equivalent installer availability. |
| Model | Chat Pro does not bundle Ollama, llama-server, weights or LoRA adapters. Replies require a working local model service or cloud API. |
| Computer and network | Local memory, VRAM and storage needs depend on the model and runner; cloud use needs connectivity and service credit. |

### 1.2 Install

1. Extract the complete test package and read its instructions. Run `tools/Verify-Package.cmd` when supplied. Checksums establish integrity, not publisher identity.
2. Existing users should close the app and back up first. Do not manually mix old/new `resources` directories.
3. Windows needs WebView2. If an offline installation lacks it, prepare the runtime before retrying and follow the installer message.
4. For unsigned installers, verify the source and supplied checksums. [CONTRIBUTING](../../CONTRIBUTING.md) covers source builds; ordinary users need not compile.

### 1.3 First launch

1. Select or import a character in the **role library**.
2. Enter its workspace, open **Model & connection**, choose a local model or cloud API, save and test.
3. Configuration can be deferred. The library remains usable, but normal replies need an available model.
4. Send a short message and wait. Saving proves storage; connectivity and a real reply require separate confirmation.

### 1.4 Local model unreachable

Check that the runner is running, the weights are downloaded, and address/name match the app selection. `ollama list` shows Ollama models. Installing the app does not install a model. Use **Help & recovery** for environment/kernel status and consult [Error codes](ERROR_CODES.md).

## 2. Role packs

### 2.1 Import and update

Use **Import role pack** in the library and follow the picker's supported formats. Check the source and target when an ID collision/overwrite prompt appears. Wait for progress to finish.

The installed app seeds missing bundled roles/plugins into a writable user library without replacing existing packages with the same name. Older custom content after an upgrade is not necessarily installation failure; export and back it up before replacing it.

### 2.2 Switch roles

Return to the library and choose another character. Each has its own workspace. Sidebar thumbnails crop existing assets; the workspace keeps the full portrait and established missing-image fallbacks.

### 2.3 No marketplace button

The default workspace does not expose market/share-install entries. Import packs from a known source rather than searching for unavailable legacy menus. Authors can read [Creator workflow](CREATOR_WORKFLOW.md) and [Compatibility](../COMPATIBILITY.md).

## 3. Daily chat

### 3.1 Chat and stop

The main area contains messages and input. Stop an in-progress generation when needed. Read the short failure explanation before expanding details. Repeated sends can duplicate work or external charges.

### 3.2 Shortcuts

Use in-app help and **Settings → Advanced settings → Keybindings**. Availability depends on the shell, role and capabilities; legacy plugin-manager, debug or voice shortcuts do not guarantee a corresponding default-workspace entry.

### 3.3 Recent conversations and records

The sidebar lists the current character's recent conversations. Create separate records for that character and use **Settings → Conversation records** to view, search, edit or export.

**A new conversation separates the transcript; memory and relationships continue.** It does not reset the character. Hiding story controls preserves older scene records. Developers can read the [transcript/memory boundary](../../handoff/CHAT_STORAGE_ARCHITECTURE.md#transcript-only-conversations).

### 3.4 Identity, relationship and expressions

Choose identity through the character's identity control or **Settings → Roles & identity**; use the workspace relationship control for relationship selection. Pack choices and defaults apply. These are separate settings without requiring the legacy debug panel. A pack without an identity catalog does not gain templates automatically.

Portraits follow replies and pack assets, retaining missing-image fallbacks. The independent expression hint does not add a model call for image selection. Accuracy still depends on model adherence; an image alone does not prove internal state.

## 4. Settings, models and data

### 4.1 Categories

| Category / entry | Purpose |
|------------------|---------|
| Appearance & language | Language, light/dark and scale; the top-right switch changes light/dark quickly. System following remains a settings choice. |
| Chat experience | Existing input, presentation and chat preferences. |
| Roles & identity | Role packs and supported user identities; relationship selection is in the workspace. |
| Conversation records | View, search, edit, export or clean a defined scope; back up first. |
| Connection & repair | Kernel connection and recovery options. |
| Advanced settings | Existing advanced preferences/keybindings, not automatic reopening of experiments. |
| Help & recovery | Kernel/environment, recent errors and logs, with manual copying/saving. |

The Win98 Easter egg retains an independent classic palette. Disabling it restores default appearance without resetting roles, drafts or model configuration.

### 4.2 Model & connection

| Route | Configuration and checks |
|-------|--------------------------|
| Local | Prepare the runner/weights independently, then use supported panel selection/import. For Ollama, verify address and actual model name. GGUF/LoRA also depends on runner/model compatibility. |
| Cloud API / compatible endpoint | Use the provider's address, model ID and key; save and test. A free app does not make external services free. Never send keys to the maintainer. |

Cloud connection tests use saved settings; save edited endpoints, models or keys first. Record backend/model ID and test a short message. Troubleshoot the role's effective configuration, not merely the last form edited.

### 4.3 Update, backup and uninstall

Close the app and back up its actual user-data paths, role library, plugin configuration and chat storage, including configured external locations. **Installation files and user data differ.** Copying only the program does not preserve chats.

Use the installer for upgrades. Rollback needs the corresponding older version and a pre-upgrade backup; do not assume older code reads newer data. Uninstall through the system app interface and separately confirm personal-data deletion. Chat Pro, the pack editor and model runners are separate apps.

## 5. Troubleshooting and feedback

### 5.1 Blank window or endless loading

After the UI loads, if a recovery card says the area cannot be displayed, expand/copy its details or choose “Reopen this area”. A work-area rendering failure keeps the Help & recovery entry available; an outer rendering failure has a root recovery card. Retrying does not resend messages or clear chat history. Diagnostics remain local for this launch and do not require remote error reporting.

Check installation resources and WebView2, then close and relaunch. If the UI cannot open, use the supplied repair tool with the correct installation folder. Do not delete the entire user directory. Start with Help & recovery when accessible.

### 5.2 Thinking, no reply or model errors

“Stopped receiving the previous reply” only describes the local receiver; it does not guarantee server generation stopped. Reopen the conversation to inspect saved content. This notice never automatically resends the message.

Check backend, address, model name and reachability. Read the summary/details; stop an unfinished generation before resending. If both primary and fallback fail, check both services.

### 5.3 Useful feedback

Include version, system, pack/model name, time, minimal steps, expected/actual results and a small redacted log excerpt. The panel masks common keys, but review paths/text before sharing. Do not send keys, full chat databases or whole user directories. Help & recovery does not upload logs automatically.

## 6. Advanced features and further reading

The default workspace hides story/scene/time controls, voice, history-based memory re-extraction, marketplace and internal debugging. This preserves ordinary chat memory and existing configuration; previously enabled experiments may retain a management/disable path. See the [deferred/frozen surface map](../../handoff/MODULE_MAP_AND_HANDOFF.md#chatpro-deferred-surfaces).

- [Error codes](ERROR_CODES.md): apply legacy-shell/developer steps only within their scope.
- [Configuration/storage](../guides/CONFIGURATION_FILES.md): ownership and custom locations.
- [Plugin/role-pack FAQ](../FAQ.md): author troubleshooting; users need not enable plugin panels first.
- [Documentation index](DOCUMENTATION_INDEX.md): choose creation, development or integration paths as needed.
