# reply-post-process-polish — scope closure

**Status:** Minimal loop **Done** · **Stop expanding**

## Positioning

Technical pre-research for **Theater v0 local beat patch** (rule gate + preset cache + Ollama pass-through). **Not** the Theater product itself.

## Delivered (minimal loop)

| Piece | Location |
|-------|----------|
| Directory plugin | `examples/reply-post-process-polish/` |
| Rule gate | `polish_rules.mjs` + tests |
| Preset builder / cache | `preset_builder.mjs`, `preset_cache.mjs` |
| Ollama client | `ollama_client.mjs` |
| Integration smoke | `src-tauri/tests/reply_post_processor_directory_roundtrip.rs` |
| Dev role pack | `roles/polish-dev/` |

## Orchestration contract (unchanged)

`turn_pipeline/post.rs` calls `process_reply` **after** portrait/favor persistence, **before** `append_turn_to_chat_storage`. Response field is **`reply`**.

## Stop line

Do **not** add features to this plugin until Theater v0 is validated. Theater beat patch lives in frontend (`src/theater/useTheaterBeatPatch.ts`), not `process_message`.
