# Experience differentiation backlog · vision cross-check

This page consolidates **product experience directions** and **vision items still in flight** for scheduling. It does **not** replace monthly milestones in [VISION_ROADMAP_MONTHLY.md](VISION_ROADMAP_MONTHLY.md).

Update the date note here on major direction changes; sync with `CHANGELOG.md` and contract docs.

[中文](../../creator-docs/roadmap/BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md)

---

## 1. Differentiation themes (runtime / editor / launcher)

### 1.1 Integrated create-and-test

| | |
|--|--|
| **Meaning** | Quick try-chat inside the editor after config changes—WYSIWYG for role authors. |
| **Repos** | Primarily **oclive-pack-editor**; align with **oclivenewnew** `load_role` and chat API contracts. |
| **Considerations** | Embedded light chat vs local runtime subprocess; **same validation as `load_role`**. |
| **Status** | **Pending product decision and scheduling**. |

### 1.2 Smarter dependency management (launcher)

| | |
|--|--|
| **Meaning** | Beyond env detection: one-click Ollama install/setup, recommended model pull—“download and chat”. |
| **Repos** | **oclive-launcher**; env boundaries per creator plugin architecture docs. |
| **Considerations** | Permissions, disk/network, model licenses, offline; no silent overwrite of user config. |
| **Done (baseline)** | Launcher env/diagnostics: Node/npm, Ollama CLI+API, editor/runtime paths; config reset with `.corrupt.bak`; open config dir. |
| **Status** | **Advanced** one-click Ollama/model/bundle flows **still scheduled**. |

### 1.3 Plugin / role marketplace and UGC

| | |
|--|--|
| **Meaning** | Browse, install, update official/community packs and plugins. |
| **Repos** | All three apps + index/server strategy; ties to `schema_version`, signing, trust. |
| **Considerations** | Relationship to disk import / `.ocpak`; security; remote protocol boundaries. |
| **Status** | **Pending** (usually after single-machine loop is stable). |
| **Launch notes** | [MARKET_LAUNCHER_INTEGRATION.md](MARKET_LAUNCHER_INTEGRATION.md) |

### 1.4 Open collaboration

| | |
|--|--|
| **Meaning** | Community plugins, packs, docs; templates lower onboarding. |
| **Baseline** | [CONTRIBUTING.md](../../CONTRIBUTING.md), [EXTENSION_POINTS.md](../plugin-and-architecture/EXTENSION_POINTS.md). |
| **Status** | **Ongoing**; complements marketplace but not identical. |

---

## 2. Vision items still in progress

See the Chinese source for the full table (Monolith mode, dual-core, HTTP `--api`, pack editor maturity, etc.). Treat **code + validation** as authority; this EN page is a scheduling mirror.

---

## Related docs

- Monthly plan: [VISION_ROADMAP_MONTHLY.md](VISION_ROADMAP_MONTHLY.md)
- Open lab summary: [VISION_OPEN_LAB.md](VISION_OPEN_LAB.md)
- Market + launcher: [MARKET_LAUNCHER_INTEGRATION.md](MARKET_LAUNCHER_INTEGRATION.md)
- Community web: [COMMUNITY_WEB_VISION.md](COMMUNITY_WEB_VISION.md)
