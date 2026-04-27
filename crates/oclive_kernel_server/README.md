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

## Status (Phase 2)

This binary currently wires the **OOCP transport + protocol handler** but does **not** yet
wire the full runtime (roles, DB, plugins). That wiring is the next step of the “Linux kernel”
transition.

