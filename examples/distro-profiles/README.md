# Distro capability profile examples

Examples for [DISTRO_CAPABILITY_PROFILE.md](../../creator-docs/kernel/DISTRO_CAPABILITY_PROFILE.md).

| File | `distro_id` | Use |
|------|-------------|-----|
| `desktop.oclive.toml` | `desktop` | Full capability; reference for desktop host |
| `vscode.oclive.toml` | `vscode` | Concise prompt; no agent / complex_emotion |

Runtime loading is implemented in P4 (`OCLIVE_DISTRO_ID` + `OCLIVE_DISTRO_PROFILE`). P1 is schema + samples only.
