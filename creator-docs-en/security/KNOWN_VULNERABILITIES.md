# Known vulnerability tracking (`cargo-audit`)

This file records **vulnerability-level** hits from **`cargo audit`** on the **workspace root `Cargo.lock`**, as the single source of truth for supply-chain risk and upgrade planning. It **does not** include `cargo audit` entries reported only as *warning* (*unmaintained* / *unsound*; see full `cargo audit` output and [SECURITY_AUDIT_SCOPE.md](./SECURITY_AUDIT_SCOPE.md)).

**Full doc index**: [../getting-started/DOCUMENTATION_INDEX.md](../getting-started/DOCUMENTATION_INDEX.md)  
**Lightweight profile & audit flow**: [../development/LIGHTWEIGHT_PROFILE.md](../development/LIGHTWEIGHT_PROFILE.md) §6.4

---

## Current status

| Item | Value |
|------|-----|
| **cargo-audit version** | **0.22.1** (pin this major line for comparable reports) |
| **Last scan date** | **2026-07-12** (local `cargo audit`; `crossbeam-epoch` **0.9.20** · `plist` **1.10.0** · `quick-xml` **0.41.0**) |
| **Scan path** | Workspace root `Cargo.lock` |
| **Vulnerability-level count** | **0** (`cargo audit` exit code **0**; `sqlx-mysql` / `rsa` removed from lockfile graph) |
| **Warning-level count** | **3** (`cargo audit` + [`.cargo/audit.toml`](../../.cargo/audit.toml) ignores **11** gtk-rs GTK3 / toolchain *unmaintained*; see table below) |

> If CI or your machine cannot fetch advisory-db: `cargo audit --no-fetch --stale` (requires a previously fetched local DB).

---

## Vulnerability list (vulnerability level)

| RUSTSEC ID | Crate | Status | Notes |
|------------|-------|--------|-------|
| [RUSTSEC-2023-0071](https://rustsec.org/advisories/RUSTSEC-2023-0071) | `rsa` via `sqlx-mysql` | **Cleared** | workspace uses `sqlx-sqlite` only via `oclive_sqlx` |
| [RUSTSEC-2026-0098](https://rustsec.org/advisories/RUSTSEC-2026-0098) | rustls-webpki 0.101 | **Cleared** | |
| [RUSTSEC-2026-0099](https://rustsec.org/advisories/RUSTSEC-2026-0099) | rustls-webpki 0.101 | **Cleared** | |
| [RUSTSEC-2026-0104](https://rustsec.org/advisories/RUSTSEC-2026-0104) | rustls-webpki 0.101 | **Cleared** | |
| [RUSTSEC-2024-0363](https://rustsec.org/advisories/RUSTSEC-2024-0363) | sqlx 0.7.4 | **Cleared** — upgraded to **0.8.6** | |
| [RUSTSEC-2026-0185](https://rustsec.org/advisories/RUSTSEC-2026-0185) | quinn-proto &lt; 0.11.15 | **Fixed** — lockfile **0.11.15** (2026-06-24) | |

---

## Resolution roadmap

### Completed (2026-05-20)

- **sqlx ≥ 0.8.6**, `default-features = false`, features: `runtime-tokio-rustls`, `sqlite` (no umbrella `migrate`).
- Runtime migrations: `kernel/crates/oclive_kernel_host/src/infrastructure/sql_migrate.rs`.
- **CI**: `cargo-audit` job fails on vulnerability-level hits; `Cargo.lock` PRs use `cargo-audit-lockfile.yml`.

### Maintenance rules

1. After lockfile changes: `cargo audit` (or `cargo audit --no-fetch --stale` if offline).
2. Sync **vulnerability-level** changes to the table above; sync policy to [LIGHTWEIGHT_PROFILE.md §6.4](../development/LIGHTWEIGHT_PROFILE.md).
3. Do not claim “zero vulnerabilities” in outward copy; link here with actual counts.

---

## Warning-level tracking

| RUSTSEC / category | Crate | Status | Reason |
|--------------------|-------|--------|--------|
| gtk-rs GTK3 cluster (11 IDs) | `gtk`/`gdk`/… | **Recorded + audit.toml ignore** | Tauri 1.x / wry Linux WebView |
| **RUSTSEC-2025-0057** | `fxhash` | **Open** | via Tauri HTML parse |
| **RUSTSEC-2024-0429** | `glib` | **Open** | transitive |
| **RUSTSEC-2026-0097** | `rand` 0.7 | **Open** | via `phf`/Tauri macros |

See [`.cargo/audit.toml`](../../.cargo/audit.toml) and [SECURITY_AUDIT_SCOPE.md](./SECURITY_AUDIT_SCOPE.md).

---

[中文](../../creator-docs/security/KNOWN_VULNERABILITIES.md)
