# PR gate matrix (local)

> Full flow: [CONTRIBUTING.en.md](../CONTRIBUTING.en.md#tests-before-merge)

| Change type | Run locally | Optional |
|-------------|-------------|----------|
| Docs only | Link check; CHANGELOG → `node scripts/check-changelog-parity.mjs` | — |
| Public Base / minimal-role development feedback | `npm run check:kernel-base` (OCLive source root) | Batch acceptance still follows actual impact; public Rust API changes also require `cargo test --workspace --doc` |
| Complete contracts/types and compatibility checks | `npm run check:kernel-contracts` (source root) | Shared-consumer/Host/distro checks and downstream impact still apply |
| Frontend (`distros/shared/` + `distros/chat-pro/`) | `npm run test:unit` · `npm run build` | `npm run test:e2e:preview` (Linux/macOS) |
| Kernel / orchestration | `npm run check` · `npm run check:release` if HTTP/persist | `node scripts/dimension5-acceptance.mjs --ci` |
| Tauri API | `cargo test -p oclivenewnew-tauri` | `npm run test:e2e:core-api-restart` |
| `Cargo.lock` | `node scripts/dimension5-acceptance.mjs --ci` | `cargo audit` |

```bash
npm run check:kernel-base  # public foundation feedback, from OCLive source root
npm run check:kernel-contracts # complete two-crate contract/compatibility checks
npm run check              # daily
npm run check:release      # release / engine
npm run check:ci-local     # dimension5 + check
```

The local entry runs the existing Base fixture, minimal-role contract tests, Clippy for the two public crates, and explicit doctests. It requires Node (per package.json), Cargo, and Clippy, without preparing frontend or desktop resources. A failed step stops the chain with a nonzero exit. Each command is printed before execution; Cargo output appears when that step finishes, with a 20-minute limit per step. The [CI design](../creator-docs-en/roadmap/SOMEDAY_TOOLCHAIN_CI.md#kernel-host-ci-first-slice) owns the scope and limits.

For example, use this entry to check OCLive's public baseline while developing a Memory Base implementation, then run your own module tests. A generated scaffold still uses `cargo check` / `cargo test` in its own directory; this entry does not check external modules automatically. Consumer wiring, Host behavior, and milestone acceptance need their applicable checks. Narrow success does not replace full CI.

For the complete two-crate surface, `check:kernel-contracts` runs all-targets/all-features Clippy with `--no-deps`, then unfiltered default tests and applicable doctests, including existing compatibility tests. It has the same tool requirements, stop-on-failure behavior, and 20-minute per-step limit without Host/frontend preparation. The crates are not entirely small Kernel; complete checks for these two crates are not complete Kernel/workspace acceptance. See the [CI ownership table](../creator-docs-en/roadmap/SOMEDAY_TOOLCHAIN_CI.md#ci-responsibility-map).

The local workflow assigns these checks to `kernel-contracts` on Linux/Windows. `rust` retains frontend/desktop preparation, workspace formatting, and remaining workspace lint/tests, excluding only CLI and the two migrated roots. This configuration awaits formal exact-SHA CI after integration; Host/combination checks, required `ci-gate`, and Push/ready rules remain. Ownership does not authorize narrower selection.

Chinese: [human-docs/08_PR_GATE_MATRIX.md](../human-docs/08_PR_GATE_MATRIX.md)
