"""HTTP 客户端：`GET /health`、`POST /chat`（与 `examples/kernel_remote_simple/client.py` 契约一致）。"""

from __future__ import annotations

import json
import time
import urllib.error
import urllib.request
from dataclasses import dataclass
from typing import Any, Mapping, Optional


class OcliveError(RuntimeError):
    """内核返回非 2xx、JSON 非法或契约字段缺失。"""


@dataclass
class OcliveClient:
    """与 `oclive_kernel_server` 或 runtime HTTP API 对话的最小客户端。"""

    base_url: str = "http://127.0.0.1:48888"
    bearer_token: Optional[str] = None
    timeout_s: float = 120.0
    max_retries: int = 2
    retry_backoff_s: float = 0.4

    def __post_init__(self) -> None:
        self.base_url = self.base_url.rstrip("/")

    def _headers(self, extra: Optional[Mapping[str, str]] = None) -> dict[str, str]:
        h: dict[str, str] = {}
        if extra:
            h.update(extra)
        if self.bearer_token:
            h["Authorization"] = f"Bearer {self.bearer_token}"
        return h

    def _request_json(
        self,
        method: str,
        path: str,
        *,
        body: Optional[bytes] = None,
        headers: Optional[dict[str, str]] = None,
        timeout: float,
    ) -> tuple[int, Any]:
        url = f"{self.base_url}{path}"
        hdrs = self._headers(headers)
        last_err: Optional[BaseException] = None
        for attempt in range(self.max_retries + 1):
            try:
                req = urllib.request.Request(url, data=body, method=method, headers=hdrs)
                with urllib.request.urlopen(req, timeout=timeout) as resp:
                    raw = resp.read().decode("utf-8", errors="replace")
                    return resp.status, json.loads(raw) if raw.strip().startswith("{") else raw
            except urllib.error.HTTPError as e:
                raise OcliveError(
                    f"HTTP {e.code} {path}: {e.read().decode('utf-8', errors='replace')}"
                ) from e
            except urllib.error.URLError as e:
                last_err = e
                if attempt >= self.max_retries:
                    break
                time.sleep(self.retry_backoff_s * (attempt + 1))
        assert last_err is not None
        raise OcliveError(f"请求失败 {path}: {last_err.reason!s}") from last_err

    def _request_text(
        self,
        method: str,
        path: str,
        *,
        body: Optional[bytes] = None,
        headers: Optional[dict[str, str]] = None,
        timeout: float,
    ) -> str:
        url = f"{self.base_url}{path}"
        hdrs = self._headers(headers)
        last_err: Optional[BaseException] = None
        for attempt in range(self.max_retries + 1):
            try:
                req = urllib.request.Request(url, data=body, method=method, headers=hdrs)
                with urllib.request.urlopen(req, timeout=timeout) as resp:
                    return resp.read().decode("utf-8", errors="replace")
            except urllib.error.HTTPError as e:
                raise OcliveError(
                    f"HTTP {e.code} {path}: {e.read().decode('utf-8', errors='replace')}"
                ) from e
            except urllib.error.URLError as e:
                last_err = e
                if attempt >= self.max_retries:
                    break
                time.sleep(self.retry_backoff_s * (attempt + 1))
        assert last_err is not None
        raise OcliveError(f"请求失败 {path}: {last_err.reason!s}") from last_err

    def health(self) -> str:
        """`GET /health` → 纯文本 `ok`。"""
        t = min(10.0, self.timeout_s)
        return self._request_text("GET", "/health", timeout=t).strip()

    def health_verbose(self) -> dict[str, Any]:
        """`GET /health?verbose=true` → JSON。"""
        t = min(30.0, self.timeout_s)
        _, data = self._request_json("GET", "/health?verbose=true", timeout=t)
        if not isinstance(data, dict):
            raise OcliveError("verbose health 期望 JSON 对象")
        return data

    def health_db(self) -> dict[str, Any]:
        """`GET /health/db` → JSON（监控用）。"""
        t = min(10.0, self.timeout_s)
        _, data = self._request_json("GET", "/health/db", timeout=t)
        if not isinstance(data, dict):
            raise OcliveError("health/db 期望 JSON 对象")
        return data

    def chat(
        self,
        *,
        role_path: str,
        message: str,
        session_id: Optional[str] = None,
        scene_id: Optional[str] = None,
    ) -> dict[str, Any]:
        """`POST /chat` → 解析后的 JSON 对象（含 `reply`）。"""
        payload = {
            "role_path": role_path,
            "message": message,
            "session_id": session_id,
            "scene_id": scene_id,
        }
        body = json.dumps(payload, ensure_ascii=False).encode("utf-8")
        _, data = self._request_json(
            "POST",
            "/chat",
            body=body,
            headers={"Content-Type": "application/json; charset=utf-8"},
            timeout=self.timeout_s,
        )
        if not isinstance(data, dict):
            raise OcliveError("/chat 响应应为 JSON 对象")
        if "reply" not in data:
            raise OcliveError("/chat 响应缺少 reply 字段")
        return data

    def close(self) -> None:
        """占位：无持久连接，与 `with` 对称。"""

    def __enter__(self) -> OcliveClient:
        return self

    def __exit__(self, *exc: object) -> None:
        self.close()
