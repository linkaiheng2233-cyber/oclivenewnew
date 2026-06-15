# Glossary (one page)

| Term | Meaning |
|------|---------|
| **mrid** | Manifest role id — pack folder / `manifest.json` `id` |
| **srid** | Session role id — `conversation_state_role_id(mrid, session_id)` |
| **pl** | Resolved plugins for the turn (`PluginHost` + six slots) |
| **Six slots** | memory · emotion · event · prompt · llm · agent |
| **plugin_backends** | Per-slot backend choice: `builtin` / `remote` / `directory` / `none` |
| **slot_registry** | Blueprint slot overrides (admin layer) |
| **reply** | API response field (**not** `response`) |
| **OOCP** | OCLive Open Chat Protocol — HTTP black-box tests |

```
User picks mrid → ensure_role_loaded
Optional session_id → srid → DB role_runtime / memory keys
```

Chinese: [human-docs/09_GLOSSARY.md](../human-docs/09_GLOSSARY.md)
