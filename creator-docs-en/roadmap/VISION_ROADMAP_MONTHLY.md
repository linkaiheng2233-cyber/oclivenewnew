# oclive vision delivery · monthly plan

This document breaks down the platform vision—**open platform + dual apps + role packs + swappable memory/emotion + optional multilingual plugins**—into **monthly, shippable milestones**. Order may shift with staffing, but **contracts before implementation, default implementations before real plugins** stays fixed.

**Product launch (P0)**: desktop host gaps and task buckets live in **[`handoff/archive/PRODUCT_AND_KERNEL_GAP_CHECKLIST.md`](../../handoff/archive/PRODUCT_AND_KERNEL_GAP_CHECKLIST.md)** and **[`handoff/PRODUCT_LINE_TASK_BUCKETS.md`](../../handoff/PRODUCT_LINE_TASK_BUCKETS.md)**; release sign-off in **[`handoff/archive/PRODUCT_RELEASE_CHECKLIST.md`](../../handoff/archive/PRODUCT_RELEASE_CHECKLIST.md)**.

[中文](../../creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md)

---

## Vision pillars (reference)

| Pillar | Meaning | Plan items |
|--------|---------|------------|
| Open | Not chasing a single SOTA point—**replaceable, documented, versioned** subsystems | Contract docs, trait boundaries, open-source readiness |
| Dual apps | **Runtime (player)** vs **creator tools**, **role pack** as the only handoff | Pack spec, editor, README split |
| Role as workflow | Each pack is declarative config + optional backends | manifest extensions, `min_runtime`, backend enums |
| Swappable memory / emotion | Seven dimensions are **current defaults**, not platform limits | Memory/Emotion facades, second implementations, future sidecars/WASM |
| **Soul weight layer** | Speech habits and tone can ship as **LoRA/SFT adapters** alongside prompt/memory; **expert-model facility** switches them at runtime (`slot.lora.apply`), not a closed “personality engine” | Fine-tune workshop (standalone creator tool), pack satellite adapter files, `expert_routing.json`, directory inference plugins |

---

## Month 1: contracts and code boundaries (foundation) — **aligned with current code**

**Goal**: Clarify what can be swapped without changing product behavior.

| Deliverable | Notes |
|-------------|-------|
| `creator-docs/plugin-and-architecture/PLUGIN_V1.md` | Subsystem DTOs, `settings.json` enums; orchestration order vs `chat_engine` / `PluginHost`. |
| `creator-docs/role-pack/PACK_VERSIONING.md` | Pack version, `schema_version`, `min_runtime_version`, unknown-field policy. |
| Rust facades | **`PluginHost`**: Memory, Emotion, Event, Prompt, LLM traits; orchestration only in the main path. |
| `settings.json` | Nested **`plugin_backends`** (`memory` / `emotion` / `event` / `prompt` / `llm` / `agent`); see `plugin_backends.rs`. |

**Acceptance**: Full `cargo test`, `npm run build`; dialogue and favor behavior unchanged unless explicitly documented.

---

## Month 2: role pack editor MVP

**Goal**: Creators ship loadable packs **without hand-editing JSON**.

| Deliverable | Notes |
|-------------|-------|
| Editor shape | Standalone app or in-app creator mode; prefer **standalone**. |
| Scope | manifest + basic `settings.json`, **same validation as runtime**. |
| Export | `distros/chat-pro/roles/{id}/` or zip per `distros/chat-pro/roles/README_MANIFEST.md`. |

**Acceptance**: New/edit pack via editor loads in oclive and chats normally.

---

## Month 3: prove replaceability — second built-in implementation

**Goal**: Minimal second backend to validate trait/config wiring (not SOTA quality).

| Deliverable | Notes |
|-------------|-------|
| Second Memory or Affect | e.g. simplified FIFO / tag filter or emotion passthrough; **must use real enum paths**. |
| Editor | Optional picker for the second backend. |
| Regression | Default backend unchanged; switch path tested. |

**Acceptance**: Same pack, only `*_backend` changed → **measurable difference** (logs or fixed fixtures).

---

## Month 4: external plugin protocol draft + engineering

**Goal**: Formal hook for multilingual plugins; ship **one** host invocation style first.

| Deliverable | Notes |
|-------------|-------|
| Protocol draft | Subprocess JSON-RPC or gRPC; version, timeout, error codes documented. |
| Pilot | Memory sidecar first; emotion may stay built-in. |
| Security | No arbitrary execution by default; manifest declares paths/URLs; user consent documented. |
| CI / OSS | LICENSE, README, `.github/workflows/ci.yml` (already in repo). |

**Acceptance**: Minimal external demo plugin completes one mock retrieve/write round-trip.

---

## Month 5: in-pack knowledge carrier + retrieval hook

**Goal**: Pre-authored answers ship with the pack and version with it.

