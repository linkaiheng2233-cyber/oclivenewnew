# Integrator path

> **Audience**: Engineers building headless HTTP, embedded hardware, or second-party kernel integrations.  
> **Time**: ~1–2 days to onboard.  
> **Chinese SSOT**: [`human-docs/paths/integrator.md`](../../human-docs/paths/integrator.md)
> **Next**: [modules/surfaces/distro-hostprofile.md](../modules/surfaces/distro-hostprofile.md)

---

## Suggested order

1. [02 thirty-minute start](02_THIRTY_MINUTE_START.md)
2. [01 simple architecture](01_ARCHITECTURE_SIMPLE.md) — `process_message` main chain (overview)
3. **Surface pack** → [modules/surfaces/distro-hostprofile.md](../modules/surfaces/distro-hostprofile.md)
4. [KERNEL_INTEGRATOR_LEARNING_PATH](../../creator-docs/getting-started/KERNEL_INTEGRATOR_LEARNING_PATH.md) (SSOT)
5. `cargo run -p oclive-cli -- init` — minimal skeleton

---

## Key entry points

| Capability | Path / command |
|------------|----------------|
| Headless HTTP | `oclivenewnew-tauri --api` or `oclive-kernel-server` |
| Health check | `GET :8420/health` |
| OOCP smoke | `examples/oocp-test-suite/run.mjs` |
| Shared data dir | `OCLIVE_APP_DATA` · [OCLIVE_APP_DATA.md](../../creator-docs/kernel/OCLIVE_APP_DATA.md) |
| VS Code same-origin policy | `resolve_kernel_action` · [CROSS_HOST_MEMORY](../../creator-docs/role-pack/CROSS_HOST_MEMORY.md) |

---

## Acceptance

- [ ] Can name `send_message` stages (align [BUS_FACTOR §1](../../handoff/BUS_FACTOR_NOTES.md))
- [ ] Can run `--api` + mock LLM for one conversation locally

---

## Deep links

- [modules/ picker](../modules/README.md)
- [PURE_KERNEL_BOUNDARY](../../creator-docs/getting-started/PURE_KERNEL_BOUNDARY.md)
- [DISTRO_KERNEL_LIFECYCLE](../../creator-docs/kernel/DISTRO_KERNEL_LIFECYCLE.md) — bundled-first spawn · single-kernel attach/replace
- [KERNEL_SCHEDULER_RESCOPE](../../handoff/KERNEL_SCHEDULER_RESCOPE.md) — scheduler rescope
