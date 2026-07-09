# Plugin placement guide (contributor decision tree)

[中文](../../creator-docs/plugin-and-architecture/PLUGIN_PLACEMENT_GUIDE.md)

Physical install path: `{app_data}/distros/chat-pro/plugins/<manifest.id>/` (directory plugins). During development, plugins may also be scanned from `distros/chat-pro/plugins/` next to roles and the working tree (see [DIRECTORY_PLUGINS.md](DIRECTORY_PLUGINS.md)).

## Three-question decision tree

### 1. Replace a six-slot backend (memory / emotion / event / prompt / llm / agent)?

→ Declare `provides` with the slot capability (e.g. `llm`, `memory`) in `manifest.json`, and bind the directory or remote plugin in the role blueprint `slot_registry` / `plugin_backends`.

→ Contracts: [PLUGIN_V1.md](PLUGIN_V1.md) · [HOW_TO_REPLACE_MODULES.md](HOW_TO_REPLACE_MODULES.md)

### 2. Polish displayed replies after LLM output (not in Prompt, not a six-slot)?

→ `provides: reply_post_process` · RPC `reply_post_process.process`

→ Role pack `config.json` → `reply_post_processor`; distro `[post_process].chain` may merge policy.

→ Resolver: `resolve_reply_post_processor` ([reply_post_processor.rs](../../kernel/crates/oclive_kernel_host/src/domain/reply_post_processor.rs))

### 3. Generate theater scene prompts (not in `send_message`, not a six-slot)?

→ `provides: theater_director` · RPC `theater.build_prompt`

→ Distro `distro.oclive.toml` → `[theater].director_plugin = "<manifest.id>"`; dev env `OCLIVE_THEATER_DIRECTOR_PLUGIN` overrides profile.

→ Resolver: `resolve_theater_director` ([theater_director.rs](../../kernel/crates/oclive_kernel_host/src/domain/theater_director.rs)); entry `generate_theater_scene` / `POST /theater/scene`.

→ Official example: `distros/chat-pro/plugins/com.oclive.theater_director_official/`

### 4. Turn microphone speech into text for chat (not a six-slot, not a `process_message` hook)?

→ `provides: voice.asr` · RPC `voice.probe` / `voice.transcribe` / `voice.import_model` / **`voice.speak`**

→ UI: `ui_slots` → **`chat_toolbar`** (hold-to-talk) + **`settings.panel`** (model dir / import / `auto_tts`)

→ Text: `com.oclive.voice.asr:submit` event → host `send_message` or `chat:set_input_draft` (`mode: fill`) (see [`voiceAsrEvents.ts`](../../distros/shared/src/lib/voiceAsrEvents.ts))

→ Official example: [`distros/chat-pro/plugins/com.oclive.voice.asr/`](../../distros/chat-pro/plugins/com.oclive.voice.asr/) · HTTP smoke test [`examples/voice-loop-minimal/`](../../examples/voice-loop-minimal/)

## Appendix (not six-slot, not the side channels above)

| Capability | Placement | Notes |
|------------|-----------|-------|
| **Complex emotion `narrative_hint`** | Facility submodule 1 | `complex_emotion` provider; not a directory six-slot key |
| **User identity Prompt** | Role pack `user_identities/` | Side channel `user_identity`; not a plugin directory |
| **Vitest test runner** | `provides: test_runner` | Pack editor tooling; e.g. `official-vue-test-runner` |

## Related

- Side-channel registry: [RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md](../../creator-docs/rfc/RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md) (summary: pending EN mirror)
- Directory plugin scan & permissions: [DIRECTORY_PLUGINS.md](DIRECTORY_PLUGINS.md)
- Theater roadmap: [../../handoff/theater/DEVELOPMENT_ROADMAP.md](../../handoff/theater/DEVELOPMENT_ROADMAP.md)
