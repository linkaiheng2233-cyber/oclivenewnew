# Distro capability profile examples

Examples for [DISTRO_CAPABILITY_PROFILE.md](../../creator-docs/kernel/DISTRO_CAPABILITY_PROFILE.md).

| File | `distro_id` | Use |
|------|-------------|-----|
| `desktop-chat.oclive.toml` | `desktop-chat` | **Official daily chat** — `pure_chat` default, concise prompt, light memory |
| `desktop.oclive.toml` | `desktop` | Full capability desktop reference; `pure_chat` default + mode switch |
| `vscode.oclive.toml` | `vscode` | Concise prompt; no agent / complex_emotion; `pure_chat` default |

Runtime loading is implemented in P4 (`OCLIVE_DISTRO_ID` + `OCLIVE_DISTRO_PROFILE`). P1 is schema + samples only.
