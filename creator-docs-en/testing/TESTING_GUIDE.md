# Performance and soak testing guide (pre-release)

For integrators running **v2 blueprint / Monolith** benchmarks locally. Run commands from the **oclivenewnew** repo root; `-o` points at a kernel project that **`init`**’d and **`cargo build --release`**’s cleanly.

**See also:** [PERFORMANCE.md](../getting-started/PERFORMANCE.md) · [OCLIVE_CLI_GUIDE.md](../cli/OCLIVE_CLI_GUIDE.md) · [TEST_OUTPUT_SCHEMA.md](TEST_OUTPUT_SCHEMA.md)

---

## Three test types

| Type | Goal | Command (replace paths) | Duration | Pass criteria |
|------|------|-------------------------|----------|---------------|
| **Monolith matrix** | **4×3** tier × preset weld combos | `cargo run -p oclive-cli -- bench --matrix --release -o <monolith-project> --json > matrix.json` | **2–4 h** | `matrix.json` has 12 combos; fill p50 ms into [PERFORMANCE.md](../getting-started/PERFORMANCE.md) table |
| **Cold start** | First `/chat` after process spawn | `cargo run -p oclive-cli -- bench --cold-start --cold-start-runs 5 -o <project>` | **~30 min** | 5 runs without timeout; kernel exposes `--api` with `OCLIVE_HTTP_API_MOCK_LLM=1` |
| **Soak** | RSS / leak trend over time | `cargo run -p oclive-cli -- bench --soak --soak-duration 72 -o <project> --json` | **72h nominal** (CLI may accelerate locally) | **Final RSS ≤ first sample × 1.2** |

---

## Prerequisites

1. **Matrix:** `oclive init --monolith` (or any tree with `monolith.toml`); optional `--kernel-source` to this repo.  
2. **Cold start / soak:** `cargo run --release -- --api` works; mock LLM env recommended.  
3. **Role pack:** v2 example `distros/chat-pro/roles/mumu/pipeline.ocblueprint` (load reference only).

---

## Suggested order

1. Cold start (fast wiring check)  
2. Matrix (Monolith tuning)  
3. Soak (dedicated machine; shorten `--soak-duration` for local smoke)

---

## CI and other tests

| Command | Notes |
|---------|--------|
| `oclive test -o <project>` | `cargo check`, clippy, `pack validate` |
| `oclive test --json` | Schema: [TEST_OUTPUT_SCHEMA.md](TEST_OUTPUT_SCHEMA.md) |
| `oclive test --ci-parity` | Mirrors generated `ci.yml` |
| OOCP / `npm run test:unit` | [OVERVIEW.md](OVERVIEW.md) |

---

[中文](../../creator-docs/testing/TESTING_GUIDE.md)
