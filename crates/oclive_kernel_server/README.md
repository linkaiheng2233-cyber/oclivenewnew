# oclive_kernel_server

Standalone **Oclive Kernel Server** (no Tauri). It exposes:

- `GET /health` → `"ok"`
- `GET /oocp` → OOCP WebSocket endpoint (capabilities first frame)

## Run

```bash
# default: 127.0.0.1:48888
cargo run -p oclive_kernel_server
```

Optional env:

- `OOCP_API_PORT`: listening port (default `48888`)
- `OOCP_API_TOKEN`: enable Bearer auth (optional)
- `OCLIVE_DB_PATH`: sqlite db path (optional; default under app data dir)
- `OCLIVE_ROLES_DIR`: roles root dir (optional)
- `OCLIVE_APP_DATA_DIR`: app data dir (optional)

## Status

This binary runs the **full kernel runtime** (roles, DB, plugin backends) and exposes it via OOCP.

