# First PR recipe (English summary)

> Detailed tasks: [human-docs/07_COMMON_TASKS.md](../human-docs/07_COMMON_TASKS.md)

## Before you code

1. Read [02_THIRTY_MINUTE_START.md](02_THIRTY_MINUTE_START.md) — app runs locally.
2. Skim [04_ENGINEERING_RULES_SUMMARY.md](04_ENGINEERING_RULES_SUMMARY.md) — where logic may live.
3. Pick a scoped change (<500 lines); avoid `process_message` orchestration unless explicitly assigned.

## Typical first PR shapes

| Change type | Self-check |
|-------------|------------|
| Docs only | `node scripts/check-changelog-parity.mjs` if CHANGELOG touched |
| Frontend | `npm run test:unit` + `npm run build` |
| Kernel / Rust | `npm run check` or `cargo test -p oclive_kernel_host --lib` |
| `Cargo.lock` | `npm run check:ci-local` |

## Checklist

- [ ] CHANGELOG.md + CHANGELOG.en.md `[Unreleased]` parity if user-visible
- [ ] No new six-slot keys in role packs for “character” tasks
- [ ] Tauri `invoke` keys are **camelCase** on the frontend ([`distros/shared/src/api/`](../distros/shared/src/api/))
- [ ] Link PR to [08_PR_GATE_MATRIX.md](08_PR_GATE_MATRIX.md) gates in the description

Fork workflow: [CONTRIBUTING.en.md](../CONTRIBUTING.en.md).
