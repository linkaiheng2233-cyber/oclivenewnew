#!/usr/bin/env bash
# Check internal relative links in creator-docs/**/*.md (repository root).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
exec python3 "$(dirname "$0")/check_doc_links.py" "$@"
