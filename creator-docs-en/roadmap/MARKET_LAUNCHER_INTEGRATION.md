# Role pack / plugin market · launcher integration (release together)

**Goal experience**: when shipping desktop apps (runtime / launcher / editor), the **market site or index** is available; **oclive-launcher** exposes a clear entry (browser or embedded page).

[中文](../../creator-docs/roadmap/MARKET_LAUNCHER_INTEGRATION.md)

**Contracts**: [PACK_VERSIONING.md](../role-pack/PACK_VERSIONING.md); remote plugins: [REMOTE_PLUGIN_PROTOCOL.md](../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md). The market solves **discovery and distribution**, not the Remote protocol itself.

---

## Target UX (acceptance)

| Dimension | Notes |
|-----------|-------|
| **User** | Launcher → “Market / role packs” → list shows name, version, `min_runtime_version`, download, blurb. |
| **Release** | GitHub Release (or fixed channel) updates **index/static site**; launcher can override market root URL in settings. |
| **Security** | v1: HTTPS + official index; signing/third-party upload later. |

---

## Phased rollout (recommended)

1. **Entry + static index** — `catalog.json`, launcher deep link, host in-app market panel reading cache.
2. **Install loop** — git/zip install from index with dependency checks.
3. **UGC / Civitai-like** — accounts, uploads, CDN, moderation (heavy; plan separately).

See the Chinese doc for stage tables, launcher wiring, and **oclive-plugin-market** repo notes.

---

## Related

- Community site shape: [COMMUNITY_WEB_VISION.md](COMMUNITY_WEB_VISION.md)
- Plugin web section IA: [PLUGIN_WEB_SECTION.md](PLUGIN_WEB_SECTION.md)
