# Distro capability profile examples

Examples for [DISTRO_CAPABILITY_PROFILE.md](../../creator-docs/kernel/DISTRO_CAPABILITY_PROFILE.md) and [DISTRO_DEFAULT_PLUGINS.md](../../creator-docs/kernel/DISTRO_DEFAULT_PLUGINS.md).

| File | `distro_id` | Product / use |
|------|-------------|----------------|
| `desktop.oclive.toml` | `desktop` | **Chat Pro** — open ceiling; full prompt + mode switch (Tauri default spawn) |
| `vscode.oclive.toml` | `vscode` | **VS Code Flash** — explicit all-`builtin` six slots |
| `desktop-chat.oclive.toml` | `desktop-chat` | **dev lab only** — concise/light + open ceiling; **not** Release hero |
| `vscode-penetration.oclive.toml` | `vscode-penetration` | Penetration defaults (orthogonal to six slots; 0.4+) |
| `vscode-agent.oclive.toml` | `vscode-agent` | VS Code + Agent/MCP (`skip_agent = false`) |

**Bundled install (K-PROFILE-04):** Release builds copy `desktop.oclive.toml` into Tauri `resources/distro-profiles/`. Desktop spawn resolves `{resource}/distro-profiles/desktop.oclive.toml` by default, unless `OCLIVE_DISTRO_PROFILE` is set.

**Spawn policy (K-SCHED-05):** When nothing listens on `:8420`, hosts spawn **this distro's bundled** `oclive-kernel-server` first, then **shared runtime** fallback with the same `OCLIVE_APP_DATA` / `OCLIVE_DISTRO_PROFILE` / `OCLIVE_ROLES_DIR`. Dev builds participate only when `OCLIVE_DEVELOPER=1`. See [KERNEL_SCHEDULER_RESCOPE.md](../../handoff/KERNEL_SCHEDULER_RESCOPE.md) and [DISTRO_KERNEL_LIFECYCLE.md](../../creator-docs/kernel/DISTRO_KERNEL_LIFECYCLE.md).

Runtime loading is implemented in P4 (`OCLIVE_DISTRO_ID` + `OCLIVE_DISTRO_PROFILE`).
