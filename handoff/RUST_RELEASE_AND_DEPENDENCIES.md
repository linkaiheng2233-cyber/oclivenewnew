# Rust Release 与 workspace 依赖

根 [`Cargo.toml`](../Cargo.toml)：`[profile.release]`（`opt-level=z`, `lto=thin`, `codegen-units=1`）与 `[workspace.dependencies]` 的 `tokio`、`reqwest`。

- **Tokio**：`macros`, `net`, `rt-multi-thread`, `time`（替代 `full`）。若使用 `tokio::fs` 等需在 workspace 补 feature。
- **Reqwest**：`default-features = false`，`json`, `blocking`, `gzip`, `default-tls`。
- **侧车**：[`src-tauri/sidecars/oclive-llama-sidecar/Cargo.toml`](../src-tauri/sidecars/oclive-llama-sidecar/Cargo.toml) 独立配置。

门禁与 `package.json` 中 `check:rust:*` 一致：`--manifest-path src-tauri/Cargo.toml` + `--workspace`。

详见 [`BUNDLE_RESOURCES_SIZING.md`](./BUNDLE_RESOURCES_SIZING.md)。
