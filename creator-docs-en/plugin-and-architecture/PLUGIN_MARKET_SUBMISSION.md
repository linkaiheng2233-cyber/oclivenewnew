# Plugin market submission and distribution (link-curation model)

[中文](../../creator-docs/plugin-and-architecture/PLUGIN_MARKET_SUBMISSION.md)

**Scope:** Directory plugins (`manifest.json` + `type: ocliveplugin`). **Excludes** role-pack market and Supabase community uploads.

## Design principles

| Role | Does | Does not |
|------|------|----------|
| **Plugin author** | Maintain source and docs in **your own Git repo**; submit **one link record** to the index (`git` + optional `gitSubdir`) | Upload zip to oclive official storage |
| **Index maintainer** (project) | Review PRs, maintain `plugins.json` (main-repo draft → [awesome-oclive-plugins](https://github.com/linkaiheng2233-cyber/awesome-oclive-plugins)) | Long-term hosting per plugin; curates metadata only |
| **End user** | Paste a trusted **share link** (`plugins.json` or single repo URL) in desktop **Plugin market**, then browse/install | Auto-fetch untrusted public catalogs by default |

This keeps **network traffic small** (index JSON is tiny; install only `git clone`s the author repo) and makes **spam plugins** unlikely in catalogs the user did not paste; official listing requires **GitHub PR + human review**.

## User side: share links

Desktop **Plugin & backend management → Plugin market**:

1. Paste the link the creator provides.
2. Click **Load**.
   - **`…/plugins.json`**: load catalog, browse multiple plugins.
   - **Git repo HTTPS / SSH**: treated as a single-plugin repo; install card shown (shallow clone to `{app_data}/distros/chat-pro/plugins/<id>/`).
3. After install, configure slots in **Plugin workbench**; high-risk permissions prompt per [DIRECTORY_PLUGINS.md](./DIRECTORY_PLUGINS.md).

Maintainers can share an audited catalog raw link, for example:

`https://raw.githubusercontent.com/linkaiheng2233-cyber/awesome-oclive-distros/chat-pro/plugins/main/plugins.json`

## Author side: submission flow (GitHub)

1. **Prepare the plugin repo** (standalone or monorepo subpath), meeting [PLUGIN_V1.md](./PLUGIN_V1.md) and [DIRECTORY_PLUGINS.md](./DIRECTORY_PLUGINS.md).
2. **Document inside the plugin directory** (checklist below; maintainers may reject incomplete entries).
3. Open a PR to oclivenewnew [`data/plugins.json`](../../data/plugins.json) adding one `plugins` element (fields in [GITHUB_PLUGIN_INDEX_LINE.md](../../handoff/GITHUB_PLUGIN_INDEX_LINE.md)).
4. After merge, maintainer syncs `plugins.json` to the awesome repo.
5. Put the **catalog raw link** or **your repo link** in README / release notes for users to paste into the market.

Local self-check:

```bash
node scripts/validate-plugins-index.mjs
```

## Required in-plugin documentation (author responsibility)

Index `description` is one or two lines; **real docs must live in the plugin repo** so users and reviewers can read after `git clone`.

### 1. `README.md` (required)

Suggested sections (Chinese or English, but complete):

| Section | Content |
|---------|---------|
| **Features** | Problem solved, `provides` / slots offered |
| **Requirements** | Node/Python versions, system deps, GPU/Ollama if needed |
| **Install** | Manual copy path under `distros/chat-pro/plugins/`; or market / `git` only |
| **Configuration** | `plugin_state`, env vars, mapping to `plugin_backends` |
| **Permissions** | Why each `manifest.json` `permissions` / `shell.bridge.invoke` entry is needed |
| **Compatibility** | Tested oclive / host versions (e.g. `0.2.x`) |
| **Support** | Issue link, email, or forum; **do not** write “contact author” with no URL |

### 2. `manifest.json` (required and honest)

- `id`: stable reverse-DNS, **exact match** with index entry `id`.
- `version`: semver, **match** index `version`.
- `permissions` / `process` / `shell.bridge`: **least privilege**; new permissions must be explained in README.
- Optional: `description` (one line), `author` (see below).

### 3. Optional but recommended

| File | Purpose |
|------|---------|
| `CHANGELOG.md` | Version history |
| `LICENSE` | License (main repo Apache-2.0; plugins may use MIT / Apache-2.0, etc.) |
| `author.json` | Author display name, recommended backends (shape in [AUTHOR_JSON.md](../role-pack/AUTHOR_JSON.md) if reused) |

### 4. Index entry (`plugins.json`) vs repo consistency

| Field | Requirement |
|------|-------------|
| `git` | URL that supports `git clone --depth 1` |
| `gitSubdir` | Monorepo path containing `manifest.json` |
| `description` | List summary; must not contradict README opening |
| `permissions` | Display list; should match manifest |
| `dependencies` | Other plugin ids with semver ranges if any |

## Maintainer review checklist (anti-spam)

Before merge:

- [ ] Repo accessible and contains **plugin source** (not empty shell or ad page).
- [ ] `manifest.json` matches index `id` / `version` / `git`(+`gitSubdir`).
- [ ] README includes required sections; permissions explained.
- [ ] No gratuitous `network:*`, `process:spawn`, etc.
- [ ] `validate-plugins-index.mjs` passes.
- [ ] No duplicate `id`; `description` / `tags` not misleading.

Reject examples: no README, over-broad permissions without explanation, index pointing at non-plugin repo, impersonating official `com.oclive.*` ids.

## Related docs

| Doc | Content |
|-----|---------|
| [PLUGIN_V1.md](./PLUGIN_V1.md) | Contract and RPC |
| [PLUGIN_AUTHOR_LEARNING_PATH.md](./PLUGIN_AUTHOR_LEARNING_PATH.md) | Author onboarding |
| [GITHUB_PLUGIN_INDEX_LINE.md](../../handoff/GITHUB_PLUGIN_INDEX_LINE.md) | Index fields, env vars, install semantics |
| [ERROR_CODES.md](../getting-started/ERROR_CODES.md) | Market offline/cache troubleshooting |
