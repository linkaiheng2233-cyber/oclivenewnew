# Lightweight profile and supply-chain baseline (LIGHTWEIGHT_PROFILE)

[中文](../../creator-docs/development/LIGHTWEIGHT_PROFILE.md)

This document records **Release settings, dependency slimming, audits, and binary size baselines**, aligned with root `Cargo.toml` / `distros/desktop-tauri/Cargo.lock`. Audience: maintainers and release owners.

**Related**: known vulns and upgrade path in **[security/KNOWN_VULNERABILITIES.md](../../creator-docs/security/KNOWN_VULNERABILITIES.md)**; audit scope boundaries in **[security/SECURITY_AUDIT_SCOPE.md](../../creator-docs/security/SECURITY_AUDIT_SCOPE.md)** (complements this doc §6.4).

---

## §1 Workspace Release profile (root `Cargo.toml`)

| Key | Current value | Notes |
|-----|----------------|-------|
| `profile.release.opt-level` | `3` | Runtime performance first (workspace default) |
| `profile.release.lto` | `"thin"` | Thin LTO; dependency crates use `codegen-units = 16` under `[profile.release.package."*"]` |
| `profile.release.codegen-units` | `1` | One CGU for workspace crates; slower compilation with more stable release performance |
| `profile.release.strip` | `"symbols"` | Strip symbols from release artifacts |
| `profile.release.panic` | `"abort"` | Abort the release process on panic |

**`target-dir`**: see repo root [`.cargo/config.toml`](../../.cargo/config.toml); build output can live outside the tree under `../oclive-dev-artifacts/oclivenewnew-cargo-target/`.

---

## §6 Supply chain and size

### §6.1 `cargo audit` toolchain

- **Pinned version**: **cargo-audit 0.22.1** (matches the audit step owned by CI `dimension5-acceptance`).
- **Local run**: `cargo audit` from repo root (workspace-root `Cargo.lock`)
  Offline: `cargo audit --no-fetch --stale` (requires a successful prior fetch of `advisory-db`).

### §6.4 Audit status (current)

**Current vulnerability-level count is 0** (last reviewed **2026-08-01**); warning-level findings remain tracked. Do not turn this measured result into an unconditional “zero vulnerabilities” claim. See **[KNOWN_VULNERABILITIES.md](../security/KNOWN_VULNERABILITIES.md)**.

Summary (**2026-08-01**, workspace-root `Cargo.lock`, `cargo audit`):

- **Vulnerability level (error)**: **0** (`sqlx-mysql` / `rsa` are absent; `event-listener` resolves to fixed 5.4.2).
- **Warning level (warning)**: **8** allowed/tracked findings, mainly the gtk/webkit Linux cluster, `glib`, `unic-*`, and yanked `spin`.

CI: **`dimension5-acceptance`** uniquely owns the main workflow's `cargo audit` plus `cargo deny licenses+bans`; `cargo-audit-lockfile.yml` covers lockfile/audit-policy PRs, and the duplicate standalone job is removed. `npm-audit` hard-gates both production dependencies and the full development graph. K-SUPPLY-12 passed npm audit and the Linux/Windows frontend gates in remote CI [`30714475985`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/30714475985) at frozen implementation `728219e7`.

### §6.5 Unused / optional dependencies (review conclusion)

| Item | Status |
|------|--------|
| **`reqwest` features (D-OPUS-01)** | Since 2026-06-08, workspace and desktop-host declarations use `default-features = false` with only **`json`** + **`rustls-tls`**; no `fs-*` or `blocking`. |
| **`sqlx` SQLite boundary** | `distros/desktop-tauri/Cargo.toml` consumes `oclive_sqlx` through the workspace alias. That thin facade depends directly on **`sqlx-core` 0.8.6** and **`sqlx-sqlite` 0.8.6**, not the umbrella `sqlx` crate. Dimension 5 rejects any `sqlx-mysql`, `sqlx-postgres`, or `rsa` lockfile regression. |
| **Dev-only / tooling deps** | Periodically check with `cargo machete` / `cargo udeps` (optional); never remove without full `cargo test` green. |

