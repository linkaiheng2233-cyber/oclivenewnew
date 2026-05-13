#!/usr/bin/env bash
# 探活 oclive_kernel_server — 供 cron、Uptime Kuma、Prometheus blackbox 等调用。
# 依赖：curl
#
# 环境变量：
#   OCLIVE_HEALTH_URL  默认 http://127.0.0.1:48888
#   CURL_OPTS            附加 curl 参数（如 -H "Authorization: Bearer ..." 若探活需过网关）
#
# 成功：退出码 0；失败：非 0。

set -euo pipefail

BASE_URL="${OCLIVE_HEALTH_URL:-http://127.0.0.1:48888}"
BASE_URL="${BASE_URL%/}"

# shellcheck disable=SC2086
body="$(curl -fsS ${CURL_OPTS:-} "${BASE_URL}/health" || true)"
if [[ "${body}" == "ok" ]]; then
  exit 0
fi

echo "health check failed: expected body 'ok', got '${body}' (url=${BASE_URL}/health)" >&2
exit 1
