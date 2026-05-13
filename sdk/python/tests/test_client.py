"""`OcliveClient` tests with mocked HTTP (no live kernel)."""

from __future__ import annotations

import http.client
import json
from io import BytesIO
from unittest.mock import MagicMock, patch

import pytest
import urllib.error

from oclive import OcliveClient, OcliveError


def _json_response(status: int, payload: object) -> MagicMock:
    body = json.dumps(payload, ensure_ascii=False).encode("utf-8")
    mock_resp = MagicMock()
    mock_resp.status = status
    mock_resp.read.return_value = body
    mock_resp.__enter__.return_value = mock_resp
    mock_resp.__exit__.return_value = None
    return mock_resp


def _text_response(status: int, text: str) -> MagicMock:
    mock_resp = MagicMock()
    mock_resp.status = status
    mock_resp.read.return_value = text.encode("utf-8")
    mock_resp.__enter__.return_value = mock_resp
    mock_resp.__exit__.return_value = None
    return mock_resp


@patch("oclive.client.urllib.request.urlopen")
def test_health_returns_stripped_ok(mock_urlopen: MagicMock) -> None:
    mock_urlopen.return_value = _text_response(200, "ok\n")
    client = OcliveClient(base_url="http://127.0.0.1:9", max_retries=0)
    assert client.health() == "ok"


@patch("oclive.client.urllib.request.urlopen")
def test_chat_parses_reply_field(mock_urlopen: MagicMock) -> None:
    mock_urlopen.return_value = _json_response(
        200, {"reply": "hello", "emotion": "Neutral"}
    )
    client = OcliveClient(base_url="http://127.0.0.1:9", max_retries=0)
    out = client.chat(role_path="/roles/x", message="hi")
    assert out["reply"] == "hello"


@patch("oclive.client.urllib.request.urlopen")
def test_http_401_raises_oclive_error(mock_urlopen: MagicMock) -> None:
    def _raise(_req, **_kwargs):
        raise urllib.error.HTTPError(
            "http://127.0.0.1:9/chat",
            401,
            "Unauthorized",
            http.client.HTTPMessage(),
            BytesIO(b'{"detail":"invalid token"}'),
        )

    mock_urlopen.side_effect = _raise
    client = OcliveClient(
        base_url="http://127.0.0.1:9",
        bearer_token="invalid",
        max_retries=0,
    )
    with pytest.raises(OcliveError) as exc_info:
        client.chat(role_path="/roles/x", message="x")
    assert "401" in str(exc_info.value)
