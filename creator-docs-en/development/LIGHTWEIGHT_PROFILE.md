# Lightweight profile and supply-chain baseline (LIGHTWEIGHT_PROFILE)

[中文](../../creator-docs/development/LIGHTWEIGHT_PROFILE.md)

This document records **Release settings, dependency slimming, audits, and binary size baselines**, aligned with root `Cargo.toml` / `distros/desktop-tauri/Cargo.lock`. Audience: maintainers and release owners.

**Related**: known vulns and upgrade path in **[security/KNOWN_VULNERABILITIES.md](../creator-docs/security/KNOWN_VULNERABILITIES.md)**; audit scope boundaries in **[security/SECURITY_AUDIT_SCOPE.md](../creator-docs/security/SECURITY_AUDIT_SCOPE.md)** (complements this doc §6.4).

---

## §1 Workspace Release profile (root `Cargo.toml`)

| Key | Current value | Notes |
|-----|----------------|-------|
| `profile.release.opt-level` | `"z"` | Size-first |
| `profile.release.lto` | `true` | Full-crate LTO at link time; equivalent to **fat LTO** (`true` semantics since Rust 1.46+) |
| `profile.release.strip` | *(unset)* | Optionally add `strip = "debuginfo"` or `"symbols"` to shrink release artifacts further (validate crash symbol needs per release) |
| `profile.release.codegen-units` | *(unset)* | Optionally `codegen-units = 1` for smaller binary and more reproducible perf (slower compile) |

**`target-dir`**: see repo root [`.cargo/config.toml`](../.cargo/config.toml); build output can live outside the tree under `../oclive-dev-artifacts/oclivenewnew-cargo-target/`.

---

## §6 Supply chain and size

### §6.1 `cargo audit` toolchain

- **Pinned version**: **cargo-audit 0.22.1** (matches CI `cargo-audit` job for comparable reports).
- **Local run**: `cargo audit` from repo root (lockfile: `distros/desktop-tauri/Cargo.lock`)  
  Offline: `cargo audit --no-fetch --stale` (requires a successful prior fetch of `advisory-db`).

### §6.4 Audit status (current)

**Known vulnerabilities under tracking**; **do not claim zero vulns**. Vulnerability-level hits and roadmap: **[KNOWN_VULNERABILITIES.md](../creator-docs/security/KNOWN_VULNERABILITIES.md)** (see that file for last update date).

Summary (**2026-05-12**, `cargo audit --no-fetch --stale`, `distros/desktop-tauri/Cargo.lock`; matches that CLI run):

- **Vulnerability level (error)**: **5** (`rsa`, `rustls-webpki` ×3 advisories, `sqlx`).
- **Warning level (warning)**: **17** (includes gtk-rs *unmaintained*, `rustls-pemfile` *unmaintained*, `glib` *unsound*, etc.); **not** listed in the KNOWN table, but release review should read full `cargo audit` output.

CI: `.github/workflows/ci.yml` **`cargo-audit`** job uses **`continue-on-error: true`** for visibility without blocking merges; tighten to fail-on-red after dependency upgrades.

### §6.5 Unused / optional dependencies (review conclusion)

| Item | Status |
|------|--------|
| **`sqlx` default features** | Current `distros/desktop-tauri/Cargo.toml` uses **`sqlx = { version = "0.7", features = [...] }`** explicit list; if the lockfile still contains **`sqlx-mysql` / `sqlx-postgres`**, it is often from **macros / compile-time** or historical resolution—**mid-term** should combine **sqlx 0.8+** and **sqlite-only** features for another trim pass. |
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

**Last sample**: **2026-05-12**, `oclivenewnew-tauri.exe` (`--release`, profile §1; `cargo bloat --release -n 8`, path under external `target-dir` as on your machine).

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
| 2026-05-12 | §6.4 / §6.7: re-ran `cargo audit` and `cargo bloat --release -n 8`, refreshed summary date and bloat numbers (`.text` 7.6 MiB, PE 12.0 MiB). |
| 2026-05-13 | First version: aligned with `main` lockfile, `cargo audit` / `cargo bloat` sampling; linked KNOWN_VULNERABILITIES. |
