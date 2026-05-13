#!/usr/bin/env bash
# 演示：将外挂「视觉摘要」前缀拼入 message 后调用 POST /chat。
# 用法：
#   export OCLIVE_KERNEL_URL=http://127.0.0.1:48888
#   export ROLE_PATH=/abs/path/to/roles/shimeng
#   export OOCP_API_TOKEN=...   # 若服务端启用
#   bash chat_with_context.sh
#
# 依赖：curl、python3（用于 JSON 转义）；若已安装 jq 可自行改写。

set -euo pipefail
BASE="${OCLIVE_KERNEL_URL:-http://127.0.0.1:48888}"
ROLE="${ROLE_PATH:?set ROLE_PATH to role directory containing manifest.json}"
VISION_SUMMARY="${VISION_SUMMARY:-用户看起来疲惫，室内光线偏暗。}"
USER_LINE="${USER_LINE:-我今天好累。}"

MSG="[视觉上下文] ${VISION_SUMMARY}"$'\n'"用户说：${USER_LINE}"
BODY=$(ROLE_PATH="$ROLE" MSG="$MSG" python3 -c 'import json,os; print(json.dumps({"role_path":os.environ["ROLE_PATH"],"message":os.environ["MSG"],"scene_id":"default"}))')

AUTH=()
if [[ -n "${OOCP_API_TOKEN:-}" ]]; then
  AUTH=(-H "Authorization: Bearer ${OOCP_API_TOKEN}")
fi

curl -sS "${AUTH[@]}" -H "Content-Type: application/json" -d "$BODY" "${BASE}/chat"
