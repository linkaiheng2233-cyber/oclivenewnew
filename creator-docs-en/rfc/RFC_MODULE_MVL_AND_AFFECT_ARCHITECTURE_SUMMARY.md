# Module MVL & affect architecture — English summary

[中文](../../creator-docs/rfc/RFC_MODULE_MVL_AND_AFFECT_ARCHITECTURE.md)

Full RFC (Chinese SSOT): [RFC_MODULE_MVL_AND_AFFECT_ARCHITECTURE.md](../../creator-docs/rfc/RFC_MODULE_MVL_AND_AFFECT_ARCHITECTURE.md).

**Status:** **Draft**, revised 2026-09-03 for model-intelligence preservation (`display_metrics` and domain port layering landed; otherwise trust source and [MODULE_MAP_AND_HANDOFF.md](../../handoff/MODULE_MAP_AND_HANDOFF.md)).

## Core ideas

**MVL (Minimum Viable Loop)** = smallest capability per module that keeps co-present `send_message` runnable.

- **T0 = MVL** (stable contract; official builtin + `none` must satisfy)
- **T1+ = optional enhancements** (official defaults allowed; authors may replace or omit)

**Affect split (target architecture):**

- **Simulation evidence** (advises **`reply`**): `core_personality.txt` + `mutable_personality` + optional emotion T2 candidates in prompt — **not** numeric favor / trait scores and not a unique emotional truth
- **Display** (UI only): `display_metrics` JSON (`favor`, `traits[7]`, `relation_summary`) — **must not** be read by `PromptBuilder` mechanics

The reply model keeps semantic interpretation and natural expression. Target candidates carry source, confidence, TTL, and scope; the current `EmotionResult` remains mainly a seven-number distribution and does not yet implement those semantics uniformly. Legacy favor formulas and numeric `PersonalityEngine` evolution in prompt are **deprecated**, not the long-term kernel boundary.

## Whole-machine minimal path

```text
user message → emotion T0 or none → memory / identity / profile → prompt T0 → llm T0 → built-in post → reply + DTO
```

| Slot | Required for boot health | When `none` |
|------|--------------------------|-------------|
| **prompt** | **Yes** | Health check fails |
| **llm** | **Yes** | Health check fails |
| memory | No | Empty list |
| emotion | No | Neutral `EmotionResult` |
| event | No | `Ignore` / impact 0 |
| agent | No | No short-circuit |

**robot-soul / headless minimal pack:** at least `prompt` + `llm`; emotion T0 recommended but not a hard health gate.

## Relation to `none` semantics

- `plugin_backends.<slot> = none` → slot **does not participate** (Noop)
- `builtin` **T0** → slot participates with minimum behavior defined in this RFC

See [MODULE_NONE_SEMANTICS.md](../kernel/MODULE_NONE_SEMANTICS.md).

## Design goals

1. **Runs:** T0 (or `none`) on every slot still returns legal **`reply`**
2. **Replaceable:** plugins meet T0 trait to integrate; T1+ via optional capabilities
3. **Future-proof:** display metrics decoupled from prompt mechanics; numeric favor never T0-hard-required
4. **Author freedom:** remote / directory / stronger LLM / custom UI within minimal contract
5. **Preserve model intelligence:** explicit facts, candidate interpretations, model-owned expression. Rules and helper models do not become a second authoritative mind.

## Breaking note (§8 in full RFC)

Target architecture may break: numeric favor mechanics, default `vector` personality path, DTO semantics — see full RFC before implementing pack or plugin changes.

## Related

- [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md) · [MODULE_NONE_SEMANTICS.md](../kernel/MODULE_NONE_SEMANTICS.md)
- [personality-archive-notes.md](../../docs/personality-archive-notes.md)
- Module map: [MODULE_MAP_AND_HANDOFF.md](../../handoff/MODULE_MAP_AND_HANDOFF.md)
