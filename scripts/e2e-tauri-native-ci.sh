#!/usr/bin/env bash
# A1.1c — minimal Tauri native WebDriver smoke (Ubuntu CI / local Linux).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
export ROOT="$ROOT"

export OCLIVE_ROLES_DIR="${OCLIVE_ROLES_DIR:-$ROOT/distros/chat-pro/roles}"
export OCLIVE_SKIP_STARTUP_HEALTH=1
export OCLIVE_SKIP_LLM_STARTUP_PROBE=1
export OCLIVE_TAURI_E2E=1
export ROOT="$ROOT"

DIAG_DIR="${E2E_TAURI_DIAG_DIR:-/tmp/e2e-tauri-diagnostics}"
mkdir -p "$DIAG_DIR"

# GTK/WebKit under xvfb often needs a session bus (artifact showed DBUS_SESSION_BUS_ADDRESS=).
if [[ -z "${DBUS_SESSION_BUS_ADDRESS:-}" ]] && command -v dbus-launch >/dev/null 2>&1; then
  eval "$(dbus-launch --sh-syntax)"
  echo "dbus-launch: DBUS_SESSION_BUS_ADDRESS=$DBUS_SESSION_BUS_ADDRESS"
fi

TARGET="$(python - <<'PY'
import json, os, subprocess
root = os.environ["ROOT"]
out = subprocess.check_output(
    ["cargo", "metadata", "--format-version=1", "--no-deps"],
    cwd=os.path.join(root, "distros", "desktop-tauri"),
    text=True,
)
print(json.loads(out)["target_directory"])
PY
)"
export TAURI_E2E_APP_PATH="${TAURI_E2E_APP_PATH:-$TARGET/debug/oclivenewnew-tauri}"
export TAURI_DRIVER_HOST="${TAURI_DRIVER_HOST:-127.0.0.1}"
export TAURI_DRIVER_PORT="${TAURI_DRIVER_PORT:-4444}"
export TAURI_NATIVE_DRIVER_PORT="${TAURI_NATIVE_DRIVER_PORT:-4445}"

if [[ ! -x "$TAURI_E2E_APP_PATH" ]]; then
  echo "Missing debug binary: $TAURI_E2E_APP_PATH" >&2
  exit 1
fi

if ! command -v tauri-driver >/dev/null 2>&1; then
  echo "tauri-driver not on PATH" >&2
  exit 1
fi

if ! command -v WebKitWebDriver >/dev/null 2>&1; then
  echo "WebKitWebDriver not on PATH (required for Linux tauri-driver sessions)" >&2
  exit 1
fi

dump_diagnostics() {
  echo "=== e2e-tauri diagnostics (session / driver) ===" >&2
  {
    echo "=== date ==="
    date -u || true
    echo "=== TAURI_E2E_APP_PATH ==="
    echo "$TAURI_E2E_APP_PATH"
    ls -la "$TAURI_E2E_APP_PATH" 2>&1 || true
    echo "=== DISPLAY / dbus ==="
    echo "DISPLAY=${DISPLAY:-}"
    echo "DBUS_SESSION_BUS_ADDRESS=${DBUS_SESSION_BUS_ADDRESS:-}"
    echo "=== ports ==="
    (ss -ltnp 2>/dev/null || netstat -ltnp 2>/dev/null || true) | grep -E '4444|4445' || true
    echo "=== related processes ==="
    ps auxww 2>/dev/null | grep -E '[t]auri-driver|[W]ebKitWebDriver|[o]clivenewnew-tauri' || true
    echo "=== tauri-driver.log (tail) ==="
    tail -n 200 /tmp/tauri-driver.log 2>&1 || echo "(missing /tmp/tauri-driver.log)"
    echo "=== webkit-webdriver.log (tail) ==="
    tail -n 200 /tmp/webkit-webdriver.log 2>&1 || echo "(missing /tmp/webkit-webdriver.log)"
  } | tee "$DIAG_DIR/diagnostics.txt" >&2

  cp -f /tmp/tauri-driver.log "$DIAG_DIR/tauri-driver.log" 2>/dev/null || true
  cp -f /tmp/webkit-webdriver.log "$DIAG_DIR/webkit-webdriver.log" 2>/dev/null || true
  cp -f "$DIAG_DIR/diagnostics.txt" /tmp/e2e-tauri-diagnostics.txt 2>/dev/null || true
}

cleanup() {
  if [[ -f /tmp/tauri-driver.pid ]]; then kill "$(cat /tmp/tauri-driver.pid)" 2>/dev/null || true; fi
  # Kill any WebKitWebDriver children tauri-driver may have spawned on the native port.
  pkill -f "WebKitWebDriver --port=${TAURI_NATIVE_DRIVER_PORT}" 2>/dev/null || true
}

on_exit() {
  local code=$?
  if [[ $code -ne 0 ]]; then
    dump_diagnostics || true
  fi
  cleanup || true
}
trap on_exit EXIT

# Do NOT pre-start WebKitWebDriver on TAURI_NATIVE_DRIVER_PORT: tauri-driver also
# spawns one there (artifact: FATAL Unable to listen on port 4445 + zombie WebKit).
: > /tmp/webkit-webdriver.log
tauri-driver --port "$TAURI_DRIVER_PORT" --native-port "$TAURI_NATIVE_DRIVER_PORT" >/tmp/tauri-driver.log 2>&1 &
echo $! >/tmp/tauri-driver.pid

for _ in $(seq 1 60); do
  if curl -sf "http://${TAURI_DRIVER_HOST}:${TAURI_DRIVER_PORT}/status" >/dev/null 2>&1; then
    break
  fi
  sleep 2
done
if ! curl -sf "http://${TAURI_DRIVER_HOST}:${TAURI_DRIVER_PORT}/status" >/dev/null 2>&1; then
  echo "tauri-driver did not become ready on :${TAURI_DRIVER_PORT}" >&2
  dump_diagnostics || true
  exit 1
fi

npm run test:e2e:tauri-native
