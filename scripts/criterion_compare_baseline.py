#!/usr/bin/env python3
"""
将 Criterion 终端输出与 kernel_perf_baseline_v0.json 对比；超阈值打印 GitHub Actions ::warning::。
不退出非零（默认），以免阻塞主 CI；可用 --fail-on-regression 使进程返回 1。
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path


def _parse_time_token(tok: str) -> float:
    """返回纳秒。"""
    t = tok.strip()
    m = re.match(r"^([\d.]+)\s*(ms|µs|us|ns)\s*$", t, re.I)
    if not m:
        raise ValueError(f"bad time token: {tok!r}")
    val = float(m.group(1))
    unit = m.group(2).lower()
    if unit == "ms":
        return val * 1_000_000.0
    if unit in ("µs", "us"):
        return val * 1_000.0
    if unit == "ns":
        return val
    raise ValueError(unit)


def extract_median_ns(log_text: str, bench_id: str) -> float | None:
    """从 Criterion 日志中提取某基准的中间估计值（95% CI 三项的中间项）。"""
    # 允许 ID 含 '/'；非贪婪匹配到 time 行
    pat = re.compile(
        re.escape(bench_id) + r"[\s\S]{0,400}?time:\s+\[([^\]]+)\]",
        re.MULTILINE,
    )
    m = pat.search(log_text)
    if not m:
        return None
    inner = m.group(1)
    parts = [p.strip() for p in inner.split() if p.strip()]
    # 形如 "12.5 ms" "13.0 ms" "14.0 ms" — 每两个 token 一组
    tokens: list[str] = []
    i = 0
    while i + 1 < len(parts):
        tokens.append(f"{parts[i]} {parts[i + 1]}")
        i += 2
    if len(tokens) < 2:
        return None
    mid = tokens[len(tokens) // 2]
    return _parse_time_token(mid)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--log",
        type=Path,
        required=True,
        help="Criterion 合并 stdout/stderr 的文本日志",
    )
    ap.add_argument(
        "--baseline",
        type=Path,
        default=Path("creator-docs/kernel/kernel_perf_baseline_v0.json"),
        help="JSON 基线",
    )
    ap.add_argument(
        "--fail-on-regression",
        action="store_true",
        help="任一超阈值则 exit 1",
    )
    args = ap.parse_args()

    raw = args.log.read_text(encoding="utf-8", errors="replace")
    baseline = json.loads(args.baseline.read_text(encoding="utf-8"))
    ratio = float(baseline.get("threshold_warn_ratio", 1.15))
    benches: dict[str, dict] = baseline["benchmarks"]

    any_warn = False
    missing = []
    for bid, meta in benches.items():
        base_ns = float(meta["median_ns"])
        cur = extract_median_ns(raw, bid)
        if cur is None:
            missing.append(bid)
            print(f"[criterion-compare] skip (not found in log): {bid}", file=sys.stderr)
            continue
        r = cur / base_ns if base_ns > 0 else float("inf")
        pct = (r - 1.0) * 100.0
        line = f"{bid}: baseline_median_ns={base_ns:.0f} current_median_ns={cur:.0f} ratio={r:.3f} ({pct:+.1f}%)"
        print(line)
        if r > ratio:
            any_warn = True
            msg = (
                f"Performance regression over baseline: {bid} ratio={r:.3f} "
                f"(threshold {ratio:.2f}). {line}"
            )
            print(f"::warning::{msg}")

    if missing:
        print(
            f"[criterion-compare] warning: {len(missing)} benchmark(s) missing from log "
            "(job may need longer run or bench rename).",
            file=sys.stderr,
        )

    if args.fail_on_regression and any_warn:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
