# Distro default plugin matrix

[中文](../../creator-docs/kernel/DISTRO_DEFAULT_PLUGINS.md)

**Status**: P1 contract (design + example profiles aligned)  
**Audience**: Distro integrators, agents  
**Prerequisite**: [DISTRO_CAPABILITY_PROFILE.md](DISTRO_CAPABILITY_PROFILE.md) · [ROLE_PACK_BOUNDARY.md](../../handoff/ROLE_PACK_BOUNDARY.md)

---

## 1. Customize plugins, not kernel binaries

| Layer | Per-distro custom? | Notes |
|-------|-------------------|-------|
| **Kernel binary** | **No (trimming Deferred)** | Each distro ships **bundled full kernel**; **shared fallback** on bundled failure |
| **`distro.oclive.toml`** | **Yes** | HostProfile: prompt/memory/post_process, `host_flags`, optional **`[plugin_backends]`** |
| **Default role packs** | **Yes** | Bundled `distros/chat-pro/roles/*` tuned per scenario |
| **Directory plugins / side channels** | **As needed** | VS Code penetration, `reply_post_process`, `theater_director`, etc. |

**One sentence**: **Single process** `:8420`; distro differences = **profile** + **default blueprints** + **directory plugins** — **not** per-distro trimmed binaries (Deferred).

---

## 2. Merge semantics (must read)

Actual code behavior ([`host_backends.rs`](../../kernel/crates/oclive_kernel_host/src/state/host_backends.rs)):

1. Resolve six slots from role **`slot_registry`** (or legacy `plugin_backends`);
2. Apply user LLM / env overrides;
3. If distro declares **`[plugin_backends]`** → **`profile_override` replaces entire six-slot table** (`directory_plugins` still from role pack);
4. `host_flags.skip_agent = true` → force `agent = none`.

**Design meaning**:

- **Chat Pro** (`desktop`): **omit** `[plugin_backends]` → open ceiling from role blueprint.
- **VS Code Flash** (`vscode`): **explicit** `[plugin_backends]` → locked matrix regardless of pack.
- **dev lab** (`desktop-chat`): omit + lighter prompt/memory — not Release hero.
- **Theater** (`theater`): explicit light matrix + `theater_director` side channel.

---

## 3. Three main products + dev lab

### 3.1 `desktop` — Chat Pro

| Dimension | Strategy |
|-----------|----------|
| `[plugin_backends]` | **Omitted** (open ceiling) |
| `host_flags` | agent / complex_emotion **on** |
| `prompt.profile` | `full` |
| `memory.retrieval` | `default` (8) |
| Bundled profile | `distros/desktop-tauri/resources/distro-profiles/desktop.oclive.toml` |

### 3.2 `vscode` — VS Code Flash

| Dimension | Strategy |
|-----------|----------|
| `[plugin_backends]` | **Explicit** all `builtin` + `llm = ollama` |
| `host_flags` | `skip_agent` · `skip_complex_emotion` |
| `prompt.profile` | `concise` |
| `memory.retrieval` | `light` (4) |
| `post_process.chain` | `minimal` |
| Penetration | Non-six-slot independent vsix |

| Dimension | Chat Pro | VS Code Flash |
|-----------|----------|---------------|
| Kernel binary | Full bundled | **Same build** |
| `[plugin_backends]` | Omitted | Full builtin replace |
| Agent / CE | On | Off |

### 3.3 `desktop-chat` — dev lab only

Omit `[plugin_backends]`; `concise` + `light` memory; examples/ only.

### 3.4 `theater` — AI Theater (delivered 2026-06)

| Slot | Value | Reason |
|------|-------|--------|
| memory | `none` | Transcript on frontend |
| emotion | `builtin` | Tone per line |
| event | `none` | Short single-scene play |
| prompt | `builtin` | Standard PromptBuilder |
| llm | `ollama` | Local default |
| agent | `none` | No toolchain |

`[theater].director_plugin` = `com.oclive.theater_director_official` (side channel, not six-slot).

---

## 4. Where to configure what

| Goal | Write in | Who |
|------|----------|-----|
| Lock six-slot matrix | `distro.oclive.toml` → `[plugin_backends]` | Distro author |
| Disable Agent / CE | `[host_flags]` / `[slots]` | Distro author |
| Prompt / memory / post_process | `[prompt]` / `[memory]` / `[post_process]` | Distro author |
| Per-role LLM model | Blueprint `slot_registry.llm.model` | Blueprint author |
| Reply post-process | `config.json` → `reply_post_processor` | Role pack (side channel) |

Do **not** put distro policy in blueprint `runtime_config` distro fields — see [ROLE_PACK_BOUNDARY.md](../../handoff/ROLE_PACK_BOUNDARY.md).

---

## Related

- [DISTRO_CAPABILITY_PROFILE.md](DISTRO_CAPABILITY_PROFILE.md)
- [MODULE_NONE_SEMANTICS.md](MODULE_NONE_SEMANTICS.md)
- [VSCODE_DISTRIBUTION.md](../../handoff/vscode/VSCODE_DISTRIBUTION.md)
- [KERNEL_SCHEDULER_RESCOPE.md](../../handoff/KERNEL_SCHEDULER_RESCOPE.md)
