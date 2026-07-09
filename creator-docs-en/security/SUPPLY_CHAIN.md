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
| **Third-party plugins/packs** | Source on disk + high-risk grants | **Review `manifest` / source before run** |
| **Process boundary** | Kernel separate process, HTTP contract, directory plugin gates | Plugin/LLM crash ≠ hardware runaway by default |

We **do not solve** root causes like XZ-style attacks; guardrails **lower probability and improve traceability**.

---

## 2. Baseline (shipped)

| Guardrail | Location |
|-----------|----------|
| Vulnerability scan | `cargo audit` · dimension5 · `ci.yml` |
| Licenses / duplicate deps | `deny.toml` · `cargo deny check licenses bans` |
| Lockfile ratchet | dimension5 blocks `sqlx-mysql` / `rsa` regression |
| Vuln SSOT | [KNOWN_VULNERABILITIES.md](KNOWN_VULNERABILITIES.md) |
| Audit scope | [SECURITY_AUDIT_SCOPE.md](SECURITY_AUDIT_SCOPE.md) |
| Plugin permissions | `plugin_permissions` / `high_risk_grants` |
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
| **K-SUPPLY-04** | Elevate `npm-audit` | P2 | **Observe** |
| **K-SUPPLY-05** | `deny.toml` multiple-versions → deny | P2 | **OPEN** |
| **K-SUPPLY-06** | Bit-identical reproducible builds | — | Deferred |
| **K-SUPPLY-07** | SBOM | — | Deferred |

---

## 5. Maintenance rhythm

1. PRs touching **`Cargo.lock`**: dimension5 `--ci` green + update KNOWN_VULN scan date.
2. Before release: `cargo audit` · `cargo deny` · `oclive lint --deny`.
3. Each feature cycle: revisit [SECURITY_AUDIT_SCOPE.md](SECURITY_AUDIT_SCOPE.md) limits.
4. **`npm-audit`**: CI `continue-on-error: true` today; escalation per K-SUPPLY-04.

---

## 6. Recommendations

- **Developers**: `npm ci` + `cargo build`; avoid polluted global toolchains.
- **End users**: Prefer official Release; verify SHA256 when published; review plugin dirs before enable.
- **Hardware integrators**: Separate MCU/actuator domain from LLM process; see distro profile + HTTP contracts.
