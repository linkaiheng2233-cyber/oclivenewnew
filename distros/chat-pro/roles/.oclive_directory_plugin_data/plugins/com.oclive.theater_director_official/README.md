# Official Theater Director Plugin

Directory plugin **`com.oclive.theater_director_official`** — implements **`theater.build_prompt`** for AI Theater (`generate_theater_scene` / `POST /theater/scene`).

- **Provides**: `theater_director`
- **RPC**: `theater.build_prompt` → `{ "prompt": "..." }`
- **Modes**: `ripple`, `patch`, `cast_adapt`, `cast_rewrite`, `cast_rewrite_minimal`

Bundled with Theater distro profile (`[theater].director_plugin`). Kernel falls back to builtin Rust templates when this plugin is missing or RPC fails.
