# Distro capability profile examples

Examples for [DISTRO_CAPABILITY_PROFILE.md](../../creator-docs/kernel/DISTRO_CAPABILITY_PROFILE.md).

| File | `distro_id` | Use |
|------|-------------|-----|
| `desktop-chat.oclive.toml` | `desktop-chat` | **Official daily chat** — `pure_chat` default, concise prompt, light memory |
| `desktop.oclive.toml` | `desktop` | Full capability desktop reference; `pure_chat` default + mode switch |
| `theater.oclive.toml` | `theater` | AI Theater v0 — concise prompt, agent/complex_emotion off |
| `vscode.oclive.toml` | `vscode` | Concise prompt; no agent / complex_emotion; `pure_chat` default |
| `vscode-penetration.oclive.toml` | `vscode-penetration` | Penetration plugin defaults (0.4+; core profile has no `[penetration]`) |
| `vscode-agent.oclive.toml` | `vscode-agent` | VS Code + Agent/MCP profile |

**Bundled install (K-PROFILE-04):** Release builds copy `desktop.oclive.toml` and `theater.oclive.toml` into Tauri `resources/distro-profiles/`. Desktop spawn resolves `{resource}/distro-profiles/desktop.oclive.toml` by default, or `theater.oclive.toml` when `OCLIVE_SHELL=theater`, unless `OCLIVE_DISTRO_PROFILE` is set.

Runtime loading is implemented in P4 (`OCLIVE_DISTRO_ID` + `OCLIVE_DISTRO_PROFILE`). P1 is schema + samples only.
