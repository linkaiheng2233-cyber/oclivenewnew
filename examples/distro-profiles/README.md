# Distro capability profile examples

Examples for [DISTRO_CAPABILITY_PROFILE.md](../../creator-docs/kernel/DISTRO_CAPABILITY_PROFILE.md) and [DISTRO_DEFAULT_PLUGINS.md](../../creator-docs/kernel/DISTRO_DEFAULT_PLUGINS.md).

| File | `distro_id` | Plugin strategy / use |
|------|-------------|------------------------|
| `desktop-chat.oclive.toml` | `desktop-chat` | **Lab** — no `[plugin_backends]`; official daily chat first impression |
| `desktop.oclive.toml` | `desktop` | Open ceiling; full prompt + mode switch (desktop spawn default) |
| `theater.oclive.toml` | `theater` | **Fixed light matrix** — memory/event `none`; AI Theater shell |
| `vscode.oclive.toml` | `vscode` | **Stable reference** — explicit all-`builtin` six slots |
| `vscode-penetration.oclive.toml` | `vscode-penetration` | Penetration defaults (orthogonal to six slots; 0.4+) |
| `vscode-agent.oclive.toml` | `vscode-agent` | VS Code + Agent/MCP (`skip_agent = false`) |

**Bundled install (K-PROFILE-04):** Release builds copy `desktop.oclive.toml` and `theater.oclive.toml` into Tauri `resources/distro-profiles/`. Desktop spawn resolves `{resource}/distro-profiles/desktop.oclive.toml` by default, or `theater.oclive.toml` when `OCLIVE_SHELL=theater`, unless `OCLIVE_DISTRO_PROFILE` is set.

**Spawn policy (product SSOT):** When nothing listens on `:8420`, hosts should spawn **this distro's bundled** `oclive-kernel-server` first, then **shared runtime** fallback with the same `OCLIVE_APP_DATA` / `OCLIVE_DISTRO_PROFILE` / `OCLIVE_ROLES_DIR`. Code gap: `discover_spawn_kernel_candidates` still sorts by discovery score (K-SCHED-05). See [KERNEL_SCHEDULER_RESCOPE.md](../../handoff/KERNEL_SCHEDULER_RESCOPE.md) and [DISTRO_KERNEL_LIFECYCLE.md](../../creator-docs/kernel/DISTRO_KERNEL_LIFECYCLE.md).

Runtime loading is implemented in P4 (`OCLIVE_DISTRO_ID` + `OCLIVE_DISTRO_PROFILE`).
