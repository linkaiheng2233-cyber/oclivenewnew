# Security audit scope and limitations (SECURITY_AUDIT_SCOPE)

This document states **what security-related work exists in this repo today** and **what is intentionally out of scope**, to avoid over-claiming. Implementation detail remains source code and CI.

**Related**: [KNOWN_VULNERABILITIES.md](./KNOWN_VULNERABILITIES.md) · [LIGHTWEIGHT_PROFILE.md §6.4](../../creator-docs/development/LIGHTWEIGHT_PROFILE.md) · root [AGENTS.md](../../AGENTS.md)

---

## Completed in this round (engineering)

- **`unsafe` blocks**: full inventory with comments (concurrency and invariants); see `# Safety` / module headers in `src-tauri/src/**/*.rs`.
- **Cancellation and concurrency**: `process_message` path, `PluginHost` resolution, **cancellable LLM** (e.g. `llm_cancelable` modules)—**lock ordering**, `.await` boundaries, and cancel semantics are documented in source comments and key module headers.
- **`cargo audit`**: run regularly; **vulnerability-level** hits are tracked in [KNOWN_VULNERABILITIES.md](./KNOWN_VULNERABILITIES.md).
- **Concurrency review**: targeted review (not formal verification) of **`Arc` / `Mutex` / `JoinHandle`** and **async cancellation** on the main orchestration path.

---

## Not covered (known gaps)

- **Third-party supply chain**: no systematic audit of crate **author reputation, release history, reproducible builds**, etc.
- **Miri**: not full Miri over all `unsafe`; feasibility assessed only on critical paths.
- **Fuzzing**: no `cargo-fuzz` / `proptest` infrastructure.
- **Side channels**: no timing/power side-channel analysis.
- **Threat modeling (STRIDE, etc.)**: no full-product model; only concurrency/cancel-oriented review on the **main dialogue orchestration** path.

---

## Rolling follow-ups

1. **Each feature cycle**: run `cargo audit` and update [KNOWN_VULNERABILITIES.md](./KNOWN_VULNERABILITIES.md).
2. **Miri**: introduce an **allow-fail** Miri CI job, expanding from the **smallest `unsafe` closures** outward.
3. **Fuzzing**: evaluate `proptest` or `cargo-fuzz` for **protocol parsing**, **prompt assembly edges**, **untrusted JSON**, etc.
4. **Tauri / gtk-rs warning cluster**: track *unmaintained* items in [KNOWN_VULNERABILITIES.md](./KNOWN_VULNERABILITIES.md); converge with **major Tauri** upgrades.

---

## Revision history

| Date | Notes |
|------|--------|
| 2026-05-13 | First version: defined completed scope and known gaps. |
