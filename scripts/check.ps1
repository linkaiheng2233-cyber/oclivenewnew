$ErrorActionPreference = "Stop"

Write-Host "== oclivenewnew check (Windows-stable) =="

# Work around intermittent LNK1104 on Windows (file locked/reset during concurrent linking).
$env:CARGO_BUILD_JOBS = "1"
$env:CARGO_INCREMENTAL = "0"
$env:RUST_TEST_THREADS = "1"

Write-Host "-- npm build"
npm run -s build

Write-Host "-- cargo fmt"
cargo fmt --all -- --check

Write-Host "-- cargo clippy"
cargo clippy --all-targets --all-features -- -D warnings

Write-Host "-- cargo test"
cargo test -q

Write-Host "OK"

