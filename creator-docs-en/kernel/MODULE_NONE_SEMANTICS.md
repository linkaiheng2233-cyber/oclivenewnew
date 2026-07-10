# Six-slot `none` semantics

[中文](../../creator-docs/kernel/MODULE_NONE_SEMANTICS.md)

**Status**: v0.3.x runtime contract  
**SSOT enums**: `kernel/crates/oclive_validation/src/plugin_backends.rs` (`*Backend::None`)  
**Noop impl**: `kernel/crates/oclive_kernel_host/src/domain/noop_slot_backends.rs`

---

## 1. Positioning

`plugin_backends.<slot> = none` means **that orchestration slot does not participate in business logic this turn** — a zero-cost Noop backend implements the trait, **not** silent fallback to `builtin`.

Relation to **`host_flags.skip_agent`**:

| Mechanism | Scope | Effect |
|-----------|-------|--------|
| `plugin_backends.agent = none` | Role / session effective backends | Agent Noop; `process()` returns unhandled |
| `host_flags.skip_agent = true` | Distro `distro.oclive.toml` | Runtime forces `agent = none` (`apply_host_ceiling`) |

Distros should prefer `skip_agent`; packs may declare `agent: none` directly.

---

## 2. Per-slot behavior

| Slot | `none` behavior | Co-present dialogue |
|------|-----------------|---------------------|
| **memory** | No LTM retrieval; empty list | Allowed |
| **emotion** | Neutral seven-dim vector | Allowed |
| **event** | `Ignore` / 0 impact | Allowed |
| **prompt** | `build_prompt` → `InvalidParameter` | **Forbidden** (startup health blocks) |
| **llm** | `generate` → `InvalidParameter` | **Forbidden** (startup health blocks) |
| **agent** | `process` → `handled: false` | Allowed (skip Agent short-circuit) |

**complex_emotion** is not a six-slot enum; disable via `[slots] complex_emotion = "off"` / `skip_complex_emotion`.

---

## 3. Agent `remote` / `directory`

v0.3.x+ **implemented** host-orchestrated Agent remote/directory:

- Protocol: [AGENT_REMOTE_PROTOCOL.md](../plugin-and-architecture/AGENT_REMOTE_PROTOCOL.md)
- MCP via `AgentMcpBridge`; failure (except grant denial) → `BuiltinReActAgent`
- Blueprint / settings may use `agent` = `remote` / `directory` / `none`

---

## 4. Blueprint and settings

- `pipeline.ocblueprint` → `slot_registry[].backend` may be `"none"`
- `settings.json` → `plugin_backends.*` may be `"none"`
- `distro.oclive.toml` → `[plugin_backends]` may declare `none`

---

## 5. Validation

- Startup: `validate_plugin_backends_slots` + `validate_co_present_dialogue_backends` (`startup_health.rs`)

---

## Related

- [DISTRO_CAPABILITY_PROFILE.md](DISTRO_CAPABILITY_PROFILE.md)
- [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md)
