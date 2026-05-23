# Someday · toolchain & CI (cost/benefit memo)

> **Nature**: pacing reminders—not urgent “tech debt.” Touch when capacity allows; delete rows you decide not to pursue.

[中文](../../creator-docs/roadmap/SOMEDAY_TOOLCHAIN_CI.md)

---

## Three one-line rules

1. **Contracts**: when **oclivenewnew** release changes pack contracts or validation crate, run **oclive-pack-editor** `npm run contract:json-keys` and align **`HOST_RUNTIME_VERSION`**.
2. **Automation**: add CI/E2E only when failures would surface **late** or blast radius is **large**; skip automating issues you catch quickly by hand.
3. **Matrix**: Windows + Linux CI covers most cases; add **macOS CI** when you ship Mac packages or Mac feedback grows.

---

## Optional later items

| Item | When it’s worth it |
|------|---------------------|
| Heavier E2E (multi-browser, real Tauri window) | Frequent UI/export churn or stronger quality promise |
| macOS-specific CI | Mac installer or Mac user base |
| Automated `HOST_RUNTIME_VERSION` vs host version check | Frequent releases or past alignment incidents |

**Related**: [BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md](BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md)
