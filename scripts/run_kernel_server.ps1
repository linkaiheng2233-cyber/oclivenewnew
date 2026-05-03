# 从仓库根启动无头 OOCP / HTTP 内核（见 crates/oclive_kernel_server）。
param(
    [int]$Port = 48888
)
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root
$env:OOCP_API_PORT = "$Port"
cargo run -p oclive_kernel_server
