# Side-channel capability enhancements — English summary

[中文](../../creator-docs/rfc/RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md)

Full RFC (Chinese SSOT): [RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md](../../creator-docs/rfc/RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md).

## What they are

**Side-channel capability enhancement modules** are kernel extensions that:

- **Do not** use the six `plugin_backends` / `slot_registry` host keys
- **Do not** take a numbered facility submodule (①–④)
- Wire through a **dedicated resolver** and a **fixed anchor** on the Stable turn chain **or** a **standalone HTTP/Tauri API** outside `process_message`
- May optionally attach a **directory plugin** via manifest `provides` (independent of slot resolution)

**Not:** Experimental `dual_core` pipeline steps, module-4 Prompt slot, or module-5 LLM slot (those only get **consumed**).

## Registry v1 (delivered entries)

| `id` | English name | Anchor / API | Config | `provides` | Status |
|------|--------------|--------------|--------|------------|--------|
| **`user_identity`** | User Identity Prompt Template | `turn_pipeline/pre` → `resolve_active_user_identity` → `PromptBuilder` (**pre-LLM**) | Role pack `user_identities/`; distro `[user_identity]` | None (pack content) | **Delivered** |
| **`reply_post_process`** | Reply Post-Processor Plugin | After built-in `post_llm` → `resolve_reply_post_processor` → `process_reply` | Pack `config.json` → `reply_post_processor`; distro `[post_process].chain` | **`reply_post_process`** · RPC `reply_post_process.process` | **Delivered** |
| **`theater_director`** | Theater Scene Director | **`generate_theater_scene`** / **`POST /theater/scene`** (**outside** `process_message`) | `distro.oclive.toml` → `[theater].director_plugin`; env override | **`theater_director`** · RPC `theater.build_prompt` | **Delivered** |
| **`voice.asr`** | Voice ASR Input | **`chat_toolbar`** + **`plugin_rpc_invoke`** → `send_message` (**outside** turn hooks) | Plugin `models/` + settings | **`voice.asr`** | **Windows delivered** (v0.4) |

Appendix host tools (e.g. **`test_runner`** for pack editor) use `provides` but are **not** registered as side-channel modules.

## vs other architecture layers

| Layer | Uses six slots? | Typical wiring |
|-------|-----------------|----------------|
| Modules 1–6 (backends) | Yes | `PluginHost` → `process_message` |
| Facility modules | No (optional submodule N) | In `turn_pipeline` orchestration |
| **Side-channel** | **No** | **Own resolver** + pre/post anchor or standalone API |
| Backend plugin modules | No (hang on module K) | `provides: ["llm"]`, etc. |

Experimental core changes **step order** on Stable; side-channels are **fixed hooks** or **out-of-band APIs** — not interchangeable.

## Adding a new side-channel

1. RFC (or extend this registry): `id`, English/Chinese names, anchor/API, `provides` if any
2. Implement **`resolve_*`** (or equivalent); **do not** add to `slot_registry` six keys or `pipeline.experimental`
3. Sync architecture overview, [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md) `provides` table, [NAMING_CONVENTIONS.md](../NAMING_CONVENTIONS.md) §1.2

## Related

- Phase 2 detail: [RFC_USER_IDENTITY_AND_REPLY_POST_PROCESSOR_SUMMARY.md](./RFC_USER_IDENTITY_AND_REPLY_POST_PROCESSOR_SUMMARY.md)
- Theater: [handoff/theater/DEVELOPMENT_ROADMAP.md](../../handoff/theater/DEVELOPMENT_ROADMAP.md)
- Architecture: [OCLIVE_ARCHITECTURE_OVERVIEW.md](../getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md)
