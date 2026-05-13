#!/usr/bin/env python3
"""
通过 HTTP 调用 oclive_kernel_server 的 /health 与 /chat（标准库，无 pip 依赖）。
"""
from __future__ import annotations

import argparse
import json
import sys
import urllib.error
import urllib.request


def main() -> int:
    p = argparse.ArgumentParser(description="Oclive kernel HTTP /health + /chat demo")
    p.add_argument(
        "--base-url",
        default="http://127.0.0.1:48888",
        help="内核 HTTP 根地址（默认与 OOCP_API_PORT=48888 一致）",
    )
    p.add_argument(
        "--role-path",
        required=True,
        help="含 manifest.json 的角色目录绝对路径（与桌面 README POST /chat 一致）",
    )
    p.add_argument("--message", default="你好，请用一句话自我介绍。", help="用户消息")
    p.add_argument("--session-id", default=None, help="可选会话 id")
    p.add_argument("--scene-id", default=None, help="可选场景 id")
    p.add_argument("--timeout", type=float, default=120.0, help="单次 HTTP 超时秒数")
    args = p.parse_args()

    base = args.base_url.rstrip("/")
    timeout = args.timeout

    # --- GET /health ---
    health_url = f"{base}/health"
    try:
        req = urllib.request.Request(health_url, method="GET")
        with urllib.request.urlopen(req, timeout=min(10.0, timeout)) as resp:
            body = resp.read().decode("utf-8", errors="replace")
    except urllib.error.HTTPError as e:
        print(f"[health] HTTP {e.code}: {e.read().decode('utf-8', errors='replace')}", file=sys.stderr)
        return 1
    except urllib.error.URLError as e:
        print(f"[health] 连接失败或超时（内核是否在跑？）: {e.reason}", file=sys.stderr)
        return 1

    print(f"[health] {health_url} -> {body!r}")
    if body.strip() != "ok":
        print("[health] 预期响应纯文本 ok", file=sys.stderr)
        return 1

    # --- POST /chat ---
    chat_url = f"{base}/chat"
    payload = {
        "role_path": args.role_path,
        "message": args.message,
        "session_id": args.session_id,
        "scene_id": args.scene_id,
    }
    data = json.dumps(payload, ensure_ascii=False).encode("utf-8")
    req = urllib.request.Request(
        chat_url,
        data=data,
        method="POST",
        headers={"Content-Type": "application/json; charset=utf-8"},
    )
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            raw = resp.read().decode("utf-8", errors="replace")
    except urllib.error.HTTPError as e:
        err_body = e.read().decode("utf-8", errors="replace")
        print(f"[chat] HTTP {e.code}", file=sys.stderr)
        try:
            err_json = json.loads(err_body)
            print(json.dumps(err_json, ensure_ascii=False, indent=2), file=sys.stderr)
        except json.JSONDecodeError:
            print(err_body, file=sys.stderr)
        return 1
    except urllib.error.URLError as e:
        print(f"[chat] 连接失败或超时: {e.reason}（可增大 --timeout 或检查 Ollama）", file=sys.stderr)
        return 1

    try:
        obj = json.loads(raw)
    except json.JSONDecodeError:
        print(raw)
        return 1

    reply = obj.get("reply")
    if reply is None:
        print(json.dumps(obj, ensure_ascii=False, indent=2))
        print("[chat] 响应中无 reply 字段", file=sys.stderr)
        return 1

    print("[chat] reply:")
    print(reply)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