> Historical lists of removed deps are **not** kept here permanently; use `git log -p -- distros/desktop-tauri/Cargo.toml`.

### §6.6 Duplicate dependency review (`cargo tree -d`)

**Conclusion (summary)**: common **multiple versions** come from **Tauri / WebView / windows-\*** stacked with **sqlx / reqwest / toml**—acceptable technical debt; **prefer** upstream major alignment over hand-pinning single crates.

**Gate (K-SUPPLY-05 Minimal · 2026-07-15)**:

| Guard | Behavior |
|-------|----------|
| `deny.toml` `multiple-versions` | **`deny`** (new duplicates hard-fail) |
| `[bans.skip]` | Documented skips for **eco-unfixable** families (nonzero dup ≠ uncontrolled) |
| Ratchet | `handoff/LAYERING_BASELINE.json` → `cargo_duplicate_groups` (currently **80**) · `scripts/check-cargo-dedup-ratchet.mjs` |

**Remaining families (`cargo deny check bans`; default excludes pure-dev edges)**:

| Class | Examples | Disposition |
|-------|----------|-------------|
| **Eco-unfixable → skip** | `windows*` / multi-gen `windows-sys`, `toml`/`winnow`, `thiserror` 1\|2, `hashbrown`/`getrandom`/`bitflags` 1\|2, `base64`/`reqwest` | Per-crate `reason` in `deny.toml`; wait Tauri/sqlx/HTTP alignment |
| **Leaf-pinnable (not this wave)** | Occasional single crates via `[patch]` / bump | Full zero-skip is a separate campaign; **no** lock churn just to dodge the ratchet |

Examples: `bitflags` 1 vs 2 · `toml` 0.8 vs 0.9/1.x · `windows-sys` 0.48–0.61. Spot-check before release.

### §6.7 `cargo-bloat` baseline (Windows x86_64, Release)

**Sample command** (adjust path when `target-dir` is external):

```bash
cd distros/desktop-tauri
cargo bloat --release -n 8
```

**Last sample**: **2026-05-20**, `oclivenewnew-tauri.exe` (`--release`, profile §1; `cargo bloat --release -n 8`, path under external `target-dir` as on your machine). The values remained in the same range as the 2026-05-12 sample.

| Metric | Value |
|--------|-------|
| **`.text` section (bloat report)** | **7.6 MiB** (the “63.1% 100.0%” line in the report) |
| **PE file size** | **12.0 MiB** (bloat report last line “the file size is …”) |

**Top symbols (by `.text` contribution, excerpt)**:

| Share (of file) | Size | Notes |
|-------------------|------|-------|
| 1.4% | 170.4 KiB | `oclivenewnew_tauri::run::closure$3` |
| 0.9% | 113.1 KiB | `RoleStorage::load_role_from_dir` |
| 0.7% | 88.8 KiB | `tauri::app::Builder::build` |
| 0.7% | 87.6 KiB | `tauri_runtime_wry::handle_user_message` |
| 0.5% | 60.5 KiB | `plugin_bridge::dispatch_bridge_command::async_fn$0` |
| 0.4% ×2 | 52.3 KiB | `chat_engine::co_present::process_co_present::async_fn$0` |
| 0.3% | 41.6 KiB | `tauri::asset_protocol::asset_protocol_handler` |

> Numbers move with **Rust version, dependency upgrades, LTO/strip**; refresh this table’s date and one-line command output before release.

---

## Revision history

| Date | Notes |
|------|--------|
| 2026-08-02 | Aligned the Release profile with root `Cargo.toml`, recorded K-SUPPLY-12 remote closure, restored the reqwest feature row, and aligned the latest bloat sample date with the Chinese canonical document. |
| 2026-05-12 | §6.4 / §6.7: re-ran `cargo audit` and `cargo bloat --release -n 8`, refreshed summary date and bloat numbers (`.text` 7.6 MiB, PE 12.0 MiB). |
| 2026-05-13 | First version: aligned with `main` lockfile, `cargo audit` / `cargo bloat` sampling; linked KNOWN_VULNERABILITIES. |
