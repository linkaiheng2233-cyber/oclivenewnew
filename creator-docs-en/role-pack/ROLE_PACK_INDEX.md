# Community role pack index format (ROLE_PACK_INDEX)

This defines a **static JSON index** for market sites, launchers, or scripts to fetch a discoverable list of role packs. It is orthogonal to **single-pack on-disk layout** ([ROLE_PACK_SPEC.md](ROLE_PACK_SPEC.md)): the index does not replace in-pack `manifest.json`.

---

## File conventions

- **Media type**: `application/json; charset=utf-8`
- **Root type**: **array** of **objects** (“entries”)
- **Encoding**: UTF-8
- **Suggested extension**: `.json` (e.g. `catalog.json`, `role_pack_index.json`)

---

## Entry fields

| Field | Type | Required | Notes |
|-------|------|----------|--------|
| `id` | string | yes | Matches `manifest.id` in the pack |
| `name` | string | yes | Display name |
| `version` | string | yes | Align with `manifest.version` |
| `author` | string | no | Author |
| `description` | string | no | Summary |
| `tags` | string[] | no | Tags; clients may filter |
| `download_url` | string (uri) | yes | `.zip` / `.ocpak` / `.oclivepack`, etc. |
| `sha256` | string | no | Lowercase hex SHA-256 for integrity |
| `min_runtime_version` | string | no | Copy from manifest when identical |

**Programmatic filter example**: filter where `tags` contains `"sf"`; compare `version` with semver (parse yourself).

---

## Minimal example

```json
[
  {
    "id": "com.example.demo",
    "name": "Demo",
    "version": "0.1.0",
    "author": "Example",
    "description": "Minimal sample",
    "tags": ["sf", "builtin-only"],
    "download_url": "https://cdn.example.com/packs/com.example.demo-0.1.0.oclivepack",
    "sha256": "abcdef0123456789…"
  }
]
```

**JSON Schema**: `kernel/crates/oclive-cli/schemas/role_pack_index.schema.json`.

---

[中文](../../creator-docs/role-pack/ROLE_PACK_INDEX.md)
