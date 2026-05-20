# Oclive architecture overview (single-kernel, dual-mode build)

This page is the **public architecture narrative**, defines **single-kernel, dual-mode build architecture**, and lists **characteristics**. Implementation details remain in [PLUGIN_V1.md](../../creator-docs/plugin-and-architecture/PLUGIN_V1.md), [PURE_KERNEL_BOUNDARY.md](../../creator-docs/getting-started/PURE_KERNEL_BOUNDARY.md), [RFC_OCLIVE_MONOLITH_MODE.md](../../creator-docs/rfc/RFC_OCLIVE_MONOLITH_MODE.md), and source.

[中文](../../creator-docs/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md)

---

## Architecture in brief

**Oclive** uses a **contract-first thin kernel**: the kernel owns turn orchestration (`process_message`), session state, and cross-host error semantics; memory, emotion, event, prompt, LLM, and agent capabilities attach via **PLUGIN_V1 seven backend slots**—builtin (incl. v2), Remote JSON-RPC, and directory process plugins.

**Delivery** follows **distribution-style discipline**: stable HTTP / **OOCP** black-box contracts, **role pack** specs, and the **`oclive-cli` kernel factory** ship **headless kernels** (`--api` / `kernel_server`) or the **desktop host** (Tauri + Vue). Role content is the single integration surface at `roles/{roleId}/`, decoupled from the pack editor and launcher.

**Build** uses **single-kernel, dual-mode build architecture**: **one** orchestration semantics and DTO contract (single kernel), with two **compile-time tiers**—**exo-mode** (low coupling, `PluginHost` dynamic resolution) and **macro-mode** (Monolith compile-time weld, optional all-seven-slot weld). `oclive init` scaffolds dual `[[bin]]` projects; `oclive bench` and OOCP provide parity checks—you **choose the artifact per product**, not two kernel products. Exo-mode favors replaceability and ecosystem experiments; macro-mode favors static hot paths and single-binary delivery on latency-sensitive devices (engineering analogy, not OS taxonomy).

The **open lab** product axis: local-first, contracts and CI guard compatibility—creators swap implementation layers without forking core orchestration ([VISION_OPEN_LAB.md](../../creator-docs/roadmap/VISION_OPEN_LAB.md)).

---

## Single-kernel, dual-mode build architecture

| Term | Meaning |
|------|---------|
| **Single kernel** | One orchestration core: `process_message` order and `reply` / `KernelErrorBody` contracts—not “single CPU core,” not two dialogue engines. |
| **Dual-mode** | Two **build tiers**, coexisting long-term ([RFC_OCLIVE_MONOLITH_MODE.md](../../creator-docs/rfc/RFC_OCLIVE_MONOLITH_MODE.md)). |
| **Build** | Tier chosen at **`oclive init` + `monolith.toml` + `cargo build`**; typical outputs are **standard** and **`-monolith`** binaries—**not** runtime hot-switch in one process. |

### Two modes

| | **Exo-mode** | **Macro-mode** |
|---|-------------|----------------|
| **Analogy** | Thin core + swappable implementations | Selected backends welded into the image at compile time |
| **Names in docs/code** | Low coupling, PLUGIN_V1, `PluginHost` | Monolith, high coupling, `monolith.toml` |
| **Typical entry** | `src/main.rs` | `src/main_monolith.rs` + `feature monolith` |
| **Slots** | `settings.json` backends | `weld_modules`; empty list + empty `exclude` → **weld all seven** |
| **Desktop host (main repo)** | **Yes** (default `oclivenewnew-tauri`) | Factory scaffold complete; full hot-path weld matching `process_message` still evolving (RFC §9) |

### Relation to three layers

[KERNEL_FACTORY_VISION.md](KERNEL_FACTORY_VISION.md) **recipe · implementation · code** layers apply to **both** build modes. Macro-mode changes **how slots resolve** on welded paths, not the orchestration contract.

---

## Characteristics

### Runtime and contracts

- Contract-first thin kernel; business rules stay in engines, not API glue.
- Seven replaceable slots + session overrides; unified `PluginHost::resolve_for_role`.
- Cross-host errors: `KernelErrorBody` and code conventions.
- Complex emotion `narrative_hint` across turns.
- Directory plugins and MCP: high-risk capabilities require user grants.

### Delivery and ecosystem

- Role packs as the integration surface; editor/launcher/runtime exchange on disk.
- Distribution discipline: OOCP, pack validate/sign, breaking-change process.
- Kernel factory: `init` / `build` / `bench` / templates / `--kernel-source`.
- Dual-mode artifacts: standard + optional Monolith binary; bench compares latency and size.
- Multiple hosts: desktop, HTTP `--api`, scaffold `kernel_server`.

### Extension

- Remote sidecars (JSON-RPC, BYOK paths).
- Directory plugins under `plugins/<id>/`.
- Agent slot + MCP + function-calling parsing.
- Open lab: second implementations over single-vendor lock-in.

### Engineering

- Local-first defaults; SQLite + configurable `app_data`.
- Three-layer testing: protocol (this repo), components (pack-editor), plugins (editor patterns).
- Startup health before first turn.
- Documented audit posture (no zero-vulnerability claim).

---

## Related docs

| Topic | Doc |
|-------|-----|
| Kernel factory | [KERNEL_FACTORY_VISION.md](../../creator-docs/getting-started/KERNEL_FACTORY_VISION.md) |
| Kernel-centric diagram | [KERNEL_AND_MODULES_ARCHITECTURE.md](../../creator-docs/getting-started/KERNEL_AND_MODULES_ARCHITECTURE.md) |
| Pure kernel boundary | [PURE_KERNEL_BOUNDARY.md](../../creator-docs/getting-started/PURE_KERNEL_BOUNDARY.md) |
| Monolith RFC | [RFC_OCLIVE_MONOLITH_MODE.md](../../creator-docs/rfc/RFC_OCLIVE_MONOLITH_MODE.md) |
| Plugin contract | [PLUGIN_V1.md](../../creator-docs/plugin-and-architecture/PLUGIN_V1.md) |
| CLI | [OCLIVE_CLI_GUIDE.md](../../creator-docs/cli/OCLIVE_CLI_GUIDE.md) |
