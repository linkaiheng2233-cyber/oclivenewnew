# Theater Director Minimal (comedy fork example)

Self-contained directory plugin demonstrating how to fork the official theater director prompt pack.

| Item | Value |
|------|-------|
| **Provides** | `theater_director` |
| **RPC** | `theater.build_prompt` |
| **Prompt pack** | Local `prompts/` (copied from official; edit freely) |

## Why self-contained?

Copy this entire folder to `{app_data}/plugins/<your-id>/`. The bundled `prompts/` avoids broken imports to the monorepo `plugins/` tree after install.

Official reference: [`plugins/com.oclive.theater_director_official/README.md`](../../plugins/com.oclive.theater_director_official/README.md)

## Quick start

1. Copy folder → `{app_data}/plugins/com.example.theater_director_comedy/`
2. Edit `manifest.json` → new `id` (must not collide with official)
3. Tweak `prompts/drama_guardrails.mjs` or the comedy wrapper in `rpc_server.mjs`
4. Point Theater at your plugin:
   - Dev: `OCLIVE_THEATER_DIRECTOR_PLUGIN=com.example.theater_director_comedy`
   - Profile: `distro.oclive.toml` → `[theater].director_plugin = "<id>"`

## Local smoke

```powershell
cd examples/directory-plugin-theater-director-minimal
node -e "import { buildTheaterPrompt } from './prompts/index.mjs'; console.log(buildTheaterPrompt({ mode: 'patch', cast_a_name: 'A', cast_b_name: 'B', patch_tweak: { drama_seed: 'test' } }).slice(0, 200));"
node rpc_server.mjs
```

Drift guard (official plugin only): `node scripts/theater-prompt-drift.mjs` from repo root.
