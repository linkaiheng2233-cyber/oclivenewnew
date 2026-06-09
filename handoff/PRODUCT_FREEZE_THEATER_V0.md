# Product freeze — Theater v0 validated

**Effective:** 2026-06-09 (Opus 4.8 plan Phase 0)

Until strangers say 「卧槽」 on **AI Theater v0**, the kernel **does not expand**:

- No new `process_message` orchestration stages
- No new six-slot backends
- No blueprint v3 scheduling DSL
- **`dual_core`** remains frozen (opt-in beta, default off)
- **`expert_routing`** remains frozen

**Current product spear:** `distro_id=theater` · breakfast scene · dual contrast roles · 3 poke chips.

**Deferred unchanged:** `D-PORT-02`, `D-SLOT-01`, `K-PERF-10` (chat chrome eager load), §3.1 library API — see §Phase 5 thaw below.

**In-flight pre-research only:** `examples/reply-post-process-polish/` — theater local beat patch tech; **not** a substitute for Theater v0 product delivery. Scope closed per [REPLY_POST_PROCESS_POLISH_SCOPE.md](./REPLY_POST_PROCESS_POLISH_SCOPE.md).

---

## Phase 5 — thaw criteria (no action until user feedback)

| ID | Item | Thaw when |
|----|------|-----------|
| K-PERF-10 | Chat chrome lazy load | Theater first-screen perf fails acceptance |
| D-PORT-02 | Port consolidation | External contributor or second mature plugin backend |
| D-SLOT-01 | Slot merge scheduling | Same as D-PORT-02 |
| §3.1 | Pure library API | Second host strong demand + RFC |
| dual_core | Experimental pipeline | **No thaw** unless major release decision |
