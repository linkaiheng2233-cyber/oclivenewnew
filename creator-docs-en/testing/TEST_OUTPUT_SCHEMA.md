# Test output contracts (TEST_OUTPUT_SCHEMA)

## `oclive test --json`

**Schema:** [`kernel/crates/oclive-cli/schemas/oclive_test_report.schema.json`](../../kernel/crates/oclive-cli/schemas/oclive_test_report.schema.json)

**Shape:** `schema_version`, `summary` (`passed` / `failed` / `skipped`), `suites[]` (`name`, `status`, optional `duration_ms`, `detail`), `failures[]` (`suite`, optional `file` / `line`, `error`).

**CI:** `oclive test -o . --json | jq '.summary.failed'` must be `0`; non-zero exits with code **1**.

**Note:** `oclive test --ci-parity --json` still emits the legacy job list format.

---

## Tauri / Rust integration tests

- Chat DTOs: **`oclive_kernel_types`**; assistant text field is **`reply`**.
- Fixtures: prefer `distros/desktop-tauri/tests/fixtures/`.

## Frontend

- `npm run test:unit` + `npm run build`; Ubuntu CI also runs Playwright preview E2E.

## HTTP `--api`

- `POST /chat` success body includes **`reply`**.

## Other schemas

| Command | Schema file |
|---------|-------------|
| `oclive bench --json` | `oclive_bench_report.schema.json` |
| `oclive doctor --json` | `oclive_doctor_report.schema.json` |

## Module behavior-quality reports

[`MODULE_QUALITY_HARNESS.md`](./MODULE_QUALITY_HARNESS.md) defines separate fixture observations, single-configuration `report` output, and multi-configuration `comparison` output:

- `report.dimensions.memory|emotion|prompt|llm` each contain `passed`, `total`, and `score`; no aggregate score is emitted.
- `comparison.quality.configurations[]` retains each four-module identity set, observation digest, case summary, and four dimension scores.
- `comparison.performance.status` is currently `not_measured` and `metrics` is empty; behavior quality never masquerades as a performance report.
- SHA-256 digests bind the suite, observations, and comparison. Compared suite digests must match exactly.

---

[中文](../../creator-docs/testing/TEST_OUTPUT_SCHEMA.md)
