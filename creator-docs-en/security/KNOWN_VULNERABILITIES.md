# Known vulnerability tracking (`cargo-audit`)

This file records **vulnerability-level** hits from **`cargo audit`** on the **workspace root `Cargo.lock`**, as the single source of truth for supply-chain risk and upgrade planning. It **does not** include `cargo audit` entries reported only as *warning* (*unmaintained* / *unsound*; see full `cargo audit` output and [SECURITY_AUDIT_SCOPE.md](./SECURITY_AUDIT_SCOPE.md)).

**Full doc index**: [../getting-started/DOCUMENTATION_INDEX.md](../getting-started/DOCUMENTATION_INDEX.md)  
**Lightweight profile & audit flow**: [../development/LIGHTWEIGHT_PROFILE.md](../development/LIGHTWEIGHT_PROFILE.md) §6.4

---

## Current status

| Item | Value |
|------|-----|
| **cargo-audit version** | **0.22.1** (pin this major line for comparable reports) |
| **Last scan date** | **2026-07-14** (local `cargo audit`; after K-SUPPLY-05 lock update) |
| **Scan path** | Workspace root `Cargo.lock` |
| **Vulnerability-level count** | **0** (`cargo audit` exit code **0**; `sqlx-mysql` / `rsa` removed from lockfile graph) |
| **Warning-level count** | **3** (`fxhash` · `glib` · `rand` 0.7; gtk-rs cluster see `.cargo/audit.toml` ignore **11** entries) |

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
| [RUSTSEC-2026-0204](https://rustsec.org/advisories/RUSTSEC-2026-0204) | crossbeam-epoch 0.9.18 | **Fixed** — **0.9.20** | 2026-07-09 PR #101 CI supply chain |
| [RUSTSEC-2026-0194](https://rustsec.org/advisories/RUSTSEC-2026-0194) | quick-xml 0.39.4 | **Fixed** — **0.41.0** (via plist 1.10) | same |
| [RUSTSEC-2026-0195](https://rustsec.org/advisories/RUSTSEC-2026-0195) | quick-xml 0.39.4 | **Fixed** — **0.41.0** | same |

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

## Warning-level tracking (2026-05-20 batch three)

| RUSTSEC / category | Crate | Status | Reason |
|--------------------|-------|--------|--------|
| **RUSTSEC-2026-0002** | `lru` | **Fixed** | `oclive-cli` upgraded **ratatui 0.30** → `lru` ≥ 0.16 |
| **RUSTSEC-2025-0134** | `rustls-pemfile` | **Fixed** | `reqwest` **0.12** chain no longer depends on this crate |
| gtk-rs GTK3 cluster (11 IDs) | `gtk`/`gdk`/… | **Recorded + audit.toml ignore** | Tauri 1.x / wry Linux WebView; needs Tauri 2 to remove |
| **RUSTSEC-2025-0057** | `fxhash` | **Open** | via Tauri HTML parse; no direct API |
| **RUSTSEC-2024-0429** | `glib` | **Open** | `VariantStrIter` path; host does not use |
| **RUSTSEC-2026-0097** | `rand` 0.7 | **Open** | via `phf`/Tauri macros; needs upstream Tauri 2 |
| **RUSTSEC-2026-0190** | `anyhow` | **Fixed** — lockfile **1.0.103** | 2026-07-14 K-SUPPLY-05 `cargo update` |

See [`.cargo/audit.toml`](../../.cargo/audit.toml) and [SECURITY_AUDIT_SCOPE.md](./SECURITY_AUDIT_SCOPE.md).

---

[中文](../../creator-docs/security/KNOWN_VULNERABILITIES.md)
