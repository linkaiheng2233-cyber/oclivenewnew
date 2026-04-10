"""
Shared non-LLM JSON-RPC stubs for sidecar examples (memory / emotion / event / prompt).

LLM methods are *not* implemented here — each example supplies its own llm.generate / llm.generate_tag.

Note: the host may send extra keys in params (e.g. event.estimate / prompt.build_prompt include
personality_source: \"vector\" | \"profile\"). These stubs ignore unknown fields.
"""
from typing import Any, Dict, Optional

from .jsonrpc_http import rpc_error


def handle_stub_plugin_method(method: str, params: dict) -> Optional[Dict[str, Any]]:
    """
    Returns:
      - inner `result` dict for success;
      - or full JSON-RPC error object from `rpc_error(...)` (merged with request id by dispatcher);
      - or None if the method is llm.* (handled by the example) or unknown.
    """
    if not isinstance(params, dict):
        params = {}

    if method == "memory.rank":
        memories = params.get("memories") or []
        limit = int(params.get("limit") or 8)
        if "__RATE_LIMIT__" in str(params.get("user_query") or ""):
            return rpc_error(None, -32012, "rate limited by demo query token")
        ids = [m.get("id") for m in memories if isinstance(m, dict) and m.get("id")]
        return {"ordered_ids": ids[:limit]}

    if method == "emotion.analyze":
        return {
            "joy": 0.0,
            "sadness": 0.0,
            "anger": 0.0,
            "fear": 0.0,
            "surprise": 0.0,
            "disgust": 0.0,
            "neutral": 1.0,
        }

    if method == "event.estimate":
        return {"event_type": {"Ignore": None}, "impact_factor": 0.0, "confidence": 0.5}

    if method == "prompt.build_prompt":
        return {"prompt": "[remote stub] 请在侧车实现真实 prompt 组装。\n"}

    if method == "prompt.top_topic_hint":
        return {"hint": None}

    if method in ("llm.generate", "llm.generate_tag"):
        return None

    return None
