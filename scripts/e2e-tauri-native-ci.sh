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

if command -v WebKitWebDriver >/dev/null 2>&1; then
  WebKitWebDriver --port="$TAURI_NATIVE_DRIVER_PORT" >/tmp/webkit-webdriver.log 2>&1 &
  echo $! >/tmp/webkit-webdriver.pid
  sleep 1
fi

tauri-driver --port "$TAURI_DRIVER_PORT" --native-driver-port "$TAURI_NATIVE_DRIVER_PORT" >/tmp/tauri-driver.log 2>&1 &
echo $! >/tmp/tauri-driver.pid

for _ in $(seq 1 30); do
  if curl -sf "http://${TAURI_DRIVER_HOST}:${TAURI_DRIVER_PORT}/status" >/dev/null 2>&1; then
    break
  fi
  sleep 1
done
if ! curl -sf "http://${TAURI_DRIVER_HOST}:${TAURI_DRIVER_PORT}/status" >/dev/null 2>&1; then
  echo "tauri-driver did not become ready on :${TAURI_DRIVER_PORT}" >&2
  tail -n 40 /tmp/tauri-driver.log >&2 || true
  exit 1
fi

cleanup() {
  if [[ -f /tmp/tauri-driver.pid ]]; then kill "$(cat /tmp/tauri-driver.pid)" 2>/dev/null || true; fi
  if [[ -f /tmp/webkit-webdriver.pid ]]; then kill "$(cat /tmp/webkit-webdriver.pid)" 2>/dev/null || true; fi
}
trap cleanup EXIT

npm run test:e2e:tauri-native
