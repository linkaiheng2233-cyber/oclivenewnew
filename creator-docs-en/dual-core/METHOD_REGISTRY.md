# Experimental core method registry (`pipeline.experimental`)

[中文](../../creator-docs/dual-core/METHOD_REGISTRY.md)

Each step `action` in `pipeline.experimental` must be:

```text
slot.<registry_key>.<method>
```

- `registry_key`: instance key in this role’s `slot_registry` (e.g. `emotion`, `llm_2`).
- `method`: one of the methods below; **unlisted methods cause experimental core to error and silently fall back to stable core** (`co_present`).

Stable core **does not** interpret `pipeline.stable`; the host always uses the hard-coded `process_co_present` path.

---

## Current seven method types (six slots + the `complex_emotion` facility)

These are the method types currently executable through `PluginHost`, not seven stable slots. The Stable core contract still has six slots.

| `type` | `method` | Co-present stage | Description |
|--------|----------|------------------|-------------|
| `memory` | `retrieve` | `memory_rank` | Load recent memories, scene-weighted and ranked (`MemoryRetrievalInput`) |
| `emotion` | `analyze` | `user_emotion_analyze` | Analyze user-message emotion (`EmotionResult`) |
| `event` | `detect` | `event_estimate` | Estimate turn event type and impact (needs emotion/personality context) |
| `prompt` | `assemble` | `build_prompt` | Assemble main dialogue prompt string (no LLM call) |
| `llm` | `generate` | `llm_generate` | Marks turn for generation; after experimental chain, full `co_present` runs (with LLM) |
| `agent` | `process` | `agent_process` | Invoke Agent; if `handled`, return Agent reply directly |
| `complex_emotion` | `resolve_turn` | `complex_emotion_resolve_turn` | Invoke the provider as an experimental executability/failure-boundary check; current output is not committed, while Stable completion owns formal post-LLM resolution and persistence |

---

## Per-method notes

### `retrieve`

- **Input:** Current user message, `scene_id`, session `srid`, resolved `ResolvedRolePlugins`.
- **Output:** Ranked relevant memories (in-memory only; step does not write DB).
- **Example:** `slot.memory.retrieve`

### `analyze`

- **Input:** User message text.
- **Output:** `EmotionResult` (for later `detect` / `assemble` / `resolve_turn`).
- **Example:** `slot.emotion.analyze`

### `detect`

- **Input:** User message, user emotion, personality vector, recent context, optional knowledge augment.
- **Output:** Event estimate (type, impact, confidence); may adjust personality vector (non-Profile mode).
- **Example:** `slot.event.detect`

### `assemble`

- **Input:** Personality, memory, relations, scene, emotion prompt sections, mutable personality profile, etc. (see `PromptInput`).
- **Output:** Prompt string (cached in experimental context; stable core `generate` still uses it).
- **Example:** `slot.prompt.assemble`

### `generate`

- **Input:** No extra params; indicates this turn must complete via LLM.
- **Output:** After experimental success, invokes full `co_present` (same reply contract as single-core today).
- **Example:** `slot.llm.generate`
- **Note:** Experimental pipeline **must** include at least one `generate`, or Agent `process` must short-circuit successfully.

### `process`

- **Input:** `AgentInput` (role_id, session, message, model).
- **Output:** If Agent `handled`, return `SendMessageResponse` directly; else continue pipeline.
- **Example:** `slot.agent.process`

### `resolve_turn`

- **Input:** `ComplexEmotionInput` (prior turn dialogue, `previous_narrative_hint`, seven-dimension affect metrics, etc.).
- **Output:** The provider returns `ComplexEmotionOutput`, but the frozen v3 Beta runner currently discards that value and this method does **not** write a hint. When the experiment requests `generate`, the complete Stable `co_present` path reruns formal `[EMO]` / plugin-fallback resolution and persistence. Treat this as a current limitation, not a state-authority contract.
- **Example:** `slot.complex_emotion.resolve_turn`

---

## CLI queries

```bash
cargo run -p oclive-cli -- explain DUAL_CORE
cargo run -p oclive-cli -- explain slot.emotion.analyze
```

---

## Related docs

- [DEVELOPER_GUIDE.md](./DEVELOPER_GUIDE.md)
- [RFC dual-core dual-mode](../../creator-docs/rfc/RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md)
- [handoff/DUAL_CORE_CURSOR_HANDOFF.md](../../handoff/DUAL_CORE_CURSOR_HANDOFF.md)
