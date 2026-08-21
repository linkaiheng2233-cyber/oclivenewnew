# Known vulnerability tracking (Cargo / npm)

This file treats **vulnerability-level** hits from `cargo audit` on the workspace-root **`Cargo.lock`** as the single source of truth for supply-chain risk and upgrade planning. Warning-only *unmaintained*, *unsound*, and *yanked* entries do not count as vulnerabilities, but the appendix tracks their current exposure and upstream blockers; the current `cargo audit` output and [SECURITY_AUDIT_SCOPE.md](./SECURITY_AUDIT_SCOPE.md) remain authoritative for detail.

**Full doc index**: [../getting-started/DOCUMENTATION_INDEX.md](../getting-started/DOCUMENTATION_INDEX.md)  
**Lightweight profile & audit flow**: [../development/LIGHTWEIGHT_PROFILE.md](../development/LIGHTWEIGHT_PROFILE.md) §6.4

---

## Current status

| Item | Value |
|------|-----|
| **cargo-audit version** | **0.22.2** (pin this major line for comparable reports) |
| **Last scan date** | **2026-08-21** (local `cargo audit` after the compatible-range dependency refresh; 1,225 advisories loaded) |
| **Scan path** | Workspace root `Cargo.lock` |
| **Vulnerability-level count** | **0** (`cargo audit` exit code **0**; `sqlx-mysql` / `rsa` removed from lockfile graph) |
| **Warning-level count** | **7** (`gdkx11`/GTK3 · `glib` · five `unic-*` entries; `spin` is now the non-yanked 0.9.9, while other documented warnings remain tracked by the ignore policy) |

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
| [RUSTSEC-2026-0185](https://rustsec.org/advisories/RUSTSEC-2026-0185) | quinn-proto &lt; 0.11.15 | **Fixed** — lockfile **0.11.17** (2026-08-21) | Compatible-range lockfile refresh |
| [RUSTSEC-2026-0204](https://rustsec.org/advisories/RUSTSEC-2026-0204) | crossbeam-epoch 0.9.18 | **Fixed** — **0.9.20** | 2026-07-09 PR #101 CI supply chain |
| [RUSTSEC-2026-0194](https://rustsec.org/advisories/RUSTSEC-2026-0194) | quick-xml 0.39.4 | **Fixed** — **0.41.0** (via plist 1.10) | same |
| [RUSTSEC-2026-0195](https://rustsec.org/advisories/RUSTSEC-2026-0195) | quick-xml 0.39.4 | **Fixed** — **0.41.0** | same |

---

## Resolution roadmap

### Completed (2026-05-20)

- **sqlx ≥ 0.8.6**, `default-features = false`, features: `runtime-tokio-rustls`, `sqlite` (no umbrella `migrate`).
- Runtime migrations: `kernel/crates/oclive_kernel_host/src/infrastructure/sql_migrate.rs`.
- **CI**: `dimension5-acceptance` uniquely owns the main workflow's required `cargo audit`; `Cargo.lock` PRs use `cargo-audit-lockfile.yml`.

### Maintenance rules

1. After lockfile changes: `cargo audit` (or `cargo audit --no-fetch --stale` if offline).
2. Sync **vulnerability-level** changes to the table above; sync policy to [LIGHTWEIGHT_PROFILE.md §6.4](../development/LIGHTWEIGHT_PROFILE.md).
3. Do not claim “zero vulnerabilities” in outward copy; link here with actual counts.

---

## Warning-level tracking (rolling)

| RUSTSEC / category | Crate | Status | Reason |
|--------------------|-------|--------|--------|
| **RUSTSEC-2026-0002** | `lru` | **Fixed** | `oclive-cli` upgraded **ratatui 0.30** → `lru` ≥ 0.16 |
| **RUSTSEC-2025-0134** | `rustls-pemfile` | **Fixed** | `reqwest` **0.12** chain no longer depends on this crate |
| gtk-rs GTK3 cluster (10 IDs) | `gtk`/`gdk`/… | **Recorded; 9 audit.toml ignores, with `gdkx11` still reported as a warning** | Linux WebView (wry/webkit2gtk) still pulls GTK3; ignores remain after Tauri 2 until upstream shifts |
| **RUSTSEC-2025-0075 / 0080 / 0081 / 0098 / 0100** | `unic-*` 0.9 | **Open** | Transitively pulled by Tauri `urlpattern`; wait for upstream removal of the unmaintained family |
| **RUSTSEC-2026-0221** | `event-listener` 5.4.1 | **Fixed · K-SUPPLY-11** | Lockfile upgraded to **5.4.2** on 2026-08-01; both SQLx and zbus/Tauri paths now resolve to the fixed release, without an ignore |
| **RUSTSEC-2025-0057** | `fxhash` | **Cleared** | 2026-07-14 K-PLATFORM-01a Full · no `fxhash` in Tauri 2 lock graph |
| **RUSTSEC-2024-0429** | `glib` | **Open** | `VariantStrIter` path; host does not use (Linux wry) |
| yanked | `spin` 0.9.8 | **Fixed** — lockfile **0.9.9** | 2026-08-21 compatible-range lockfile refresh; still pulled through `flume` → `sqlx-sqlite` |
| **RUSTSEC-2026-0097** | `rand` 0.7 | **Cleared** | 2026-07-14 K-PLATFORM-01a Full · no `rand` 0.7 after Tauri 2 |
| **RUSTSEC-2026-0190** | `anyhow` | **Fixed** — lockfile **1.0.104** | 2026-08-21 lockfile verification |

See [`.cargo/audit.toml`](../../.cargo/audit.toml) and [SECURITY_AUDIT_SCOPE.md](./SECURITY_AUDIT_SCOPE.md).

---

## npm dependency status (2026-08-21)

The required CI `npm-audit` job runs `npm audit --omit=dev --audit-level=high`; remote run [`30692428026`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/30692428026) reported **0 production vulnerabilities**.

On frozen implementation `728219e7`, full `npm audit` and the production scan both report **0 vulnerabilities**, while `npm ls eslint eslint-plugin-unicorn` exits successfully. ESLint 10.8.0 / Antfu 9.2.0 satisfy Unicorn 72's peer contract, WebDriverIO 9.30.0 resolves fixed `fast-xml-parser` 5.10.1, and the legacy `vue3-sfc-loader` Vue 2/PostCSS chain is removed in favor of a restricted official-compiler DEV path. Remote CI [`30714475985`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/30714475985) passed npm audit and the Linux/Windows frontend gates. This is a measured point-in-time result, not a permanent zero-risk claim.

On 2026-08-21, `package-lock.json` was refreshed without changing the declared `package.json` ranges: `rollup-plugin-visualizer` now resolves to 7.1.1, `vue-tsc` to 3.3.10, and `webdriverio` to 9.31.2. Both the production-only and full local npm audits report **0 vulnerabilities**, and `npm ls --all` exits successfully. Major-version candidates remain separate migration work and are not mixed into this lockfile refresh.

---

[中文](../../creator-docs/security/KNOWN_VULNERABILITIES.md)
