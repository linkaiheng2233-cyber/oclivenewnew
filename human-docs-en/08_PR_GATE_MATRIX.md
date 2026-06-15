# PR gate matrix (local)

> Full flow: [CONTRIBUTING.en.md](../CONTRIBUTING.en.md#tests-before-merge)

| Change type | Run locally | Optional |
|-------------|-------------|----------|
| Docs only | Link check; CHANGELOG → `node scripts/check-changelog-parity.mjs` | — |
| Frontend (`src/`) | `npm run test:unit` · `npm run build` | `npm run test:e2e:preview` (Linux/macOS) |
| Kernel / orchestration | `npm run check` · `npm run check:release` if HTTP/persist | `node scripts/dimension5-acceptance.mjs --ci` |
| Tauri API | `cargo test -p oclivenewnew-tauri` | `npm run test:e2e:core-api-restart` |
| `Cargo.lock` | `node scripts/dimension5-acceptance.mjs --ci` | `cargo audit` |

```bash
npm run check              # daily
npm run check:release      # release / engine
npm run check:ci-local     # dimension5 + check
```

Chinese: [human-docs/08_PR_GATE_MATRIX.md](../human-docs/08_PR_GATE_MATRIX.md)
