#!/usr/bin/env bash
# =============================================================================
# 一键验收：无头内核部署是否就绪（Linux）。
#
# 依赖：bash、curl；端口监听检查需要 ss 或 netstat；verbose JSON 检查需要 python3。
#
# 环境变量：
#   OCLIVE_HEALTH_URL   默认 http://127.0.0.1:${OOCP_API_PORT:-48888}
#   OOCP_API_PORT       与内核监听端口一致，用于 ss/netstat 检查（默认 48888）
#   OCLIVE_ROLES_DIR / OCLIVE_DB_PATH / OCLIVE_APP_DATA_DIR  与运行中的内核一致
#   OOCP_API_TOKEN      若内核启用了鉴权，请导出此变量（脚本会为 curl 附加 Bearer）
#
# 退出码：全部通过为 0，任一项失败为 1。
# =============================================================================

set -uo pipefail

PORT="${OOCP_API_PORT:-48888}"
BASE="${OCLIVE_HEALTH_URL:-http://127.0.0.1:${PORT}}"
BASE="${BASE%/}"

FAILED=0
pass() { echo "[PASS] $*"; }
fail() { echo "[FAIL] $*"; FAILED=1; }

CURL_AUTH=()
if [[ -n "${OOCP_API_TOKEN:-}" ]]; then
  CURL_AUTH=( -H "Authorization: Bearer ${OOCP_API_TOKEN}" )
fi

echo "== Oclive kernel deploy verification =="
echo "OCLIVE_HEALTH_URL=${BASE}  OOCP_API_PORT=${PORT}"
echo

# --- 环境变量 ---
if [[ -n "${OCLIVE_ROLES_DIR:-}" && -d "${OCLIVE_ROLES_DIR}" ]]; then
  pass "OCLIVE_ROLES_DIR is set and is a directory (${OCLIVE_ROLES_DIR})"
else
  fail "OCLIVE_ROLES_DIR missing or not a directory (current: ${OCLIVE_ROLES_DIR:-<unset>})"
fi

if [[ -n "${OCLIVE_DB_PATH:-}" ]]; then
  pass "OCLIVE_DB_PATH is set (${OCLIVE_DB_PATH})"
else
  fail "OCLIVE_DB_PATH is not set"
fi

if [[ -n "${OCLIVE_APP_DATA_DIR:-}" ]]; then
  pass "OCLIVE_APP_DATA_DIR is set (${OCLIVE_APP_DATA_DIR})"
else
  fail "OCLIVE_APP_DATA_DIR is not set"
fi

# --- 角色目录可读 ---
if [[ -n "${OCLIVE_ROLES_DIR:-}" && -r "${OCLIVE_ROLES_DIR}" ]]; then
  pass "roles directory is readable"
else
  fail "roles directory missing or not readable"
fi

# --- 数据库文件存在且可读写 ---
if [[ -n "${OCLIVE_DB_PATH:-}" && -f "${OCLIVE_DB_PATH}" ]]; then
  if [[ -r "${OCLIVE_DB_PATH}" && -w "${OCLIVE_DB_PATH}" ]]; then
    pass "database file exists and is readable/writable"
  else
    fail "database file exists but not readable/writable (${OCLIVE_DB_PATH})"
  fi
else
  fail "database file does not exist (${OCLIVE_DB_PATH:-<unset>})"
fi

# --- 端口监听 ---
if command -v ss >/dev/null 2>&1; then
  if ss -tln 2>/dev/null | grep -qE "[:.]${PORT}\\s"; then
    pass "TCP port ${PORT} is in LISTEN state (ss)"
  else
    fail "TCP port ${PORT} not found in LISTEN (ss -tln)"
  fi
elif command -v netstat >/dev/null 2>&1; then
  if netstat -tln 2>/dev/null | grep -qE "[:.]${PORT}\\s"; then
    pass "TCP port ${PORT} is listening (netstat)"
  else
    fail "TCP port ${PORT} not listening (netstat -tln)"
  fi
else
  fail "neither ss nor netstat in PATH; cannot verify port ${PORT}"
fi

# --- GET /health ---
if command -v curl >/dev/null 2>&1; then
  if body="$(curl -fsS --max-time 5 "${CURL_AUTH[@]}" "${BASE}/health" 2>/dev/null)" && [[ "${body}" == "ok" ]]; then
    pass "GET /health returns ok"
  else
    fail "GET /health did not return plain ok (url=${BASE}/health)"
  fi
else
  fail "curl not found"
fi

# --- GET /health?verbose=true ---
if command -v curl >/dev/null 2>&1 && command -v python3 >/dev/null 2>&1; then
  if vb="$(curl -fsS --max-time 10 "${CURL_AUTH[@]}" "${BASE}/health?verbose=true" 2>/dev/null)"; then
    if printf '%s' "${vb}" | python3 -c 'import json,sys
try:
    d=json.load(sys.stdin)
except Exception as e:
    raise SystemExit("invalid json: "+str(e))
st=d.get("status")
ch=d.get("checks") or {}
assert st=="ok", "status="+repr(st)
for k in ("db","roles","disk_space"):
    assert ch.get(k)=="ok", k+"="+repr(ch.get(k))
'; then
      pass "GET /health?verbose=true JSON status and checks are ok"
    else
      fail "GET /health?verbose=true JSON checks failed (body snippet: ${vb:0:200})"
    fi
  else
    fail "GET /health?verbose=true request failed"
  fi
else
  fail "curl or python3 missing; cannot verify verbose health"
fi

echo
if [[ "${FAILED}" -eq 0 ]]; then
  echo "All checks passed."
  exit 0
else
  echo "One or more checks failed."
  exit 1
fi
