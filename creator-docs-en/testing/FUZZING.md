# Fuzzing (AB5)

[中文](../../creator-docs/testing/FUZZING.md)

## Goal

Randomly mutate **OOCP-shaped JSON**, **manifest.json**, **settings.json**, and other external inputs so parsing paths **never panic**.

## Method A: `proptest` (default, CI `fuzz` job)

```bash
cargo test -p oclive_validation --test proptest_fuzz_parsing
```

Each property case defaults to **2048** mutations (raise `ProptestConfig::with_cases` locally).

## Method B: `cargo-fuzz` (libFuzzer, needs nightly)

```bash
rustup toolchain install nightly
cargo install cargo-fuzz
cd fuzz
cargo fuzz list
cargo fuzz run fuzz_manifest_load -- -runs=100000
cargo fuzz run fuzz_settings_parse -- -runs=100000
cargo fuzz run fuzz_oocp_message -- -runs=100000
cargo fuzz run fuzz_blueprint_v2 -- -runs=100000
cargo fuzz run fuzz_oclive_validation -- -runs=100000
cargo fuzz run fuzz_function_call_parser -- -runs=100000
cargo fuzz run fuzz_role_pack_loader -- -runs=100000
```

### `fuzz_oclive_validation`

Random UTF-8 / JSON through **`validate_blueprint_v2_json`**, **`validate_manifest_top_level_keys`**, **`validate_settings_top_level_keys`** — assert no panic.

### `fuzz_function_call_parser`

Random strings through **`parse_from_llm_response`** (OpenAI `tool_calls` / `function_call`) — assert no panic.

### `fuzz_role_pack_loader`

Random bytes to temp file → **`peek_role_pack_manifest`** (ZIP / corrupt input) — assert no panic.

CI **`fuzz`** job runs proptest then **256** libFuzzer smoke rounds (`continue-on-error`).

## Reproducing crashes

1. libFuzzer leaves minimal input in `kernel/fuzz/artifacts/<target>/`.
2. Replay via `cargo test -p oclive_validation --test proptest_fuzz_parsing -- --exact <case>` or unit fixtures.

## Relation to `oclive test`

No dedicated subcommand; before release run full **proptest** + optional overnight **cargo-fuzz**.
