# Known vulnerability tracking (Cargo / npm)

This file treats **vulnerability-level** hits from `cargo audit` on the workspace-root **`Cargo.lock`** as the single source of truth for supply-chain risk and upgrade planning. Warning-only *unmaintained*, *unsound*, and *yanked* entries do not count as vulnerabilities, but the appendix tracks their current exposure and upstream blockers; the current `cargo audit` output and [SECURITY_AUDIT_SCOPE.md](./SECURITY_AUDIT_SCOPE.md) remain authoritative for detail.

**Full doc index**: [../getting-started/DOCUMENTATION_INDEX.md](../getting-started/DOCUMENTATION_INDEX.md)  
**Lightweight profile & audit flow**: [../development/LIGHTWEIGHT_PROFILE.md](../development/LIGHTWEIGHT_PROFILE.md) §6.4

---

## Current status

| Item | Value |
|------|-----|
| **cargo-audit version** | **0.22.2** (pin this major line for comparable reports) |
| **Last scan date** | **2026-11-01** (local `cargo audit --json` against a refreshed advisory-db: **1,395** advisories; **698** dependency entries scanned; 0 vulnerabilities / 10 warnings; exit code **0**; this lock change only synchronizes the desktop rc.5 version and preserves the third-party dependency graph) |
| **Current npm scan** | Production **0**; full graph **3 low / 43 moderate / 0 high / 0 critical**. This new advisory snapshot does not rewrite earlier observations. Existing production/full `--audit-level=high` gates remain; no ignore expansion or unrelated dependency upgrade |
| **Scan path** | Workspace root `Cargo.lock` |
| **Vulnerability-level count** | **0** (`cargo audit --json` exit code **0**) |
| **Warning-level count** | **10 reported / 11 matched** (`glib`, five `unic-*`, `zbus_names`/`zvariant`/`zvariant_derive`, and `unsafe-libyaml`, all *unmaintained*/*unsound*; `proc-macro-error` RUSTSEC-2024-0370 remains suppressed by the [`.cargo/audit.toml`](../../.cargo/audit.toml) allowlist — see "Warning-level tracking" below) |

> If CI or your machine cannot fetch advisory-db: `cargo audit --no-fetch --stale` (requires a previously fetched local DB).

On 2026-10-29 the workspace Tokio minimum becomes **1.53.3**, and the lock moves from **1.53.2 → 1.53.3**. Only Tokio's version/checksum change in the lockfile; existing features, dependency edges and every other locked package are preserved, remediating **RUSTSEC-2026-0386**. On the same advisory snapshot, reported warnings decrease from **11 → 10**. An external no-ignore check of the identical lockfile against cached advisories reports **11**; the only additional entry remains `proc-macro-error`, and the main allowlist bytes are unchanged. No direct `AsyncFd` call was found in tracked Rust source, but this is not complete transitive-dependency/Unix reachability acceptance. Removing this advisory from the locked graph does not guarantee security for every runtime configuration. The independent kernel revision migration and installer repackaging remain separate work.

The new-frontend internal package on 2026-10-27 changes the desktop's own version from `0.6.0-rc.1` to `0.6.0-rc.2`. This Cargo.lock delta changes only that workspace package version, with no dependency, checksum, feature or ignore changes. Candidate source `b4341eb22e987ffef4a19ccad3784e2402cd2bd1` passed the release audit; the table uses its original scan. The UI update does not resolve existing warnings. Subsequent documentation-only work reuses that point-in-time evidence without claiming a new scan.

The 2026-10-27 candidate changes the desktop version to `0.6.0-rc.1` and constrains the property-test-only `proptest` dependency to `~1.11.0`, sharing `bit-set/bit-vec 0.8` with Tauri's `dom_query`. Test counts and duplicate-dependency limits are not relaxed. Existing package dependency edges are preserved; new random-number dependencies serve only this test toolchain, and no ignore entries are added. The newly observed Tokio warning concerns Unix `AsyncFd`, which is not callable in this Windows installer. It still requires separate remediation before a Unix release; Windows verification cannot establish Unix safety.

The 2026-09-27 lockfile change only updated `rustls` from 0.23.43 to 0.23.45 (version and checksum), fixing RUSTSEC-2026-0285; no other package or dependency edge changed in that batch. The earlier optional `media-png` edge belongs to the 2026-09-05 scan. This is point-in-time evidence, not a guarantee for every runtime configuration.

The separate 2026-10-07 maintenance slice updates the yanked `chacha20 0.10.1` to **0.10.2**, changing only version / checksum and preserving dependency edges and all manifests. The [upstream changelog](https://github.com/RustCrypto/stream-ciphers/blob/master/chacha20/CHANGELOG.md) describes the SSE4.1 intrinsic fix in the SSE2 backend. The post-update audit exits 0 with 0 vulnerabilities, 6 warnings, and empty stderr. The pre-update fetch/audit exits 0 but includes registry yanked-query timeouts; its original output is retained, and exit 0 does not prove every registry query completed. crates.io metadata separately confirms the new version is not yanked. This slice is batched with the selector-parser preparation; full-local and exact-SHA formal CI remain pending, with no relaxed ignore policy or npm parent-debt closure. Scope and original receipts: [DCL-51](../../handoff/debt-marathon/DEBT_CHANGELOG.md#dcl-20261007-51--兼容-chacha20-补丁与供应链合批).

On 2026-10-18 newly published upstream advisories moved the inventory from **0** to **10 vulnerability-level hits** (`dimension5-acceptance` surfaced this live on `main`; the red was a pre-existing lockfile issue and unrelated to the documentation commits landing at the time). This batch follows "upgrade what can be upgraded, never open an ignore for a fixable known vulnerability" and **adds no ignore entries**:

- `quinn-proto` **0.11.17 → 0.11.19** (RUSTSEC-2026-0338 / 0340 / 0343, two of them 7.5 high), transitive via `quinn 0.11.11 → reqwest 0.12.28`.
- `rustls` **0.23.45 → 0.23.46** (RUSTSEC-2026-0346 / 0347 / 0348 / 0349, up to 4.0 medium), transitive via `hyper-rustls → reqwest`.
- `zip` **2.4.2 → 9.0.5** (RUSTSEC-2026-0367 / 0368 / 0369, the last 9.1 critical) is a **direct** dependency of `oclive-cli` / `oclive_kernel_host` / `oclivenewnew-tauri`. All three advisories target symlink and SUID/SGID handling in `ZipArchive::extract()` / `ZipStreamReader::extract()`; this repository never calls `extract()` (it reads entries in memory via `by_index` and applies its own path check), so the runtime exposure was already narrow, but the batch upgrades instead of recording an exception.
- Feature choice: the 9.x `deflate` meta-feature switches the `flate2` backend to `zlib-rs`, which would stack on top of the `rust_backend` already requested by `png`/`oclive-cli` and produce a double backend; the workspace declaration therefore uses `deflate-flate2`, and the three member crates drop their redundant `features = ["deflate"]`. We never set a compression level, so the `zopfli` half of `deflate` is unused.
- Code adaptation: zip 9 changed `ZipFile::name()` from returning `&str` to `Result<Cow<'_, str>, ZipError>` (also part of the path-safety hardening); three sites in `role_pack.rs` and two in `lora_adapters.rs` now map invalid entry names onto the existing error shapes, without relaxing any path validation.
- Dependency graph delta: `zopfli 0.8.3` and `derive_arbitrary 1.4.2` removed, `typed-path 0.12.3` added; `flate2 1.1.9` stays on `rust_backend` (no `zlib-rs`), and dependency entries drop from 698 to **697**.
- Verification: `cargo audit --no-fetch --stale` gives **0 vulnerabilities / 10 allowed warnings / exit 0**; `cargo deny check licenses bans` reports `bans ok, licenses ok`; `cargo check` passes for all three zip consumers.
- The warning count rising from 6 to 10 is new registration for the existing graph after advisory-db grew from 1,290 to 1,338 entries (`zbus_names` / `zvariant` / `zvariant_derive` / `unsafe-libyaml`), **not** something this upgrade introduced, and none of them blocks the gate.

On 2026-10-20 two more upstream advisories (same CVE-2026-111943, emSign root trust) turned `main` red again on both `dimension5-acceptance` and `npm-audit`. This batch keeps the "upgrade what can be upgraded, never open an ignore for a fixable known vulnerability" rule and **adds no ignore entries**:

- `rustls-webpki` **0.103.14 → 0.103.16** (RUSTSEC-2026-0382, certificate verification may accept distrusted intermediates), transitive via `rustls → reqwest`; the fix requires `>= 0.103.16`.
- `webpki-roots` **1.0.9 → 1.0.10** (RUSTSEC-2026-0381, bundled roots trust distrusted emSign intermediates); the fix requires `>= 1.0.10`.
- Lockfile delta beyond those two crates is one resolver choice: `tempfile` now points at `getrandom 0.3.4` instead of `0.4.3` (all three getrandom lines coexist; `cargo metadata --locked` passes).
- npm: `vite` resolves to **6.4.5** (fixes the `<= 6.4.3` high) and `vitest` moves **4.1.11 → 4.1.12** (moderate); `npm audit --audit-level=high` exits **0**, leaving 4 low / 2 moderate that do not block the existing gate.
- Verification: local `cargo audit` → **0 vulnerabilities / 10 allowed warnings / exit 0** (1,348 advisories, 696 entries); `npm audit --package-lock-only` → 0 high. No Rust source or manifest dependency declarations changed.

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
| [RUSTSEC-2026-0338](https://rustsec.org/advisories/RUSTSEC-2026-0338) | quinn-proto 0.11.17 | **Fixed** — lockfile **0.11.19** | 2026-10-18 upgrade; zero-length STREAM frames can panic the receiver |
| [RUSTSEC-2026-0340](https://rustsec.org/advisories/RUSTSEC-2026-0340) | quinn-proto 0.11.17 | **Fixed** — lockfile **0.11.19** | same; peer minimum ACK delay can panic under ACK_FREQUENCY (7.5 high) |
| [RUSTSEC-2026-0343](https://rustsec.org/advisories/RUSTSEC-2026-0343) | quinn-proto 0.11.17 | **Fixed** — lockfile **0.11.19** | same; outgoing datagram buffer accounting can panic the sender (7.5 high) |
| [RUSTSEC-2026-0346](https://rustsec.org/advisories/RUSTSEC-2026-0346) | rustls 0.23.45 | **Fixed** — lockfile **0.23.46** | same; `Debug` impls for client session values expose TLS secrets (4.0 medium) |
| [RUSTSEC-2026-0347](https://rustsec.org/advisories/RUSTSEC-2026-0347) | rustls 0.23.45 | **Fixed** — lockfile **0.23.46** | same; buffered application data still sent to the outer server after ECH rejection (2.3 low) |
| [RUSTSEC-2026-0348](https://rustsec.org/advisories/RUSTSEC-2026-0348) | rustls 0.23.45 | **Fixed** — lockfile **0.23.46** | same; Brotli certificate decompression can allocate up to 1 GiB before authentication (3.7 low) |
| [RUSTSEC-2026-0349](https://rustsec.org/advisories/RUSTSEC-2026-0349) | rustls 0.23.45 | **Fixed** — lockfile **0.23.46** | same; an unauthenticated ServerHello can downgrade later connections to classical key exchange (3.7 low) |
| [RUSTSEC-2026-0367](https://rustsec.org/advisories/RUSTSEC-2026-0367) | zip 2.4.2 | **Fixed** — lockfile **9.0.5** | same; SUID/SGID bits preserved insecurely during extraction (7.3 high) |
| [RUSTSEC-2026-0368](https://rustsec.org/advisories/RUSTSEC-2026-0368) | zip 2.4.2 | **Fixed** — lockfile **9.0.5** | same; `ZipArchive::extract` creates symlinks whose targets escape the destination directory |
| [RUSTSEC-2026-0369](https://rustsec.org/advisories/RUSTSEC-2026-0369) | zip 2.4.2 | **Fixed** — lockfile **9.0.5** | same; `extract()` applies Unix permissions through symlinks without confining the target (9.1 critical); this repository never calls `extract()` |
| [RUSTSEC-2026-0381](https://rustsec.org/advisories/RUSTSEC-2026-0381) | webpki-roots 1.0.9 | **Fixed** — lockfile **1.0.10** | 2026-10-22 upgrade; bundled roots trust distrusted emSign intermediates |
| [RUSTSEC-2026-0382](https://rustsec.org/advisories/RUSTSEC-2026-0382) | rustls-webpki 0.103.14 | **Fixed** — lockfile **0.103.16** | 2026-10-22 upgrade; certificate verification may accept distrusted intermediates |

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
| gtk-rs GTK3 cluster (RUSTSEC-2024-0411…0420, 10 IDs) | `gtk`/`gdk`/`atk`/`gdkx11`/… | **Removed from the allowlist** | Re-checked 2026-10-19: all 10 advisories carry `withdrawn = 2026-08-14`, so `cargo audit` no longer reports them and the 9 matching `audit.toml` ignores were removed in the 2026-10-19 cleanup commit (after which `cargo audit --no-fetch` still reports `0 vulnerabilities` plus `warning: 10 allowed warnings found`). Linux WebView (wry/webkit2gtk) still pulls GTK3, but the earlier "no maintained upstream" rationale is resolved by the withdrawal |
| **RUSTSEC-2024-0370** | `proc-macro-error` 1.0.4 | **Open · allowlisted (effective)** | Pulled in via `tauri 2.11.5 → muda → gtk → glib → glib-macros` (the same Linux chain as the GTK3 cluster); no maintained upstream version, so an explicit allowlist keeps the gate readable. Exit condition: Tauri/wry moves to GTK bindings that no longer depend on `glib-macros` |
| **RUSTSEC-2024-0384** | `instant` (**no longer in the dependency graph**) | **Removed from the allowlist** | Re-checked 2026-10-19: `instant` is absent from `Cargo.lock`, so this entry suppresses nothing; it was removed from `audit.toml` in the 2026-10-19 cleanup commit |
| **RUSTSEC-2025-0075 / 0080 / 0081 / 0098 / 0100** | `unic-*` 0.9 | **Open** | Transitively pulled by Tauri `urlpattern`; wait for upstream removal of the unmaintained family |
| **RUSTSEC-2026-0221** | `event-listener` 5.4.1 | **Fixed · K-SUPPLY-11** | Lockfile upgraded to **5.4.2** on 2026-08-01; both SQLx and zbus/Tauri paths now resolve to the fixed release, without an ignore |
| **RUSTSEC-2025-0057** | `fxhash` | **Cleared** | 2026-07-14 K-PLATFORM-01a Full · no `fxhash` in Tauri 2 lock graph |
| **RUSTSEC-2024-0429** | `glib` | **Open** | `VariantStrIter` path; host does not use (Linux wry) |
| **RUSTSEC-2026-0386** | `tokio` | **Remediated in locked graph · 1.53.3** | Updated and minimum raised on 2026-10-29; refreshed audit no longer matches, with no new ignore. The Unix `AsyncFd` constructor fix does not replace platform/caller acceptance |
| yanked | `spin` 0.9.8 | **Fixed** — lockfile **0.9.9** | 2026-08-21 compatible-range lockfile refresh; still pulled through `flume` → `sqlx-sqlite` |
| yanked | `chacha20` 0.10.1 | **Locally patched; formal batch acceptance pending** | Updated to non-yanked **0.10.2** on 2026-10-07; only lock version / checksum changed, with no post-update audit hit. The 2026-09-05 observation remains historical. RNG APIs, TLS, and the ignore policy are unchanged |
| **RUSTSEC-2026-0097** | `rand` 0.7 | **Cleared** | 2026-07-14 K-PLATFORM-01a Full · no `rand` 0.7 after Tauri 2 |
| **RUSTSEC-2026-0190** | `anyhow` | **Fixed** — lockfile **1.0.104** | 2026-08-21 lockfile verification |
| **RUSTSEC-2024-0449** | `unsafe-libyaml` 0.2.11 | **Open** | Pulled in via `serde_yaml_ng`; no maintained upstream version, so removal needs a different YAML backend |
| **RUSTSEC-2026-0370 / 0371 / 0372** | `zvariant` / `zbus_names` / `zvariant_derive` 5.15.0 / 4.3.4 | **Open** | Pulled in via zbus on the Linux Tauri/wry D-Bus path; upstream migration to a maintained binding is required |

See [`.cargo/audit.toml`](../../.cargo/audit.toml) and [SECURITY_AUDIT_SCOPE.md](./SECURITY_AUDIT_SCOPE.md).

**Allowlist re-verification (2026-10-19)**: base `f8026855` (working tree carried only this documentation batch), `cargo-audit` **0.22.2**, advisory-db at `E:\Env\state\cargo-home\advisory-db` (**962** crate directories, freshly used). `cargo audit` reports `0 vulnerabilities` plus `warning: 10 allowed warnings found`, exit code **0**. To separate "suppression still effective" from "inert entry", all **11** `ignore` entries were temporarily cleared and `cargo audit --no-fetch` re-run: the reported count became **11**, and the only addition was **RUSTSEC-2024-0370** for `proc-macro-error`; none of the 9 GTK3-cluster advisories and not `instant` **0384** appeared, which reproduces the inert-entry reasons in the table above. `audit.toml` was then restored byte-for-byte (SHA256 `534DCC2A5491296480D7F5B121A27B11A2D749337A07D03C02A71A7EAFACF64D`); this records the state at verification time, and the removal of the inert entries is in the same-day cleanup commit.

---

## npm dependency status (rolling observations)

### 2026-10-29 patches after draft synchronization (target-SHA remote acceptance pending)

The first [CI run 40837019439](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/40837019439) for [draft PR #225](https://github.com/linkaiheng2233-cyber/oclivenewnew/pull/225), at `2fb8b7bd631690c06b4de2c4967d453ae79e156b`, fails the production step of `npm-audit`. Local reproduction on the same lock reports production **5 high**, and full **4 low / 2 moderate / 6 high / 0 critical**. These count affected packages, not distinct advisories. Earlier scans remain historical evidence.

| Dependency / advisory | Limited patch |
|---|---|
| Vue I18n [named-parameter escaping](https://github.com/advisories/GHSA-8rc2-428q-7ww3) and [srcdoc escaping bypass](https://github.com/advisories/GHSA-83rm-6c8r-g728) | All four manifests require `^11.4.14`; `vue-i18n` and four `@intlify` packages move **11.4.8 → 11.4.16**, within 11.x. The advisory fix starts at 11.4.14. A lock hit does not prove reachable product XSS |
| Development-only Undici | jsdom / cheerio **7.30.0 → 7.30.1**, webdriver **6.29.0 → 6.29.1**, within existing ranges; see upstream [7.30.1](https://github.com/nodejs/undici/releases/tag/v7.30.1) / [6.29.1](https://github.com/nodejs/undici/releases/tag/v6.29.1). No new override |
| Residual workspace Vitest | Three manifests move **4.1.11 → 4.1.12**, matching the existing root version. Their old copies and unreferenced old `@vitest` chain disappear; existing 4.1.12 nodes are reused, covering the [old mocker's Windows-path advisory](https://github.com/advisories/GHSA-xp26-3454-mjgp) |

Native npm **10.9.8** retained old workspace Vitest nodes during the targeted lock update. That intermediate lock and log are preserved outside the repository. Temporary official npm **11.21.0** synchronized the lock; native npm 10 then passed `npm ci --no-audit --no-fund` and `npm ls --all`, both exit **0**. The global toolchain did not change. Root Vite range metadata now matches the existing manifest's `^6.4.5` without changing its resolved version; TypeScript's `devOptional` classification is recalculated for existing Vue / Pinia optional peers, with its version and dependency edges preserved (`npm explain typescript` checked). Other package versions, four overrides, Cargo, workflows, audit thresholds and product source remain unchanged.

Patched local scans report production **0**, full **4 low / 0 moderate / 0 high / 0 critical**; both `--audit-level=high` scans exit **0**. ESLint Markdown→KaTeX low remains; this is not a zero-risk claim. Remote evidence must match the patched full SHA. A draft gate does not replace formal `ci-gate`; protected main is unmerged and installers are not rebuilt. The independent kernel's source prerelease and audit belong to another repository and do not validate this workspace.

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

### 2026-10-07 temporary shell-quote compatibility exception

The preceding `67baa952bb7e1ff0ba987609d6fe729ca327f24d` batch passed the full local chain with native exit 0, but the full audit in [formal CI 37505631986](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37505631986) hit newly recorded [GHSA-pqg4-j6r4-53mv](https://github.com/advisories/GHSA-pqg4-j6r4-53mv). The run ultimately concluded cancelled; formal acceptance was not achieved. That snapshot was production 0 and full **4 low / 2 critical**. The two critical package counts, shell-quote and concurrently, derive from one advisory, not two independent vulnerabilities. Both lock nodes were unchanged from the preceding accepted baseline; the new audit result was not introduced by the selector-parser or ChaCha20 patches.

| Temporary exception | Scope, validation, and removal condition |
|---|---|
| `overrides.concurrently.shell-quote = "1.11.0"` | Explicitly approved by the maintainer; only `concurrently 9.2.4 → shell-quote` is overridden, preserving the other four overrides. Upstream 9.2.4 / 10.0.5 still pin 1.9.0, so updating the parent alone does not remove the finding. No concurrently major upgrade; only this lock node's version/resolved/integrity change |
| Local compatibility evidence | Original npm 10 `npm ci --ignore-scripts --no-audit --no-fund` and `npm ls --all` exit 0. Real benign concurrently children both succeed with CLI exit 0; a child exit 7 produces CLI exit 1. Production argument expansion and representative quote/parse behavior remain. Four line terminators following a comment token, tested only in memory, change from old-version acceptance to TypeError rejection; no attack string is executed in a shell |
| Current audit snapshot | Production **0**, full **4 low / 0 moderate / 0 high / 0 critical**; both `--audit-level=high` commands exit 0. ESLint Markdown→KaTeX low findings remain tracked. This is evidence for the current lock and scan time, not a product-wide zero-risk guarantee |
| Owner and removal | The project dependency maintainer updates concurrently and its chain once the supported upstream line adopts safe shell-quote, removes this override, and checks the lock, npm ci/peers/audit, build behavior, and exit propagation. A new upstream release alone is insufficient to remove it; other exceptions are not updated automatically |

This patch was frozen at `1e81033d6c8efcff8bb9f502c642560e57335cf1`: one full local chain exited 0 (1491.7835099 seconds), and [formal CI 37513747676](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37513747676) completed attempt 1 with 17/17 success, including ci-gate, without a rerun. The parent debt remains Partial while the upstream removal condition and KaTeX low findings retain their respective scopes; an audit count of 0 is not treated as Done. Approval, cause, and original receipt locations: [DCL-53](../../handoff/debt-marathon/DEBT_CHANGELOG.md#dcl-20261007-53--shell-quote-临时兼容补丁与撤销条件). This result is backfilled alongside the substantive resource-reason integration in [DCL-54](../../handoff/debt-marathon/DEBT_CHANGELOG.md#dcl-20261007-54--资源原因说明合流与已验基线确认).

### 2026-10-09 upstream adoption and temporary-override removal

The maintainer required removal of the temporary patch once upstream adopts a safe version. The [official concurrently 9.2.5 release](https://github.com/open-cli-tools/concurrently/releases/tag/v9.2.5) changes its exact shell-quote dependency from 1.9.0 to **1.12.0**, within this repository's unchanged `^9.2.1` range. Official npm metadata matches the generated lock. This slice updates the parent **9.2.4→9.2.5** and leaf **1.11.0→1.12.0**, removing only `overrides.concurrently`; the other four overrides, declarations and scripts remain. Exactly these two lock nodes change. The original 2026-10-07 approval, scans and failures remain historical, not a statement that this override is still required.

Original npm 10 `npm ci --ignore-scripts --no-audit --no-fund` / `npm ls --all`, the existing representative quote/parse and production argument-expansion probe, real benign child successes (CLI 0), child failure propagation (7→CLI 1), and the actual production build all exit 0. Four line terminators after a comment still produce TypeError in memory; no attack string is sent to a shell. Temporary official npm 11.21.0 generates the targeted lock without changing the global toolchain. Deprecation messages and missing optional dependencies are retained as observed, not called peer failures or zero stderr.

Pre/post production scans both report **0**; full scans both report **4 low / 0 moderate / 0 high / 0 critical**, with both high gates exiting 0. ESLint Markdown→KaTeX low findings and other overrides keep their own scopes; K-SUPPLY-12 remains **Partial**. These scans apply only to this lock and observation time. The full local exit and formal CI for the new SHA are pending batch freeze, and earlier-SHA success is not substituted. Scope, local receipts and stopping condition: [DCL-73](../../handoff/debt-marathon/DEBT_CHANGELOG.md#dcl-20261009-73--上游安全补丁与临时-override-撤销).

### Previously verified observation (2026-08-21)

The required CI `npm-audit` job runs `npm audit --omit=dev --audit-level=high`; remote run [`30692428026`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/30692428026) reported **0 production vulnerabilities**.

On frozen implementation `728219e7`, full `npm audit` and the production scan both report **0 vulnerabilities**, while `npm ls eslint eslint-plugin-unicorn` exits successfully. ESLint 10.8.0 / Antfu 9.2.0 satisfy Unicorn 72's peer contract, WebDriverIO 9.30.0 resolves fixed `fast-xml-parser` 5.10.1, and the legacy `vue3-sfc-loader` Vue 2/PostCSS chain is removed in favor of a restricted official-compiler DEV path. Remote CI [`30714475985`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/30714475985) passed npm audit and the Linux/Windows frontend gates. This is a measured point-in-time result, not a permanent zero-risk claim.

On 2026-08-21, `package-lock.json` was refreshed without changing the declared `package.json` ranges: `rollup-plugin-visualizer` now resolves to 7.1.1, `vue-tsc` to 3.3.10, and `webdriverio` to 9.31.2. Both the production-only and full local npm audits report **0 vulnerabilities**, and `npm ls --all` exits successfully. Major-version candidates remain separate migration work and are not mixed into this lockfile refresh.

---

[中文](../../creator-docs/security/KNOWN_VULNERABILITIES.md)
