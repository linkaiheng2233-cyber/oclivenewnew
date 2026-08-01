# Windows setup appendix

| Component | Notes |
|-----------|--------|
| **Node.js ≥ 22** | Root `package.json` `engines`; optional `.nvmrc` |
| **Rust stable** | via `rustup` |
| **VS Build Tools** | “Desktop development with C++” (MSVC linker) |
| **WebView2** | Usually preinstalled on Win10/11 |

## External Cargo target-dir

[`.cargo/config.toml`](../.cargo/config.toml) → `../oclive-dev-artifacts/oclivenewnew-cargo-target/`

First full workspace build: **60–120 minutes** typical.

## LNK1104

- Kill stale `oclivenewnew-tauri.exe` / `tauri dev` processes
- Exclude `oclive-dev-artifacts/` from antivirus
- `cargo clean` on external target-dir if needed

## Playwright (CI skips on Windows frontend)

```powershell
npm run preview -- --host 127.0.0.1 --port 4180 --strictPort
$env:PW_TEST_USE_EXTERNAL='1'
npm run test:e2e:preview
```

Chinese: [human-docs/10_SETUP_WINDOWS.md](../human-docs/10_SETUP_WINDOWS.md)
