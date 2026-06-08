# oclive_sqlx

SQLite-only SQLx facade for OCLive. **Supply-chain gate crate** — keeps `sqlx-mysql` and transitive `rsa` out of `Cargo.lock`.

## Why this crate exists

The umbrella `sqlx` meta-crate pulls MySQL (and `rsa`) even when the product only uses SQLite. `oclive_sqlx` depends on `sqlx-core` + `sqlx-sqlite` only.

## Rules for contributors

- **Do not** add a direct workspace dependency on the `sqlx` meta-crate.
- Add database access through `oclive_sqlx` re-exports or this crate's API.
- Any PR that changes `Cargo.lock` must pass `cargo audit` (see root `scripts/dimension5-acceptance.mjs`).

## Verification

```bash
cargo tree -p oclive_sqlx
cargo test -p oclive_sqlx --test lockfile_guard
cargo audit --no-fetch --stale
```

`Cargo.lock` must not contain `name = "sqlx-mysql"` or `name = "rsa"`.
