# Monolith weld benchmark report (template)

Use this worksheet to compare **standard** vs **Monolith welded** subprocess latency and decide how to tune `monolith.toml` → `weld_modules`.

> **Desktop host** uses a fixed `process_message` path; this report is for **headless / scaffold** kernels only.

## 1. Environment

| Field | Your notes |
|-------|------------|
| Project | |
| Date | |
| `weld_modules` | |
| Command | `cargo run -p oclive-cli -- bench --release --runs 20 -o <project root>` |
| Auto after init | `oclive init --monolith --monolith-bench-preset <latency\|memory\|embedded>` → `bench_results/report.json` |

## 2. Latency table (ms)

| Slot / metric | Standard p50 | Monolith p50 | Standard P95 | Monolith P95 | Gain % (p50) |
|---------------|-------------|-------------|-------------|-------------|--------------|
| **Whole binary** | | | | | |
| memory | — | — | — | — | — |
| emotion | — | — | — | — | — |
| event | — | — | — | — | — |
| prompt | — | — | — | — | — |
| llm | — | — | — | — | — |
| agent | — | — | — | — | — |
| complex_emotion | — | — | — | — | — |

`oclive bench` today measures **one hot path per binary**, not per-slot. Fill the **Whole binary** row from `bench_results/report.json` (`standard_ms` / `monolith_ms`).

**Gain % (p50)** ≈ `(standard_p50 - monolith_p50) / standard_p50 × 100`.

## 3. Multi-dimensional metrics (report schema v2)

| Dimension | JSON field | Unit | Notes |
|-----------|------------|------|--------|
| Binary size | `binary_size.*` | bytes | release binaries on disk |
| Peak memory | `peak_memory.*` | MiB | max RSS across bench runs per variant |
| Build time | `build_time.*` | seconds | separate timed `cargo build --release` passes |

## 4. Next steps

- [ ] Monolith faster → keep or expand `weld_modules`
- [ ] Similar → shrink weld set (e.g. `embedded` preset)
- [ ] Monolith slower → review weld plan / binary size

After editing `monolith.toml`:

```bash
cargo run -p oclive-cli -- build -o .
cargo run -p oclive-cli -- bench --release -o . --runs 20 --save
```

See **`BLUEPRINT_V2_POINTER.md`** and **`RFC_OCLIVE_MONOLITH_MODE.md`** in the oclivenewnew repo.
