# reply-post-process-polish

Directory plugin for **optional LLM reply polish** (`reply_post_process.process`).

- **Rule gate**: only calls Ollama when `shouldPolish` hits (user echo, markdown/code, very long reply).
- **Preset**: cached per `role_id` from `polish_prompt.md` **or** `core_personality.txt` + `meta.reply_quality_anchor`.
- **Default**: pass-through when rules miss or Ollama unavailable (conversation continues).

## Install

Copy this folder to one of:

- `%LOCALAPPDATA%/OCLive/data/plugins/reply-post-process-polish/`
- `{roles_parent}/plugins/reply-post-process-polish/`

First spawn requires user grant for **`process:spawn`**.

## Enable (dev only)

In a **local** role pack `config.json` (see `roles/polish-dev/`):

```json
{
  "reply_post_processor": {
    "enabled": true,
    "backend": "directory",
    "directory": { "plugin_id": "reply-post-process-polish" }
  }
}
```

Do **not** enable by default in shipped golden packs (mumu / 枫侵月).

## Environment

| Variable | Default | Description |
|----------|---------|-------------|
| `OCLIVE_ROLES_DIR` | injected by host on spawn | Role pack root |
| `OCLIVE_POLISH_OLLAMA_URL` | `http://127.0.0.1:11434` | Ollama base URL |
| `OCLIVE_POLISH_MODEL` | empty (skip LLM) | Second-pass model; e.g. `qwen2.5:3b` |
| `OCLIVE_POLISH_MAX_EXCERPT` | `800` | `core_personality` excerpt cap |

## Modules

| File | Role |
|------|------|
| `preset_cache.mjs` | Cache preset by `role_id` + pack mtime |
| `preset_builder.mjs` | Build system preset from role pack files |
| `polish_rules.mjs` | `shouldPolish(raw, userMessage)` gate |
| `ollama_client.mjs` | `POST /api/chat` (system + user) |
| `rpc_server.mjs` | JSON-RPC server (`node rpc_server.mjs`) |

## Tests

```bash
node --test examples/reply-post-process-polish/*.test.mjs
```

## Acceptance checklist

| Case | Expected |
|------|----------|
| Post-processor disabled | Same as production, no extra latency |
| Enabled + rules miss | pass-through, `diagnostic` contains `skip:rules` |
| Enabled + rules hit + Ollama up | `display_reply` polished |
| Ollama down | raw reply, warn log, chat continues |
| Role switch | preset rebuilt / cache hit per `role_id` |
| `polish_prompt.md` present | overrides auto preset |

See [handoff/REPLY_POST_PROCESSOR_DESIGN_REPORT.md](../../handoff/REPLY_POST_PROCESSOR_DESIGN_REPORT.md).

## Scope (Opus 4.8 — stop expanding)

This plugin is **technical pre-research** for Theater v0 local beat patch only. **Not** the Theater product. Minimal loop is complete — do not add features until Theater v0 is validated. See [handoff/REPLY_POST_PROCESS_POLISH_SCOPE.md](../../handoff/REPLY_POST_PROCESS_POLISH_SCOPE.md).