| Deliverable | Notes |
|-------------|-------|
| Pack layout | e.g. `knowledge/` + manifest reference. |
| Runtime | Pre-turn retrieval/injection (keyword or vector—start lightweight). |
| Editor | Knowledge block editing tied to pack version. |

**Acceptance**: After pack upgrade, same question reflects **new authored content**.

---

## Month 6: dual-app narrative + optional launcher sketch

**Goal**: Public story matches repo layout; lower friction for non-developers.

| Deliverable | Notes |
|-------------|-------|
| Root README | Runtime vs editor, install, roles path. |
| Launcher (optional) | Ollama check, `OCLIVE_ROLES_DIR`, spawn runtime. |
| Extension index | `EXTENSION_POINTS.md`: stable traits, manifest fields, protocol version. |

**Acceptance**: New user can tell **play** vs **author packs** from README alone.

---

## Month 7+ (backlog)

| Direction | Notes |
|-----------|-------|
| WASM plugins | After process plugins stabilize. |
| Dynamic `.dll`/`.so` | Only with strong ABI need; not default. |
| Trophies / relation rituals, chat modes | Product-paced small iterations. |
| Ecosystem | Sample packs, template repos, `CONTRIBUTING.md`. |

### After three distros ship · Fine-tune workshop (creator toolchain phase 3)

**Positioning**: After **Chat Pro / VS Code Flash / AI Theater** engineering smoke passes and the pack editor’s simple-creator loop is usable, add the **weight layer**—soul is not only prompt/memory/relation, but also packable **LoRA/SFT adapters** for habits, pacing, and live-stream personas.

**Product rationale**: Vertical AI character work (e.g. AI streamers) shows prompt-only tuning is insufficient; OClive’s edge is **adapters as pack modules + expert routing at runtime**, not competing on “best memory/emotion engine.”

**Architecture hooks** (wired or pre-wired; mostly off until productized):

| Item | Notes |
|------|-------|
| **Facility #2** | Expert-model facility · `expert_routing.json` · conditional sub-pipeline |
| **`slot.lora.apply`** | Expert step: session `plugin_id` to switch adapter (`dual_core` / Experimental; not Stable main path until thaw) |
| **Module 5 `llm`** | Main chat stays on `plugin_backends.llm`; adapters default to expert sub-flow only |
| **Pack editor** | Exports `.ocpak` / `distros/chat-pro/roles/`; workshop writes satellite files (contract TBD in RFC) |

**Phased delivery (T0→T3)**:

| Phase | Deliverable | Acceptance |
|-------|-------------|------------|
| **T0 · contract** | RFC: corpus/privacy, `lora_adapters` schema, links to `expert_routing` / `slot.lora.apply`, export profiles | Doc review; validation key draft |
| **T1 · workshop MVP** | Standalone Tauri tool: import transcripts/samples → single-base LoRA → export into role pack | Validates via `oclive_validation`; loads under `distros/chat-pro/roles/{id}/` |
| **T2 · runtime** | Directory plugin or Ollama modelfile path; `slot.lora.apply` actually loads adapter | Observable diff on expert route hit (fixtures/logs) |
| **T3 · evaluation** | Extend bench/OOCP/replay: prompt-only vs LoRA vs LoRA+expert | Reproducible comparison report |

**Discipline**: training stays in the workshop/sidecar; kernel stays thin. During **expert_routing / dual_core freeze**, only T0+T1 on branches—no Stable main-path wiring. Not P0 before Theater stranger test; see [RECURRING_OPTIMIZATION_PLAYBOOK.md](../../handoff/RECURRING_OPTIMIZATION_PLAYBOOK.md) §9.

Details: [BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md](BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md) §5 · scenario **S11** in [APPLICATION_SCENARIOS.md](../../creator-docs/roadmap/APPLICATION_SCENARIOS.md).

Experience backlog (editor try-chat, launcher deps, marketplace): **[BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md](BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md)**.

---

## Monthly habits (recommended)

- **Contract changes** → docs + version; no silent field renames.
- **Default path always falls back** if a new backend fails.
- **Tests** cover trait switching and pack load at least once.

---

## `oclive-cli` scaffold (planned)

Not yet implemented in `oclive-cli`; do not mark as shipped in status overviews.

| Direction | Notes |
|-----------|-------|
| `pack diff` / `pack update` | Pack version diff and dependency checks |
| `oclive kernel update` | Align generated project kernel path deps |
| `dev --inject` | Hot-inject test messages + step trace |
| `bench history clear` / `export` / `import` | Benchmark history management |

---

## Doc index

- Pack contract: [distros/chat-pro/roles/README_MANIFEST.md](../../distros/chat-pro/roles/README_MANIFEST.md)
- Creator docs: [../role-pack/CREATOR_ROLE_PACK_CUSTOMIZATION.md](../role-pack/CREATOR_ROLE_PACK_CUSTOMIZATION.md)
- Experience backlog: [BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md](BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md)
- If this plan diverges from code, **code and validation win**—update this file.

---

*Updated with vision iterations; note date on major direction changes.*
