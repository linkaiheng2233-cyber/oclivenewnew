# Community web vision · three sections (forum / role packs / plugins)

Product-level vision for a **web-first community**; pairs with [MARKET_LAUNCHER_INTEGRATION.md](MARKET_LAUNCHER_INTEGRATION.md) (distribution + launcher entry).

[中文](../../creator-docs/roadmap/COMMUNITY_WEB_VISION.md)

---

## Overall shape

- **Primary surface**: website (launcher opens browser or future WebView).
- **Three top-level sections**:

| Section | Content | Technical tie-in |
|---------|---------|------------------|
| **Forum** | Boards/threads; optional shared chat logs (with consent) | Does not replace runtime |
| **Role packs** | Civitai-like discovery: art, config tips, story triggers, downloads | `.zip` / `.ocpak` per [PACK_VERSIONING.md](../role-pack/PACK_VERSIONING.md) |
| **Plugins** | Remote sidecar docs, examples, discussion—not host `.dll` installs | [REMOTE_PLUGIN_PROTOCOL.md](../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md) |

---

## Cost-aware strategy

- **Phase A**: static catalog + docs; manual curation via `catalog.json` / PR.
- **Phase B**: accounts, uploads, moderation—only when UGC volume justifies ops cost.
- **Privacy**: chat-log sharing requires explicit consent and redaction guidance (see Chinese doc §4).

Full user stories and Discord vs web tradeoffs: Chinese source doc.
