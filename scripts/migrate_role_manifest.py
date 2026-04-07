#!/usr/bin/env python3
"""
将旧版嵌套 personality 的 manifest.json 转为 DiskRoleManifest（扁平 default_personality + evolution 块）。

用法:
  python scripts/migrate_role_manifest.py roles/shimeng/manifest.json.bak -o roles/shimeng/manifest.json

不覆盖目标文件除非指定 -o；默认打印到 stdout。
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any


def migrate(data: dict[str, Any]) -> dict[str, Any]:
    if "default_personality" in data and isinstance(data["default_personality"], list):
        return data

    pid = data.get("id") or data.get("model") or ""
    pers = data.get("personality") or {}
    defaults = (pers.get("defaults") or {}) if isinstance(pers, dict) else {}
    order = [
        "stubbornness",
        "clinginess",
        "sensitivity",
        "assertiveness",
        "forgiveness",
        "talkativeness",
        "warmth",
    ]
    default_personality = [float(defaults.get(k, 0.5)) for k in order]

    evo_old = pers.get("evolution") if isinstance(pers, dict) else None
    evolution = {
        "event_impact_factor": 1.0,
        "ai_analysis_interval": 15,
        "max_change_per_event": 0.05,
        "max_total_change": 0.5,
    }
    if isinstance(evo_old, dict):
        if "speed" in evo_old:
            evolution["max_change_per_event"] = min(
                0.2, max(0.01, float(evo_old["speed"]) * 0.05)
            )

    scenes = data.get("scenes")
    if not isinstance(scenes, list):
        scenes = []

    ur_old = data.get("user_relations") or {}
    user_relations: dict[str, Any] = {}
    if isinstance(ur_old, dict):
        for key, val in ur_old.items():
            if not isinstance(val, dict):
                continue
            hint = val.get("prompt_hint") or ""
            user_relations[key] = {
                "prompt_hint": hint,
                "favor_multiplier": float(val.get("favor_multiplier", 1.0)),
            }

    out: dict[str, Any] = {
        "id": str(pid),
        "name": data.get("name", ""),
        "version": data.get("version", "1.0.0"),
        "author": data.get("author", ""),
        "description": data.get("description", ""),
        "default_personality": default_personality,
        "evolution": evolution,
        "scenes": scenes,
        "user_relations": user_relations,
        "default_relation": data.get("default_relation") or "friend",
        "memory_config": {
            "scene_weight_multiplier": 1.2,
            "topic_weights": {},
        },
    }
    return out


def main() -> None:
    p = argparse.ArgumentParser(description="Migrate legacy manifest.json to DiskRoleManifest")
    p.add_argument("input", type=Path, help="旧 manifest 路径")
    p.add_argument("-o", "--output", type=Path, default=None, help="写入路径（默认 stdout）")
    args = p.parse_args()
    raw = args.input.read_text(encoding="utf-8")
    data = json.loads(raw)
    new_data = migrate(data)
    text = json.dumps(new_data, ensure_ascii=False, indent=2) + "\n"
    if args.output:
        args.output.write_text(text, encoding="utf-8")
    else:
        sys.stdout.write(text)


if __name__ == "__main__":
    main()
