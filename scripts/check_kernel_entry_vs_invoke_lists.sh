#!/usr/bin/env bash
# 校验 KERNEL_ENTRY_CHECKLIST 中「命令名」表行所列举的 Tauri 命令，均出现在
# src-tauri/invoke_lists/*.txt（与 build.rs 生成的 invoke 表一致）。
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CHECKLIST="$ROOT/creator-docs/kernel/KERNEL_ENTRY_CHECKLIST.md"
LIST_DIR="$ROOT/src-tauri/invoke_lists"

if [[ ! -f "$CHECKLIST" ]]; then
  echo "missing $CHECKLIST" >&2
  exit 1
fi

tmp_invoke="$(mktemp)"
tmp_chk="$(mktemp)"
trap 'rm -f "$tmp_invoke" "$tmp_chk"' EXIT

for f in "$LIST_DIR"/*.txt; do
  [[ -f "$f" ]] || continue
  grep -v '^[[:space:]]*#' "$f" | grep -v '^[[:space:]]*$' | while read -r line; do
    line="${line//[[:space:]]/}"
    [[ -z "$line" ]] && continue
    cmd="${line##*::}"
    printf '%s\n' "$cmd"
  done
done | sort -u >"$tmp_invoke"

awk '/^## 事件\/Stream/{exit} {print}' "$CHECKLIST" \
  | sed -n 's/^|[[:space:]]*`\([a-z][a-z0-9_]*\)`.*/\1/p' \
  | sort -u >"$tmp_chk"

missing="$(comm -23 "$tmp_chk" "$tmp_invoke" || true)"
if [[ -n "${missing// }" ]]; then
  echo "CHECKLIST documents commands missing from invoke_lists:" >&2
  echo "$missing" >&2
  exit 1
fi

echo "check_kernel_entry_vs_invoke_lists: ok (checklist commands ⊆ invoke_lists, $(wc -l <"$tmp_chk" | tr -d ' ') checklist rows)"
