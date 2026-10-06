# Known vulnerability tracking (Cargo / npm)

This file treats **vulnerability-level** hits from `cargo audit` on the workspace-root **`Cargo.lock`** as the single source of truth for supply-chain risk and upgrade planning. Warning-only *unmaintained*, *unsound*, and *yanked* entries do not count as vulnerabilities, but the appendix tracks their current exposure and upstream blockers; the current `cargo audit` output and [SECURITY_AUDIT_SCOPE.md](./SECURITY_AUDIT_SCOPE.md) remain authoritative for detail.

**Full doc index**: [../getting-started/DOCUMENTATION_INDEX.md](../getting-started/DOCUMENTATION_INDEX.md)  
**Lightweight profile & audit flow**: [../development/LIGHTWEIGHT_PROFILE.md](../development/LIGHTWEIGHT_PROFILE.md) §6.4

---

## Current status

| Item | Value |
|------|-----|
| **cargo-audit version** | **0.22.2** (pin this major line for comparable reports) |
| **Last scan date** | **2026-10-07** (local `cargo audit --no-fetch --stale --json`; the DB was fetched earlier in this batch: 1,290 advisories, 698 dependency entries scanned) |
| **Scan path** | Workspace root `Cargo.lock` |
| **Vulnerability-level count** | **0** (`cargo audit` exit code **0**; `sqlx-mysql` / `rsa` removed from lockfile graph) |
| **Warning-level count** | **6** (`glib` · five `unic-*` entries; the `chacha20` yanked warning is removed, with the ignore policy unchanged) |

> If CI or your machine cannot fetch advisory-db: `cargo audit --no-fetch --stale` (requires a previously fetched local DB).

The 2026-09-27 lockfile change only updated `rustls` from 0.23.43 to 0.23.45 (version and checksum), fixing RUSTSEC-2026-0285; no other package or dependency edge changed in that batch. The earlier optional `media-png` edge belongs to the 2026-09-05 scan. This is point-in-time evidence, not a guarantee for every runtime configuration.

