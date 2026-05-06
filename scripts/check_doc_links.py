#!/usr/bin/env python3
"""Scan creator-docs/**/*.md for relative markdown links; fail if target file missing.

Skips: http(s)://, mailto:, bare #anchors, <angle> autolinks.
HTTP URLs are listed as unverified (stdout only), never fail.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

LINK_RE = re.compile(r"\[[^\]]*\]\(([^)]+)\)")

ROOT = Path(__file__).resolve().parents[1]
DOCS = ROOT / "creator-docs"


def main() -> int:
    broken: list[tuple[str, int, str, str]] = []
    unverified_http: list[tuple[str, int, str]] = []

    for md in sorted(DOCS.rglob("*.md")):
        text = md.read_text(encoding="utf-8", errors="replace")
        rel_md = md.relative_to(ROOT).as_posix()
        for i, line in enumerate(text.splitlines(), start=1):
            for m in LINK_RE.finditer(line):
                raw = m.group(1).strip()
                if not raw or raw.startswith("#"):
                    continue
                if raw.startswith("http://") or raw.startswith("https://"):
                    unverified_http.append((rel_md, i, raw))
                    continue
                if raw.startswith("mailto:"):
                    continue
                # strip optional title fragment only for path check
                path_part = raw.split()[0] if raw.split() else raw
                if "#" in path_part:
                    path_part = path_part.split("#", 1)[0]
                if not path_part:
                    continue
                target = (md.parent / path_part).resolve()
                try:
                    target.relative_to(ROOT)
                except ValueError:
                    broken.append((rel_md, i, raw, f"escapes repo root: {target}"))
                    continue
                if not target.exists():
                    broken.append((rel_md, i, raw, f"missing: {target.relative_to(ROOT)}"))

    for rel_md, line_no, url in unverified_http[:50]:
        print(f"[http unverified] {rel_md}:{line_no} -> {url}")
    if len(unverified_http) > 50:
        print(f"... and {len(unverified_http) - 50} more http(s) links (not checked)")

    if broken:
        print("Broken relative links:", file=sys.stderr)
        for rel_md, line_no, raw, reason in broken:
            print(f"  {rel_md}:{line_no}: ({raw}) {reason}", file=sys.stderr)
        return 1
    print(f"OK: scanned {len(list(DOCS.rglob('*.md')))} markdown files under creator-docs/")
    return 0


if __name__ == "__main__":
    sys.exit(main())
