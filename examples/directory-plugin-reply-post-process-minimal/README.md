# Reply Post-Process directory plugin (minimal)

Minimal [directory plugin](../directory-plugin-minimal/) that implements JSON-RPC `reply_post_process.process`.

Install: copy this folder to `{app_data}/plugins/reply-post-process-minimal/` and grant `process:spawn` for the plugin id.

Role pack `config.json`:

```json
{
  "reply_post_processor": {
    "enabled": true,
    "backend": "directory",
    "directory": { "plugin_id": "reply-post-process-minimal" }
  }
}
```

The plugin prefixes the display reply with `[dir-pp]`.
