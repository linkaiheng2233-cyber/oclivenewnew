# Windows setup appendix

| Component | Notes |
|-----------|--------|
| **Node.js ≥ 22** | Root `package.json` `engines`; optional `.nvmrc` |
| **Rust stable** | via `rustup` |
| **VS Build Tools** | “Desktop development with C++” (MSVC linker) |
| **WebView2** | Usually preinstalled on Win10/11 |

## External Cargo target-dir

[`.cargo/config.toml`](../.cargo/config.toml) → `../oclive-dev-artifacts/oclivenewnew-cargo-target/`

Before maintenance, check the effective target directory, active build/debug consumers and frozen evidence references. A directory name or Git ignore rule does not establish deletion permission, including an old in-repository `target/`. See the [build-debt retention rules](../handoff/debt-marathon/waves/WAVE-20260928-BUILD-OBSERVATION.md#缓存用途与保留准则2026-09-28) for file identity, hard links and maintenance prerequisites.

First full workspace build: **60–120 minutes** typical.

## LNK1104

1. Identify the exact file in the error. Check its path, existence, permissions and available disk space; distinguish a missing input library from a locked output exe/PDB.
2. For a locked output, verify that the owner belongs to your build/debug session and exit that session normally. Do not terminate every process with the same name.
3. If security software blocked the file, inspect its event record and follow the local policy for that event. Excluding the entire artifact directory is not a default remedy.
4. Do not start with a full `cargo clean`. Bounded cache maintenance requires a confirmed cache problem, checked frozen references, preservation/rollback, a rebuild budget and verified absolute paths. Preserve the original error and the rebuild result.

## Playwright (CI skips on Windows frontend)

```powershell
npm run preview -- --host 127.0.0.1 --port 4180 --strictPort
$env:PW_TEST_USE_EXTERNAL='1'
npm run test:e2e:preview
```

Chinese: [human-docs/10_SETUP_WINDOWS.md](../human-docs/10_SETUP_WINDOWS.md)