The separate 2026-10-07 maintenance slice updates the yanked `chacha20 0.10.1` to **0.10.2**, changing only version / checksum and preserving dependency edges and all manifests. The [upstream changelog](https://github.com/RustCrypto/stream-ciphers/blob/master/chacha20/CHANGELOG.md) describes the SSE4.1 intrinsic fix in the SSE2 backend. The post-update audit exits 0 with 0 vulnerabilities, 6 warnings, and empty stderr. The pre-update fetch/audit exits 0 but includes registry yanked-query timeouts; its original output is retained, and exit 0 does not prove every registry query completed. crates.io metadata separately confirms the new version is not yanked. This slice is batched with the selector-parser preparation; full-local and exact-SHA formal CI remain pending, with no relaxed ignore policy or npm parent-debt closure. Scope and original receipts: [DCL-51](../../handoff/debt-marathon/DEBT_CHANGELOG.md#dcl-20261007-51--兼容-chacha20-补丁与供应链合批).

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
| [RUSTSEC-2026-0285](https://rustsec.org/advisories/RUSTSEC-2026-0285) | rustls 0.23.43 | **Fixed** — lockfile **0.23.45** | Compatible patch update on 2026-09-27; TLS 1.3 handshake encryption-level boundary |
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
| yanked | `chacha20` 0.10.1 | **Locally patched; formal batch acceptance pending** | Updated to non-yanked **0.10.2** on 2026-10-07; only lock version / checksum changed, with no post-update audit hit. The 2026-09-05 observation remains historical. RNG APIs, TLS, and the ignore policy are unchanged |
| **RUSTSEC-2026-0097** | `rand` 0.7 | **Cleared** | 2026-07-14 K-PLATFORM-01a Full · no `rand` 0.7 after Tauri 2 |
| **RUSTSEC-2026-0190** | `anyhow` | **Fixed** — lockfile **1.0.104** | 2026-08-21 lockfile verification |

See [`.cargo/audit.toml`](../../.cargo/audit.toml) and [SECURITY_AUDIT_SCOPE.md](./SECURITY_AUDIT_SCOPE.md).

---

## npm dependency status (rolling observations)

### 2026-10-06 advisory fixes (locally verified; target-SHA CI pending)

The production audit in [CI 37424944008](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37424944008) on `8cd7d5d1` reported three high entries. A local full scan also reported critical development-test dependencies. This is not attributed to resource-policy code, and an earlier zero scan cannot establish current absence of risk.

| Advisory / dependency | Current fix / scope |
|---|---|
| [GHSA-g2v6-rqmx-r4w6](https://github.com/advisories/GHSA-g2v6-rqmx-r4w6) · Vue SSR attribute-name validation | Vue/compiler/renderer aligned to **3.5.43** on the existing 3.5 patch line; the advisory's fixed minimum is 3.5.42. A lockfile finding is not proof of a reachable SSR exploit in this product |
| [GHSA-68fv-2mgg-jv7q](https://github.com/advisories/GHSA-68fv-2mgg-jv7q) · source-map-js | **1.2.2**, a patch permitted by its existing transitive range, without overrides |
| [GHSA-5gmw-xhrv-c9v3](https://github.com/advisories/GHSA-5gmw-xhrv-c9v3) / [GHSA-85c8-ppgw-ccpr](https://github.com/advisories/GHSA-85c8-ppgw-ccpr) · Tinypool | Vitest 3.2.7's 1.x chain has no compatible fix. The root and all three workspaces pin **Vitest 4.1.11**; this tree removes tinypool and fixes the known mocker finding. No Vitest 5 migration, peer bypass, or audit relaxation |

Original npm 10.9.8 lock generation hit an Arborist `edgesOut` null error. Official npm 11.21.0 was used temporarily to generate the lock; original npm 10 `npm ci` and `npm ls --all` then exited 0. The global toolchain and CI configuration remain unchanged. Local production JSON audit reports **0**; the full graph reports **4 low / 1 moderate / 0 high / 0 critical**, with both `--audit-level=high` gates exiting 0. ESLint Markdown→KaTeX and postcss-selector-parser remain tracked; this is not a zero-vulnerability claim for the full graph. Results are specific to this lock and scan time. Validation and final evidence are recorded in [DCL-48](../../handoff/debt-marathon/DEBT_CHANGELOG.md#dcl-20261006-48--新披露-npm-风险与测试链修复).

**Exact formal result added later:** `3eb1ca7cafabedd739075c6a506fa9ecfcb95425` completed [37427668306](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37427668306) with success, 17/17 successful jobs including ci-gate, and native 0 full-local validation. The preceding 4 low / 1 moderate observation belongs to that frozen lock; later patch numbers do not rewrite it.

### 2026-10-06 single selector-parser patch (local preparation, no main acceptance)

The actual chain is `@antfu/eslint-config → eslint-plugin-vue → postcss-selector-parser 7.1.5`. [GHSA-rj75-hqrm-r3gf](https://github.com/advisories/GHSA-rj75-hqrm-r3gf) names **7.1.6** as fixed; the [upstream release](https://github.com/postcss/postcss-selector-parser/releases/tag/7.1.6) addresses quadratic flat-selector parsing. An isolated worktree updates only this lock node's version/resolved/integrity. Existing ranges, package declarations, peers, and toolchains stay unchanged, without overrides or the low findings' major/downgrade suggestion. A lock finding is not evidence of a product exploit path.

Original npm 10 consumed it with `npm ci --ignore-scripts --no-audit --no-fund`, and full `npm ls --all` exited 0. Production audit reports **0**; the full graph reports **4 low / 0 moderate / 0 high / 0 critical**. This removes only that moderate entry; the ESLint Markdown→KaTeX low chain remains. This slice is local preparation: subsequent batch-wide checks and exact-SHA formal CI have not run, and the previous slice's formal green is not substituted. See [DCL-50](../../handoff/debt-marathon/DEBT_CHANGELOG.md#dcl-20261006-50--selector-parser-单叶补丁预备片).

### Previously verified observation (2026-08-21)

The required CI `npm-audit` job runs `npm audit --omit=dev --audit-level=high`; remote run [`30692428026`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/30692428026) reported **0 production vulnerabilities**.

On frozen implementation `728219e7`, full `npm audit` and the production scan both report **0 vulnerabilities**, while `npm ls eslint eslint-plugin-unicorn` exits successfully. ESLint 10.8.0 / Antfu 9.2.0 satisfy Unicorn 72's peer contract, WebDriverIO 9.30.0 resolves fixed `fast-xml-parser` 5.10.1, and the legacy `vue3-sfc-loader` Vue 2/PostCSS chain is removed in favor of a restricted official-compiler DEV path. Remote CI [`30714475985`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/30714475985) passed npm audit and the Linux/Windows frontend gates. This is a measured point-in-time result, not a permanent zero-risk claim.

On 2026-08-21, `package-lock.json` was refreshed without changing the declared `package.json` ranges: `rollup-plugin-visualizer` now resolves to 7.1.1, `vue-tsc` to 3.3.10, and `webdriverio` to 9.31.2. Both the production-only and full local npm audits report **0 vulnerabilities**, and `npm ls --all` exits successfully. Major-version candidates remain separate migration work and are not mixed into this lockfile refresh.

---

[中文](../../creator-docs/security/KNOWN_VULNERABILITIES.md)
