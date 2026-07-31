# WAVE-20260717-K-PLUGIN-SEC-01-s2

> Plan: [`../long-plans/K-PLUGIN-SEC-01.md`](../long-plans/K-PLUGIN-SEC-01.md) · Previous: [s1](./WAVE-20260717-K-PLUGIN-SEC-01-s1.md)

## Summary

| Field | Value |
|------|-------|
| **Debt ID** | K-PLUGIN-SEC-01 |
| **Stage** | 2 · Official HTML parity and release compiler removal |
| **Branch** | `codex/k-plugin-sec-01` |
| **Date** | 2026-07-17 |
| **Base HEAD** | `c25bf69b` |
| **Status** | Locally verified · not pushed or merged |

## Delivered

- Replaced the official Voice toolbar/settings placeholders with functional release HTML for recording, transcription, submit/fill routing, profiles, probes, persistence, model/adapter import, and TTS warm-up.
- Extended the source-bound frame broker with plugin-namespace-only custom event emit/subscribe. Built-in host events remain unavailable to isolated frames without per-plugin authority.
- Granted microphone permission only to the official Voice toolbar asset; its settings frame and all other embedded frames retain the script-only sandbox without microphone access.
- Bound `get_plugin_settings_ui` / `set_plugin_settings_config` to the authenticated bridge plugin identity, closing a cross-plugin settings confused-deputy path.
- Moved `vue3-sfc-loader` to development-only dependencies and added a production `import.meta.env.DEV` cut. The release bundle no longer contains the 1,860.48 kB loader chunk or loader string.
- Added behavioral jsdom tests that execute both official HTML scripts against a fake isolated bridge, plus broker namespace/subscription negative tests and static release ratchets.

## Evidence

| Check | Result |
|------|--------|
| `npm run test:unit` | **PASS** · shared 18 files / 69 tests; Chat Pro 21 files / 81 tests |
| `npm audit --omit=dev --audit-level=high` | **PASS** · 0 vulnerabilities |
| `npm run build` | **PASS** · bridge + Chat Pro; 784 modules transformed |
| release output scan | **PASS** · no `vue3-sfc-loader` string or chunk |
| `node scripts/verify-frontend-patches.mjs` | **PASS** · 11 static UI/security checks |
| `cargo test -p oclivenewnew-tauri --lib settings_commands_reject_cross_plugin_identity` | **PASS** · 1 targeted test |
| `npm run check:debt-marathon` | **PASS** · 11 auto plans |
| `git diff --check` | **PASS** |

## Diagnostic note

An initial `cargo test -p oclivenewnew-tauri <filter>` invocation compiled successfully but enumerated integration binaries and exceeded the 180-second command budget, ending with a closed pipe. It was not treated as a pass. The corrected `--lib` targeted command completed and passed.

## Honest boundary

- jsdom proves bridge behavior and UI flow, but microphone permission and custom-protocol subresource loading still need the Stage 3 native WebView test.
- The microphone grant is currently keyed to the official plugin ID; Stage 4 must bind that identity to a verified installation before production trust can close.
- Unsafe inline Vue remains an explicit development-only escape hatch. It is absent from the release dependency graph, not deleted from local diagnostics.
- Full-shell WebView isolation, capability narrowing, signature/revocation, and exact-commit remote CI remain open.

## Next

K-PLUGIN-SEC-01 Stage 3 · full-shell WebView isolation and capability narrowing.

## GATES §6

- [x] One debt and one implementation stage only; no full-shell or signature completion claimed.
- [x] Official functionality was restored through the isolated broker rather than by weakening the sandbox.
- [x] Production audit/build evidence is separate from development-only dependency availability.
- [x] No push, merge, or TECHNICAL_DEBT closure was performed.
