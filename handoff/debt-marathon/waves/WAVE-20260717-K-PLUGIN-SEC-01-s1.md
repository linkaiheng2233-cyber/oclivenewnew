# WAVE-20260717-K-PLUGIN-SEC-01-s1

> Plan: [`../long-plans/K-PLUGIN-SEC-01.md`](../long-plans/K-PLUGIN-SEC-01.md) · Previous: [s0](./WAVE-20260717-K-PLUGIN-SEC-01-s0.md)

## Summary

| Field | Value |
|------|-------|
| **Debt ID** | K-PLUGIN-SEC-01 |
| **Stage** | 1 · Opaque-origin embedded slot broker |
| **Branch** | `codex/k-plugin-sec-01` |
| **Date** | 2026-07-17 |
| **Base HEAD** | `3c130b587671c3425c07a780e31f1a75acd0adef` |
| **Status** | Locally verified · not pushed or merged |

## Delivered

- Embedded plugin iframes are script-only sandboxed opaque origins.
- Parent-side bridge authority is bound to the exact iframe `contentWindow` and host-selected plugin asset identity.
- Malformed identity claims, non-opaque origins, unregistered frames, and replayed request IDs fail closed.
- The generated Rust-injected bridge routes framed invokes through the parent and does not expose direct Tauri invoke to embedded slots.
- Negative unit coverage and a static release regression ratchet were added.

## Evidence

| Check | Result |
|------|--------|
| `npm run build:plugin-bridge` | **PASS** · generated kernel bridge asset refreshed |
| `npm run test:unit -w @oclive/desktop-shared` | **PASS** · 18 files / 67 tests |
| `node scripts/verify-frontend-patches.mjs` | **PASS** · sandbox and source-bound broker ratchets |
| `npm run build -w @oclive/chat-pro` | **PASS** · 784 modules transformed |
| `npm run check:debt-marathon` | **PASS** · 11 auto plans |
| `git diff --check` | **PASS** |

## Honest boundary

- Event subscriptions in isolated frames are intentionally unavailable; the current union event list cannot establish per-plugin authority safely.
- The build still contains the existing `vue3-sfc-loader` production chunk (1,860.48 kB); Stage 2 owns its removal and official Voice HTML parity.
- Full-shell WebView isolation, native WebView evidence, verified installation identity, revocation, and exact-commit remote CI remain open.

## Next

K-PLUGIN-SEC-01 Stage 2 · official HTML parity and release compiler removal.

## GATES §6

- [x] One debt and one stage only; no full-shell or signing scope was claimed.
- [x] Security defaults fail closed; no `allow-same-origin` or direct embedded Tauri fallback was introduced.
- [x] Local checks are recorded as local evidence, not as remote CI or Done.
- [x] No push, merge, or TECHNICAL_DEBT closure was performed.
