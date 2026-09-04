# Optional vision: open experimentation tooling (summary)

OCLive's essence remains a **minimal tool kernel plus six stable capability ports**. Its reference runtime can compose those pieces into local-first experimentation tooling: replace one slot implementation while reusing the same role, turn semantics, error boundaries, and test baseline. This is one supported assembly, not a mandatory platform form, and role packs are not the only possible integration surface. Architecture: [OCLIVE_ARCHITECTURE_OVERVIEW.md](../getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md).

Strong-model assemblies may stay thinner; small-model assemblies may use more explicit assistance. Experiments should compare reproducible outcomes, not pipeline complexity. The optional **dual-core runtime** is documented in [RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md](../../creator-docs/rfc/RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md) and does not redefine the minimal kernel.

**Model adaptation layer (roadmap)**: creator tooling may add a fine-tune workshop for packable LoRA/SFT adapters selected by the expert-model facility (`expert_routing` · `slot.lora.apply`). This is reference-assembly capability, not a kernel requirement. See [BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md](BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md) §5.

[中文](../../creator-docs/roadmap/VISION_OPEN_LAB.md)

**Already aligned in the main repo**: HTTP JSON-RPC remote host path, `plugin_backends` and extension docs, directory plugins and whole-shell bridge, open-source and multi-OS CI. See root [README.md](../../README.md) “roadmap status” and [DOCUMENTATION_INDEX.md](../getting-started/DOCUMENTATION_INDEX.md).

**Still on the roadmap**: deeper in-pack knowledge, launcher/market linkage, community site shape—see sibling roadmap docs.
