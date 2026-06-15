# 00 · Vision and positioning

> **Reader:** Engineers contributing code.  
> **Time:** ~15 min.  
> **Next:** [01 Architecture](01_ARCHITECTURE_SIMPLE.md) or [02 Thirty-minute start](02_THIRTY_MINUTE_START.md).

**OCLive (A.I.Live)** is an open, local-first **AI character assembly platform**: **six swappable backend slots**, **role pack distribution**, and **contract validation** — ship your own character runtime in about 30 minutes.

Stack: **Tauri + Vue 3 + Rust**. Codename: **oclive**.

| Is | Is not |
|----|--------|
| Assembly + contracts + distribution | A fixed vertical “memory engine” product |
| Thin kernel + `PluginHost` six slots | Blueprint `steps[]` as first-turn scheduling DSL |
| Role pack (identity, prompts) vs blueprint (`slot_registry`) | Creator fields mixed into six slots |

Deep dive: [creator-docs-en/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md](../creator-docs-en/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md)

Chinese: [human-docs/00_VISION_AND_POSITIONING.md](../human-docs/00_VISION_AND_POSITIONING.md)
