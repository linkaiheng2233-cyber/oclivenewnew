# Orchestration pack · Model Tier (EN summary)

> Full checklist (ZH): [`human-docs/modules/orchestration/model-tier.md`](../../../human-docs/modules/orchestration/model-tier.md)
> Definition SSOT: [MODULE_MAP §12](../../../handoff/MODULE_MAP_AND_HANDOFF.md)

**You plug in**: **Not** a `plugin_backends` key · code `model_tier.rs` · `co_present` · `ModelTier` Small/Large heuristics · Deep Tier0 via `meta.deep_capsule_enabled` + `prompts/deep_capsule.txt` · `PersonaSource`.

**Do**: Ollama model-name Small/Large heuristics · Deep capsule vs FullCore switch · align with [DEEP_PROMPT_DISTILLATION](../../../handoff/DEEP_PROMPT_DISTILLATION.md).

**Don't**: Register ModelTier as a slot · runtime LLM prompt compression (prompt slot) · pick models in UI bypassing llm slot.

**Read next**: [turn-thinking](turn-thinking.md) · [slots/prompt](../slots/prompt.md) · [slots/llm](../slots/llm.md) · [TTFT_BENCHMARK](../../../handoff/TTFT_BENCHMARK.md).
