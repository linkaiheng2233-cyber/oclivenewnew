# 00 · Vision and positioning

> **Reader:** Engineers contributing code.  
> **Time:** ~15 min.  
> **Next:** [01 Architecture](01_ARCHITECTURE_SIMPLE.md) or [02 Thirty-minute start](02_THIRTY_MINUTE_START.md).

**OCLive (A.I.Live)** is an open, local-first **AI-character tool kernel**: one authoritative turn orchestration path connects six stable capability ports—`memory`, `emotion`, `event`, `prompt`, `llm`, and `agent`. Role packs, editors, distros, and markets help people build and distribute with it, but they are not the kernel's essence.

Stack: **Tauri + Vue 3 + Rust**. Codename: **oclive**.

| Is | Is not |
|----|--------|
| Tool kernel: one orchestration/authority boundary + six stable ports | A fixed vertical “memory engine” product |
| Optional reference runtime and creator toolchain | A centralized platform whose full default stack is mandatory |
| `PluginHost` six slots and stable contracts | Blueprint `steps[]` as first-turn scheduling DSL |
| Role pack (identity, prompts) vs blueprint (`slot_registry`) | Creator fields mixed into six slots |

Deep dive: [creator-docs-en/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md](../creator-docs-en/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md)

The six slots are capability categories, not six peer “brains” that must all run. The current co-present health gate requires `prompt + llm`; other slots may become Noop according to their contracts. The default assembly provides more explicit assistance for local small models, while strong-model assemblies can stay thinner. Rule of thumb: **make facts explicit, keep interpretations as candidates, let the model express them**.

Chinese: [human-docs/00_VISION_AND_POSITIONING.md](../human-docs/00_VISION_AND_POSITIONING.md)
