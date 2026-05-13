# Minimal kernel_runtime closure sanity check (no default features).
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root
cargo check -p oclive_kernel_runtime --no-default-features
