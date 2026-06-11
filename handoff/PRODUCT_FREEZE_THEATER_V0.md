# Product freeze — Theater v0 validated

**Effective:** 2026-06-09 (Opus 4.8 plan Phase 0)

Until strangers say 「卧槽」 on **AI Theater v0**, the kernel **does not expand**:

- No new `process_message` orchestration stages
- No new six-slot backends
- No blueprint v3 scheduling DSL
- **`dual_core`** remains frozen (opt-in beta, default off)
- **`expert_routing`** remains frozen

**Current product spear:** `distro_id=theater` · breakfast scene · dual contrast roles · 3 poke chips.

**Phase 4 engineering (2026-06-12):** Theater Release 打包链 · CI `test:theater:smoke` · 导演插件 RFC/示例/前端接线 · 陌生人测试主持人指南。**产品解冻仍待** 5 人真人 ≥60%（工程代理 100% 不替代）— 见 [`THEATER_STRANGER_FACILITATOR.md`](./THEATER_STRANGER_FACILITATOR.md)。

**Deferred unchanged:** `D-PORT-02`, `D-SLOT-01`, `K-PERF-10` (chat chrome eager load), §3.1 library API — see §Phase 5 thaw below.

**In-flight pre-research only:** `examples/reply-post-process-polish/` — theater local beat patch tech; **not** a substitute for Theater v0 product delivery. Scope closed per [REPLY_POST_PROCESS_POLISH_SCOPE.md](./REPLY_POST_PROCESS_POLISH_SCOPE.md).

**Approved exception (2026-06-10):** VS Code `POST /chat/stream` — backward-compatible delivery mode only; no new orchestration stages. See [VSCODE_STREAM_THEATER_GATE.md](./VSCODE_STREAM_THEATER_GATE.md).

---

## Phase 5 — thaw criteria (no action until user feedback)

| ID | Item | Thaw when |
|----|------|-----------|
| K-PERF-10 | Chat chrome lazy load | Theater first-screen perf fails acceptance |
| D-PORT-02 | Port consolidation | External contributor or second mature plugin backend |
| D-SLOT-01 | Slot merge scheduling | Same as D-PORT-02 |
| §3.1 | Pure library API | Second host strong demand + RFC |
| dual_core | Experimental pipeline | **No thaw** unless major release decision |
