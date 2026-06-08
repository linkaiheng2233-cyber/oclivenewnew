# Open-source licensing policy (host & plugins)

Minimal licensing policy for the current phase: actionable before release, room to grow later.

[中文](../creator-docs/LICENSE_POLICY.md)

---

## 1) Main app repo (oclivenewnew)

- **License today**: **Apache License 2.0 (Apache-2.0)**; root `LICENSE` is the official text and `NOTICE` holds copyright attribution.
- Public docs should state SPDX **`Apache-2.0`**.
- Before each release: verify root `LICENSE` and `NOTICE` exist and are intact.

**Change note (2026-06-09)**: the host moved from AGPL-3.0 + plugin exception to Apache-2.0 so commercial and embedded downstreams can combine the kernel freely; under a permissive license, independent plugins and distros pick their own licenses without a separate exception appendix.

---

## 2) Official plugins (to be added)

Defaults:

1. Each official plugin repo/folder ships its own `LICENSE` (may differ from the host; samples often use permissive licenses for redistribution).
2. Put a license line near the top of README (e.g. `SPDX-License-Identifier: MIT` or author’s choice).
3. Add “license file exists” to the release checklist.

> Today `npm run scaffold:ui-plugin` still emits an MIT‑style `LICENSE` by default to reduce misses; third parties may replace with their own.

---

## 3) Community third‑party plugins

- Authors pick the license (MIT / Apache-2.0 / GPL, …).
- Marketplace pages should **display** the license with a short notice — no deep legal automation yet.
- If missing, show **“license not declared”**.

---

## 4) Relation to security work

- **This phase**: labeling + release checks (cheap, shippable now).
- **Next phase**: stronger security (permission tiers, signing/provenance, sandboxing, …).

---

## 5) Minimal pre‑ship checklist (license slice)

1. Main repo has root `LICENSE`.
2. New/updated official plugins include `LICENSE`.
3. Docs/marketplace surfaces show the license.

From repo root:

```bash
npm run check:license
```

Checks main `LICENSE` and default official `mumu` plugin `LICENSE` files.
