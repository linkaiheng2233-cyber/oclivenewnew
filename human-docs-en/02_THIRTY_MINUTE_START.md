# 02 · Thirty-minute start

> Human onboarding — **not** [AGENTS.md](../AGENTS.md) (that file is for AI assistants).

## Prerequisites

- **Node.js ≥ 20** (see root `package.json` `engines`)
- **Rust** stable
- **Windows:** MSVC Build Tools — [10_SETUP_WINDOWS.md](10_SETUP_WINDOWS.md)

## Run

```bash
npm install
npm run tauri:dev    # desktop
# or
npm run dev          # frontend only
```

## Verify

| Level | Command | When |
|-------|---------|------|
| Daily | `npm run check` | Every PR |
| Release / engine | `npm run check:release` | Orchestration, persistence, HTTP |
| Local CI subset | `npm run check:ci-local` | dimension5 + check |

Full matrix: [08_PR_GATE_MATRIX.md](08_PR_GATE_MATRIX.md) · [CONTRIBUTING.en.md](../CONTRIBUTING.en.md)

Chinese: [human-docs/02_THIRTY_MINUTE_START.md](../human-docs/02_THIRTY_MINUTE_START.md)
