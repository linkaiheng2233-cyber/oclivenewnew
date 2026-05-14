# Known vulnerability tracking (`cargo-audit`)

This file records **vulnerability-level** hits from **`cargo audit`** on **`src-tauri/Cargo.lock`**, as the single source of truth for supply-chain risk and upgrade planning. It **does not** include `cargo audit` entries reported only as *warning* (*unmaintained* / *unsound*; see full `cargo audit` output and [SECURITY_AUDIT_SCOPE.md](./SECURITY_AUDIT_SCOPE.md)).

**Full doc index**: [../getting-started/DOCUMENTATION_INDEX.md](../getting-started/DOCUMENTATION_INDEX.md)  
**Lightweight profile & audit flow**: [../../creator-docs/development/LIGHTWEIGHT_PROFILE.md](../../creator-docs/development/LIGHTWEIGHT_PROFILE.md) §6.4

---

## Current status

| Item | Value |
|------|-------|
| **cargo-audit version** | **0.22.1** (pin this major line for comparable reports) |
| **Last scan date** | **2026-05-13** (local, `--no-fetch --stale` + cached `~/.cargo/advisory-db`) |
| **Scan path** | `src-tauri/Cargo.lock` |
| **Vulnerability-level count** | **5** (per `cargo audit` `error: N vulnerabilities found`) |
| **Warning-level count** | **17** (not listed below; includes gtk-rs *unmaintained*, *unsound*, etc.) |

> If CI or your machine cannot fetch advisory-db: `cargo audit --no-fetch --stale` (requires a previously fetched local DB).

---

## Vulnerability list (vulnerability level)

| RUSTSEC ID | Crate | Version (lockfile) | Risk / CVSS | Summary | Mitigation direction | Status |
|------------|-------|--------------------|-------------|---------|----------------------|--------|
| [RUSTSEC-2023-0071](https://rustsec.org/advisories/RUSTSEC-2023-0071) | **rsa** | 0.9.10 | Medium / **5.9** | Marvin Attack: timing side channel may recover keys | Pulled via **sqlx-mysql**; upgrade **sqlx ≥ 0.8** and tighten features to avoid pulling `rsa` | Tracking |
| [RUSTSEC-2026-0098](https://rustsec.org/advisories/RUSTSEC-2026-0098) | **rustls-webpki** | 0.101.7 | see advisory | URI name constraint handling bug | Align **rustls** / **sqlx** upgrade chain per advisory | Tracking |
| [RUSTSEC-2026-0099](https://rustsec.org/advisories/RUSTSEC-2026-0099) | **rustls-webpki** | 0.101.7 | see advisory | Wildcard cert name constraint bug | Same as above | Tracking |
| [RUSTSEC-2026-0104](https://rustsec.org/advisories/RUSTSEC-2026-0104) | **rustls-webpki** | 0.101.7 | see advisory | Reachable panic in CRL parsing | Same (need **≥0.103.13** etc. per advisory) | Tracking |
| [RUSTSEC-2024-0363](https://rustsec.org/advisories/RUSTSEC-2024-0363) | **sqlx** | 0.7.4 | see advisory | Binary protocol truncation/overflow misinterpretation | Upgrade to **≥ 0.8.1** (per advisory) | Tracking |

**Dependency summary** (2026-05-13 `cargo audit` output):

- **rsa** ← `sqlx-mysql` ← `sqlx` / `sqlx-macros-core` ← `oclivenewnew-tauri`
- **rustls-webpki** ← `rustls` ← `sqlx-core` ← `sqlx` / sqlx sub-crates
- **sqlx** direct dependency of the app crate

---

## Resolution roadmap

### Short term (this round)

- Tracked in-repo: **this file** + cross-ref [LIGHTWEIGHT_PROFILE.md §6.4](../../creator-docs/development/LIGHTWEIGHT_PROFILE.md).
- **Optional mitigation**: in `Cargo.toml` set `sqlx` `default-features = false` and enable only **`runtime-*` + `sqlite` + `migrate` + `macros`** (or what the app actually needs) to reduce chance of **MySQL / PostgreSQL** transitive deps in the lockfile (validate with full `cargo test`).

### Mid term (next feature cycle)

- **Prioritize upgrading `sqlx` to 0.8.1+** for RUSTSEC-2024-0363, then re-run `cargo audit` to see if **rsa / rustls-webpki** clears or downgrades.
- Align **reqwest** / **native TLS** vs **rustls** versions to avoid multiple **webpki** lines.

### Long term

- **CI**: `cargo audit` job exists (`continue-on-failure: true`); tighten to **`--deny warnings`** or at least **`--deny unmaintained`** in stages after dependency cleanup.
- Maintenance: each cycle end, run `cargo audit` under `src-tauri` (or from CI artifacts) and **refresh dates, versions, and row counts** in this table.

---

## Maintenance rules

1. After lockfile changes or dependency upgrades:  
   `cd src-tauri && cargo audit`  
   If network-blocked: `cargo audit --no-fetch --stale`
2. Sync **vulnerability-level** changes to the table above; sync policy changes to [LIGHTWEIGHT_PROFILE.md §6.4](../../creator-docs/development/LIGHTWEIGHT_PROFILE.md).
3. Do not claim “zero vulnerabilities” in outward copy; use **“known vulnerabilities under tracking”** and link here.
