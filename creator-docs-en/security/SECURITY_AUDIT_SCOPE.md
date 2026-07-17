# Security audit scope and limitations (SECURITY_AUDIT_SCOPE)

This document states **what security-related work exists in this repo today** and **what is intentionally out of scope**, to avoid over-claiming. Implementation detail remains source code and CI.

**Related**: [KNOWN_VULNERABILITIES.md](./KNOWN_VULNERABILITIES.md) · [LIGHTWEIGHT_PROFILE.md §6.4](../../creator-docs/development/LIGHTWEIGHT_PROFILE.md) · root [AGENTS.md](../../AGENTS.md)

---

## Completed in this round (engineering)

- **`unsafe` blocks**: full inventory with comments (concurrency and invariants); see `# Safety` / module headers in `distros/desktop-tauri/src/**/*.rs`.
- **Cancellation and concurrency**: `process_message` path, `PluginHost` resolution, **cancellable LLM** (e.g. `llm_cancelable` modules)—**lock ordering**, `.await` boundaries, and cancel semantics are documented in source comments and key module headers.
- **`cargo audit`**: run regularly; **vulnerability-level** hits are tracked in [KNOWN_VULNERABILITIES.md](./KNOWN_VULNERABILITIES.md).
- **Concurrency review**: targeted review (not formal verification) of **`Arc` / `Mutex` / `JoinHandle`** and **async cancellation** on the main orchestration path.
- **Local HTTP API**: every route except `/health` requires `OCLIVE_API_TOKEN` by default; startup fails closed without a token unless `OCLIVE_API_ALLOW_UNAUTHENTICATED=1` is explicit.
- **Untrusted paths**: role/scene/directory-plugin IDs, role asset paths, and role-pack ZIP extraction now use single-segment validation, containment, and Windows-path regression tests.
- **Minimum plugin-UI isolation**: release builds no longer compile directory-plugin Vue into the host WebView. Unsafe inline Vue requires both a Vite dev build and `VITE_OCLIVE_UNSAFE_INLINE_PLUGIN_VUE=1`.
- **Credential scan**: a high-confidence scan found an API key present since the initial commit. The working tree is clean, but provider-side revocation and optional history cleanup remain P0 (K-SECRET-01).

---

## Not covered (known gaps)

- **Third-party supply chain**: no systematic audit of crate **author reputation, release history, reproducible builds**, etc.
- **Miri**: not full Miri over all `unsafe`; feasibility assessed only on critical paths.
- **Fuzzing**: no `cargo-fuzz` / `proptest` infrastructure.
- **Side channels**: no timing/power side-channel analysis.
- **Threat modeling (STRIDE, etc.)**: no full-product model; only concurrency/cancel-oriented review on the **main dialogue orchestration** path.
- **Strong plugin isolation**: HTML fallbacks still share the `https://ocliveplugin.localhost` origin; per-plugin origins and native iframe E2E are not complete, and signature strict mode remains opt-in. Disabling inline Vue in releases is containment, not a completed sandbox.
- **Historical credentials**: plaintext is gone from the working tree, but old commits remain readable. The incident is not closed until the provider revokes the old credential.

---

## Third-party risk (models, plugins, user data)

Engineering work above does **not** cover licensing of user-downloaded model weights, third-party plugin code, or compliance of user-configured Remote egress endpoints. Until signing and per-plugin origins are complete, third-party plugins are not trusted code; release builds only ensure they do not inherit host-page authority through inline Vue. **Product-facing legal boundaries** are in [DISCLAIMER.md](../legal/DISCLAIMER.md) (Chinese canonical: [`creator-docs/legal/DISCLAIMER.md`](../../creator-docs/legal/DISCLAIMER.md)).

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
| 2026-07-17 | Added HTTP auth, path containment, inline-Vue fail-closed behavior, historical credential incident, and shared-origin limitation. |
| 2026-05-15 | Added “Third-party risk” section linking to `legal/DISCLAIMER.md`. |
| 2026-05-13 | First version: defined completed scope and known gaps. |

---

[中文](../../creator-docs/security/SECURITY_AUDIT_SCOPE.md)
