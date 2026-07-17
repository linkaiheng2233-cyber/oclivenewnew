# WAVE-20260717-K-PLUGIN-SEC-01-s3

> Plan: [`../long-plans/K-PLUGIN-SEC-01.md`](../long-plans/K-PLUGIN-SEC-01.md) · Previous: [s2](./WAVE-20260717-K-PLUGIN-SEC-01-s2.md)

## Summary

| Field | Value |
|------|-------|
| **Debt ID** | K-PLUGIN-SEC-01 |
| **Stage** | 3 · Full-shell isolation and capability narrowing |
| **Branch** | `codex/k-plugin-sec-01` |
| **Date** | 2026-07-17 |
| **Base HEAD** | `22a207ae` |
| **Status** | Implemented · non-native gates locally verified · native WebView evidence pending |

## Delivered

- Replaced same-WebView `location.replace(shellUrl)` with a full-viewport opaque-origin `sandbox="allow-scripts"` frame whose calls are handled by the trusted parent broker.
- Removed `plugin-shell-remote.json`; custom-protocol plugin pages no longer receive a remote Tauri capability, and the main CSP no longer permits custom-protocol `connect-src`.
- Bound full-shell authority to the canonical `shellUrl` plugin/asset identity and verified that the entry is readable before suppressing the built-in UI.
- Added a one-time cryptographic frame-binding token. A second load/navigation revokes the registration, preventing another custom-protocol plugin page from inheriting the first plugin's broker identity.
- Applied the same load binding to embedded slots, added replay/navigation negative tests, regenerated the Rust-injected bridge asset, and added a static regression ratchet.
- Repaired the native Playwright command, which previously prepended `testDir` twice and enumerated zero tests. It now enumerates the existing main-window smoke and the new plugin-isolation case.
- Updated Chinese/English plugin contracts, bridge reference, Tauri migration inventory, and both minimal examples to match the isolated release path.

## Evidence

| Check | Result |
|------|--------|
| `npm run test:unit` | **PASS** · shared 18 files / 70 tests; Chat Pro 21 files / 83 tests |
| `npm run build` | **PASS** · generated bridge + Chat Pro; 784 modules transformed |
| `cargo test --locked -p oclivenewnew-tauri --tests` | **PASS** · Tauri unit and integration test binaries |
| `node scripts/dimension5-acceptance.mjs --ci` | **PASS** · 24 checks |
| docs / drift / registry / marathon gates | **PASS** |
| targeted ESLint + `git diff --check` | **PASS** |
| native Playwright enumeration | **PASS** · 2 tests found |
| `npm run test:e2e:tauri-native -- --grep "plugin isolation"` | **SKIP** · local Windows host has no `tauri-driver`; not counted as PASS |

## Review findings resolved

- A distinct child WebView was rejected because Tauri treats the registered custom protocol as a local origin; without an application-wide custom-command ACL it would still expose broad host IPC. The opaque frame keeps Tauri initialization main-frame-only and retains a single broker authority path.
- Source binding alone did not cover same-frame navigation between plugin paths. The one-time token plus second-load revocation closes that confused-deputy gap.
- The minimal HTML shell invoked the bridge before the injected bridge script at `</body>` existed. Both copies now wait for `load`; the bridge queues until the parent binding arrives.

## Honest boundary

- This Stage is **not Locally verified** under the L-level plan because the native isolation test was enumerated but skipped. The plan stays at `currentStage: 3`; K-PLUGIN-SEC-01 remains Partial.
- The exact native test must run under Linux CI/`tauri-driver` and prove the plugin frame cannot access parent DOM or `__TAURI_INTERNALS__`, while broker invocation still succeeds.
- Verified installation identity, signature rotation/revocation, and exact-commit remote CI remain Stage 4–5 work.

## Next

Run `npm run test:e2e:tauri-native -- --grep "plugin isolation"` in the prepared Linux native-E2E environment. On PASS, mark Stage 3 Locally verified and advance the plan to Stage 4.

## GATES §6

- [x] One debt and one implementation stage only; signature/identity work was not claimed.
- [x] Security review findings were fixed without restoring same-process Vue or custom-protocol remote IPC.
- [x] Applicable non-native checks are recorded separately from the skipped native check.
- [x] Plan/queue/TECHNICAL_DEBT were not advanced beyond available evidence.
- [x] No push, merge, or TECHNICAL_DEBT closure was performed.
