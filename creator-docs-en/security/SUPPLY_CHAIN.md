# Open-source supply chain security

[中文](../../creator-docs/security/SUPPLY_CHAIN.md)

> **Positioning**: OCLive reduces supply-chain risk via **automated guardrails + verifiable releases + transparent extension points** — not by requiring every user to build from source.  
> **We do not claim**: zero vulnerabilities, full manual audit of all deps, bit-identical reproducible builds (see §4 limits).  
> **Ledger**: [TECHNICAL_DEBT_INVENTORY.md](../../handoff/TECHNICAL_DEBT_INVENTORY.md) § supply chain

**Related**: [KNOWN_VULNERABILITIES.md](KNOWN_VULNERABILITIES.md) · [SECURITY_AUDIT_SCOPE.md](SECURITY_AUDIT_SCOPE.md) · [LICENSE_POLICY.md](../LICENSE_POLICY.md) · [LIGHTWEIGHT_PROFILE.md](../development/LIGHTWEIGHT_PROFILE.md)

---

## 1. Trust model (honest boundary)

| Layer | Practice | User side |
|-------|----------|-----------|
| **Rust deps** | `Cargo.lock` + `cargo audit` + `cargo deny` | Reproducible `cargo build`; CI + KNOWN_VULN table public |
| **Official prebuilt kernel** | Release workflow + `SHA256SUMS` | Verify hash after download |
| **Third-party plugins/packs** | Release builds block inline Vue + high-risk grants; signing is not default yet | **Review `manifest` / source before run**; do not treat as trusted code |
| **Process boundary** | Kernel separate process, HTTP contract, directory plugin gates | Plugin/LLM crash ≠ hardware runaway by default |

We **do not solve** root causes like XZ-style attacks; guardrails **lower probability and improve traceability**.

---

## 2. Baseline (shipped)

| Guardrail | Location |
|-----------|----------|
| Vulnerability scan | `cargo audit` · dimension5 · `ci.yml` |
| Licenses / duplicate deps | `deny.toml` (`multiple-versions = deny` + documented `[bans.skip]`) · `cargo deny check licenses bans` · dedup ratchet · dimension5 |
| Lockfile ratchet | dimension5 blocks `sqlx-mysql` / `rsa` regression |
| Vuln SSOT | [KNOWN_VULNERABILITIES.md](KNOWN_VULNERABILITIES.md) |
| Audit scope | [SECURITY_AUDIT_SCOPE.md](SECURITY_AUDIT_SCOPE.md) |
| Plugin permissions | `plugin_permissions` / `high_risk_grants` |
| Minimum plugin-UI isolation | Release builds force HTML/custom-protocol paths; inline Vue requires DEV + `VITE_OCLIVE_UNSAFE_INLINE_PLUGIN_VUE=1` |
| Migration integrity | SQL migration checksum (`sql_migrate.rs`) |
| Plugin install review | Market/git/zip install → info toast + `installPath`; strict `OCLIVE_PLUGIN_SIGNATURE_STRICT` |
| Release hashes | `scripts/generate-sha256sums.mjs` · `release-kernel-checksums.yml` |

---

## 3. Verify Release kernel hash

GitHub Actions → **Release kernel checksums** → download `SHA256SUMS` from artifact.

**Windows (PowerShell)**:

```powershell
Get-FileHash .\oclive-kernel-server.exe -Algorithm SHA256
```

**Linux / macOS**:

```bash
sha256sum oclive-kernel-server
```

Local dev: `npm run bundle-kernel:tauri` writes `distros/desktop-tauri/resources/SHA256SUMS` (gitignored).

---

## 4. In progress / backlog (tech-debt IDs)

| ID | Item | Priority | Status |
|----|------|----------|--------|
| **K-SUPPLY-02** | Attach `SHA256SUMS` to GitHub Release | P1 | Workflow in repo; first Release asset = maintainer |
| **K-SUPPLY-03** | Plugin install review prompt | P2 | **Done** |
| **K-SUPPLY-04** | Elevate `npm-audit` | P2 | **Done · remote verified** — two production scans were clean, `continue-on-error` is removed, and remote CI [`30692428026`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/30692428026) passed the required gate |
| **K-SUPPLY-05** | `deny.toml` multiple-versions → deny | P2 | **Done** (Minimal · 2026-07-15) — `deny` + documented skips; remaining families in [LIGHTWEIGHT_PROFILE §6.6](../development/LIGHTWEIGHT_PROFILE.md); Full zero-skip is a separate campaign |
| **K-SUPPLY-09** | Plugin signature strict mode is opt-in | P1 | **OPEN** — sidecar SHA-256 is checked only with explicit `OCLIVE_PLUGIN_SIGNATURE_STRICT=1`; source-review prompts are not signature proof, and official/market signing plus revocation remain pending |
| **K-SUPPLY-10** | Pin GitHub Actions to full commit SHAs | P2 | **OPEN** — workflows currently use mutable `@v*` / `@stable` tags |
| **K-SUPPLY-11** | `event-listener` 5.4.1 unsound warning | P1 | **Done · remote verified** — lockfile uses 5.4.2, both SQLx and zbus/Tauri resolve to it, warnings fell from 9 to 8, and remote Dimension 5 passed |
| **K-SUPPLY-12** | npm development-tool vulnerabilities and peer-contract drift | **P1** | **Done · remote verified** — full and production audits are clean, the ESLint/Unicorn peer tree is valid, the WebDriver XML parser is fixed, and the legacy Vue 2/PostCSS SFC loader is removed. Remote CI [`30714475985`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/30714475985) passed npm audit plus Linux/Windows frontend gates at frozen implementation `728219e7` |
| **K-PLUGIN-SEC-01** | Per-plugin origin and native isolation E2E | P1 | **Partial** — inline Vue is blocked in releases; HTML fallbacks still share `ocliveplugin.localhost`, so this is not a complete sandbox |
| **K-SECRET-01** | Revoke historical API credential and decide history handling | **P0** | **Done (2026-07-17)** — the working tree uses a secret reference; the maintainer confirms N1N destroyed the old credential provider-side, and history is retained by decision |
| **K-SUPPLY-06** | Bit-identical reproducible builds | — | Deferred |
| **K-SUPPLY-07** | SBOM | — | Deferred |

---

## 5. Maintenance rhythm

1. PRs touching **`Cargo.lock`**: dimension5 `--ci` green + update KNOWN_VULN scan date.
2. Before release: `cargo audit` · `cargo deny` · `oclive lint --deny`.
3. Each feature cycle: revisit [SECURITY_AUDIT_SCOPE.md](SECURITY_AUDIT_SCOPE.md) limits.
4. **`npm-audit`**: main CI runs both the production-only and full development-graph high-severity gates; each must pass independently.
5. **Plugin installation**: until signing is the default, do not treat third-party plugins as trusted code; `process:spawn`, MCP, and network capabilities still require grants and user authorization. Blocking inline Vue in releases is containment, not a substitute for signing and per-plugin origins.

---

## 6. Recommendations

- **Developers**: `npm ci` + `cargo build`; avoid polluted global toolchains.
- **End users**: Prefer official Release; verify SHA256 when published; review plugin dirs before enable.
- **Hardware integrators**: Separate MCU/actuator domain from LLM process; see distro profile + HTTP contracts.
