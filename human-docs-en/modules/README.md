# Module packs · picker (EN summary)

> **Readers**: Engineers who finished L0–L3 and work on **one module**, not the full `process_message` chain.  
> **SSOT**: Chinese full packs → [`human-docs/modules/README.md`](../../human-docs/modules/README.md) · definitions → [MODULE_MAP](../../handoff/MODULE_MAP_AND_HANDOFF.md).  
> **Coverage**: **≥80%** — all 20 module packs mirrored (EN summary + ZH checklist link).  
> **Last updated**: 2026-08-16

---

## How to use

1. Finish [00–04](../README.md) generic layer (L0–L3).
2. Pick **one** pack from the table below.
3. §3 in each ZH pack links to creator-docs / handoff SSOT — **do not** copy six-slot tables or PLUGIN_V1 full text into PRs.
4. Changing `process_message` order → [06 kernel learning path](../06_KERNEL_LEARNING_PATH.md) (**main-chain maintainers only**).

---

## By MODULE_MAP category

| Category | Pack | ZH SSOT | EN summary |
|----------|------|---------|------------|
| **Six slots** | `memory` | [ZH](../../human-docs/modules/slots/memory.md) | [EN](slots/memory.md) |
| | `emotion` | [ZH](../../human-docs/modules/slots/emotion.md) | [EN](slots/emotion.md) |
| | `event` | [ZH](../../human-docs/modules/slots/event.md) | [EN](slots/event.md) |
| | `prompt` | [ZH](../../human-docs/modules/slots/prompt.md) | [EN](slots/prompt.md) |
| | `llm` | [ZH](../../human-docs/modules/slots/llm.md) | [EN](slots/llm.md) |
| | `agent` | [ZH](../../human-docs/modules/slots/agent.md) | [EN](slots/agent.md) |
| **Facilities** | `complex-emotion` | [ZH](../../human-docs/modules/facilities/complex-emotion.md) | [EN](facilities/complex-emotion.md) |
| | `portrait` | [ZH](../../human-docs/modules/facilities/portrait.md) | [EN](facilities/portrait.md) |
| | `visual-stage` | [ZH](../../human-docs/modules/facilities/visual-stage.md) | [EN](facilities/visual-stage.md) |
| **Role packs** | `role-pack-content` | [ZH](../../human-docs/modules/packs/role-pack-content.md) | [EN](packs/role-pack-content.md) |
| | `role-pack-config` | [ZH](../../human-docs/modules/packs/role-pack-config.md) | [EN](packs/role-pack-config.md) |
| **Side channels** | `chat-storage` | [ZH](../../human-docs/modules/side-channels/chat-storage.md) | [EN](side-channels/chat-storage.md) |
| | `user-identity` | [ZH](../../human-docs/modules/side-channels/user-identity.md) | [EN](side-channels/user-identity.md) |
| | `reply-post-process` | [ZH](../../human-docs/modules/side-channels/reply-post-process.md) | [EN](side-channels/reply-post-process.md) |
| | `reply-mode` | [ZH](../../human-docs/modules/side-channels/reply-mode.md) | [EN](side-channels/reply-mode.md) |
| **Orchestration** | `turn-thinking` | [ZH](../../human-docs/modules/orchestration/turn-thinking.md) | [EN](orchestration/turn-thinking.md) |
| | `model-tier` | [ZH](../../human-docs/modules/orchestration/model-tier.md) | [EN](orchestration/model-tier.md) |
| **Surfaces** | `frontend-chat-pro` | [ZH](../../human-docs/modules/surfaces/frontend-chat-pro.md) | [EN](surfaces/frontend-chat-pro.md) |
| | `tauri-invoke` | [ZH](../../human-docs/modules/surfaces/tauri-invoke.md) | [EN](surfaces/tauri-invoke.md) |
| | `distro-hostprofile` | [ZH](../../human-docs/modules/surfaces/distro-hostprofile.md) | [EN](surfaces/distro-hostprofile.md) |

Panorama index (ZH): [ARCHITECTURE_DECOUPLING_PANORAMA.md](../../human-docs/team/ARCHITECTURE_DECOUPLING_PANORAMA.md)

---

## By role (quick routes)

| You are | Path |
|---------|------|
| Plugin / LLM backend author | [paths/plugin-author.md](../paths/plugin-author.md) → `slots/llm` or `slots/agent` |
| Role pack copy author | L0–L2 → [packs/role-pack-content.md](packs/role-pack-content.md) |
| Chat Pro frontend | [paths/frontend.md](../paths/frontend.md) → `surfaces/` |
| Integrator / headless HTTP | [paths/integrator.md](../paths/integrator.md) → `surfaces/distro-hostprofile` |
| Main-chain maintainer | L5 [06](../06_KERNEL_LEARNING_PATH.md) + MODULE_MAP deep read |

**Kernel main-chain maintainers** still use [06_KERNEL_LEARNING_PATH](../06_KERNEL_LEARNING_PATH.md) (L5), not module packs alone.

Contracts in English: [creator-docs-en/](../../creator-docs-en/).
