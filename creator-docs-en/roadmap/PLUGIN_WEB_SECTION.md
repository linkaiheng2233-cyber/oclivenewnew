# Plugin section (website) · information architecture

For the community site **Plugins** area: discovery and docs for **Remote HTTP sidecars**, not installing host binaries from the web.

[中文](../../creator-docs/roadmap/PLUGIN_WEB_SECTION.md)

**Authority**: [REMOTE_PLUGIN_PROTOCOL.md](../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md) · [CREATOR_PLUGIN_ARCHITECTURE.md](../plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md) · [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md)

---

## One-line pitch

**Find what a sidecar is, which env vars to set, and how `plugin_backends` maps; download/source on the author’s GitHub/Release.**

---

## Suggested routes

| Path | Purpose |
|------|---------|
| `/plugins` | Card list: name, blurb, compatible oclive version, author, GitHub link, tags |
| `/plugins/how-it-works` | Remote vs built-in; env var table linking to architecture docs |
| `/plugins/examples` | Summary + link to [remote_plugin_minimal](../../examples/remote_plugin_minimal/README.md) |
| `/plugins/submit` | How to get listed (public repo, protocol compatibility, tested version) |

---

## Data source (phase A)

Maintain **`data/plugins.json`** in the market repo; static site build reads it at CI time. Field schema and i18n notes: Chinese source doc §3.
