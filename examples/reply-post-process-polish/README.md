# reply-post-process-polish

Directory plugin scaffold for **optional LLM reply polish** (`reply_post_process.process`).

- **Default behavior**: pass-through (`display_reply` = `raw_reply`).
- **Replace** `polishReply()` in `rpc_server.mjs` with your LLM call.
- **Do not** enable by default in shipped role packs; set `config.json` → `reply_post_processor.enabled: true` when ready.

See [handoff/REPLY_POST_PROCESSOR_DESIGN_REPORT.md](../../handoff/REPLY_POST_PROCESSOR_DESIGN_REPORT.md).
