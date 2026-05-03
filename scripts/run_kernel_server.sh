#!/usr/bin/env bash
# 从仓库根启动无头 OOCP / HTTP 内核（见 crates/oclive_kernel_server）。
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
export OOCP_API_PORT="${1:-48888}"
exec cargo run -p oclive_kernel_server
