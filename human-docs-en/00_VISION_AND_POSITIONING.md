# 00 · Vision and positioning

> **Reader:** Engineers contributing code.  
> **Time:** ~15 min.  
> **Next:** [01 Architecture](01_ARCHITECTURE_SIMPLE.md) or [02 Thirty-minute start](02_THIRTY_MINUTE_START.md).

**OCLive (A.I.Live)** is an open, local-first **AI-character tool kernel**. Kernel defines six-slot contracts and necessary legality boundaries; distro Hosts choose composition, scheduling, and result application. Slots are capabilities, not six fixed stages. Role runtime, memory systems, Event Ring, and persistence compose outside the small Kernel. See [MODULE_MAP](../handoff/MODULE_MAP_AND_HANDOFF.md#kernel-responsibilities); current code still ships the full reference runtime, not a physically extracted minimal core.

Stack: **Tauri + Vue 3 + Rust**. Codename: **oclive**.

| Is | Is not |
|----|--------|
| Tool kernel: six-slot contracts and necessary legality boundaries | Ownership of Host scheduling, domain commits, or product-success policy |
| Optional reference runtime and creator toolchain | A centralized platform whose full default stack is mandatory |
| `PluginHost` six slots and stable contracts | Blueprint `steps[]` as first-turn scheduling DSL |
| Role pack (identity, prompts) vs blueprint (`slot_registry`) | Creator fields mixed into six slots |

Deep dive: [creator-docs-en/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md](../creator-docs-en/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md)

The six slots are capability categories, not six peer “brains” that must all run. `event` estimates dialogue event impact; it is not Event Ring or Runtime Event Stream. The current reference Host's co-present health gate requires `prompt + llm`; other slots may become Noop according to their contracts. These are reference-Host strategies, not a universal pipeline.

A minimal role expresses persona + visual asset references; relations, favorability, blueprints, and slot registries are not prerequisites. Existing richer reference-pack examples describe supported authoring formats, not the minimal contract's required files. Local load preparation does not mean minimal-role lifecycle activation is wired.

Chinese: [human-docs/00_VISION_AND_POSITIONING.md](../human-docs/00_VISION_AND_POSITIONING.md)
