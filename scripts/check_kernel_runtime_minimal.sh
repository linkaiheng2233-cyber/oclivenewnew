#!/usr/bin/env sh
# Minimal kernel_runtime closure sanity check (no default features).
set -eu
cd "$(dirname "$0")/.."
cargo check -p oclive_kernel_runtime --no-default-features
