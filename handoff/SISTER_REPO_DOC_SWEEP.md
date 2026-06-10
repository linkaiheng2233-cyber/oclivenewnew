# Sister repository documentation sweep (read-only)

**Date:** 2026-06-08  
**Scope:** Opus 4.8 plan Phase 3 extension — verify narrative/version drift vs main repo `0.3.0`.

---

## oclive-vscode

| Item | Status | Notes |
|------|--------|-------|
| `ROADMAP.md` version header | **OK** | Lists **0.3.x** current; aligns with main host `0.3.0` |
| `VSCODE_DISTRIBUTION.md` link | **OK** | Points to main `creator-docs/role-pack/VSCODE_DISTRIBUTION.md` |
| Capability-first narrative | **OK** | Roadmap emphasizes attach/spawn, shared `OCLIVE_APP_DATA`, identity — matches main Phase 1 |
| `oclive.penetration.*` placeholders | **Deferred** | Documented as default-off; no cleanup without product traction (plan §Phase 5) |
| Open items | **Expected** | F5 acceptance, first `.vsix` release still unchecked — not doc drift |

**Action (2026-06-11):** Main repo contract docs synced — `VSCODE_DISTRIBUTION.md` / `CROSS_HOST_MEMORY.md` now **policy-first** (no standalone「attach 优先」); penetration documented as roadmap item without `oclive.penetration.*` keys yet.

---

## oclive-pack-editor

| Item | Status | Notes |
|------|--------|-------|
| README deprecated banner | **OK** | Points to **oclive-studio**; version **0.2.0** stated for archived editor |
| vs main `0.3.0` | **Expected** | Pack editor is archived at 0.2.0; main host 0.3.0 is correct split |
| Expert routing boundary | **OK** | README states expert_routing edited in main app, not editor |
| CI badge / CONTRIBUTING | **OK** | Self-contained; links to main `creator-docs` |

**Action:** None required; deprecated status matches reality.

---

## Cross-repo index

Main doc index §姊妹仓库：`creator-docs/getting-started/DOCUMENTATION_INDEX.md` — no broken relative paths found for vscode/pack-editor handoff links from main `AGENTS.md`.

---

## Related

- [RFC_PROFILE_AND_DOMAIN_REEXPORT.md](../creator-docs/rfc/RFC_PROFILE_AND_DOMAIN_REEXPORT.md)
- [TECHNICAL_DEBT_INVENTORY.md](./TECHNICAL_DEBT_INVENTORY.md) §Opus 4.8 follow-up
