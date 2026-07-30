# Distro capability profile (HostProfile)

[中文](../../creator-docs/kernel/DISTRO_CAPABILITY_PROFILE.md)

**Status**: P1 contract (schema + examples) **Done**; P4 profile scheduling (`HostProfile` load & merge) **Done** (`host_profile.rs` / `OCLIVE_DISTRO_PROFILE` on spawn).  
**Audience**: Desktop, VS Code, launcher, hardware distro integrators.  
**SSOT module shape**: Aligned with role-pack `settings.json` → `plugin_backends`; see [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md) and `kernel/crates/oclive_validation/src/plugin_backends.rs`.

---

## 1. Positioning

| Layer | File | Role |
|-------|------|------|
| **Distro** | `distro.oclive.toml` at distro root (next to bundled `bin/`) | **HostProfile** at spawn: prompt/memory/post_process, `host_flags`, optional **`[plugin_backends]` full-table replace** |
| **Role pack** | `pipeline.ocblueprint` → `slot_registry` (v2); legacy `settings.json` | Six-slot defaults; may be **fully replaced** by distro profile when `[plugin_backends]` is declared |
| **Session** | Host DB / session override | Temporary field overrides on effective backends |

**Not in**: blueprint `runtime_config` (v3 frozen). **Not** Monolith `monolith.toml` (compile-time only). Post-process chain RFC: [RFC_OCLIVE_POST_PROCESS_CHAIN.md](../../creator-docs/rfc/RFC_OCLIVE_POST_PROCESS_CHAIN.md) (ZH).

**Kernel binary**: Config describes expected module matrix at spawn — **not** binary trimming. Process selection: [DISTRO_KERNEL_LIFECYCLE.md](DISTRO_KERNEL_LIFECYCLE.md).

---

## 2. File location

| Product | `distro_id` | Path |
|---------|-------------|------|
| **Chat Pro** | `desktop` | Example: `examples/distro-profiles/desktop.oclive.toml`; **installer**: `distros/desktop-tauri/resources/distro-profiles/desktop.oclive.toml` |
| **VS Code Flash** | `vscode` | Extension root `distro.oclive.toml`; mirror: `examples/distro-profiles/vscode.oclive.toml` |
| **dev lab** | `desktop-chat` | `examples/distro-profiles/desktop-chat.oclive.toml` |
| **AI Theater** | `theater` | `examples/distro-profiles/theater.oclive.toml` |
| Custom | any | `OCLIVE_DISTRO_PROFILE` env |

**Distro id**: `OCLIVE_DISTRO_ID` env; optional HTTP header `X-OCLive-Distro-Id`.

---

## 3. Schema (`schema_version = 1`)

```toml
schema_version = 1
distro_id = "vscode"
display_name = "OCLive VS Code"

[plugin_backends]
memory = "builtin"
emotion = "builtin"
event = "builtin"
prompt = "builtin"
llm = "ollama"
agent = "builtin"

[slots]
complex_emotion = "off"

[host_flags]
skip_agent = true
skip_complex_emotion = true
event_impact_llm = false

[llm_runtime]
mode = "performance"
endpoint = "http://127.0.0.1:8421"
auto_start = true
startup_timeout_ms = 90000
retry_cooldown_ms = 30000
model_alias = "oclive-performance"

[resource_coordination]
gpu_safety_reserve_mib = 768
pending_lease_ttl_ms = 120000
active_lease_ttl_ms = 1800000
allow_unverified_admission = true

[turn_thinking]
default = "auto"
fast_skip_complex_emotion = true
auto_deep_min_chars = 80
fast_knowledge_limit = 4
fast_memory_cap = 4

[prompt]
profile = "concise"

[memory]
retrieval = "light"

[post_process]
chain = "minimal"

[user_identity]
default_id = "classmate"
allowed_ids = ["classmate"]

[interaction]
default_mode = "pure_chat"
allow_mode_switch = true
```

### 3.1 `plugin_backends` enums

Same as role pack (`snake_case`): memory, emotion, event, prompt, llm, agent — values `builtin` / `remote` / `local` / `directory` / `none` (see [MODULE_NONE_SEMANTICS.md](MODULE_NONE_SEMANTICS.md)). `directory_plugins.*` when backend is `directory`.

### 3.2 `host_flags` and `slots`

- **`skip_agent`**: Force `agent = none` at runtime.
- **`skip_complex_emotion`**: Skip co-present complex emotion.
- **`event_impact_llm = false`**: Skip event LLM `estimate_event_impact` globally; rules path still runs. `OCLIVE_EVENT_IMPACT_LLM=0` equivalent.
- **`slots.complex_emotion = off`**: Same as `skip_complex_emotion` (either off → closed).

### 3.2.1 `[turn_thinking]` (orchestration · not a six-slot)

Fast / Deep per turn via `TurnThinkingRouter` in `co_present`. Fields: `default` (`fast`|`deep`|`auto`), `fast_skip_complex_emotion`, `auto_deep_min_chars`, `fast_knowledge_limit`, `fast_memory_cap`, `deep_capsule`, `prompt_prefix_cache`, `fast_persistence` (`legacy`|`strong_only`). RFC: [RFC_TURN_THINKING_PERSISTENCE_SUMMARY.md](../rfc/RFC_TURN_THINKING_PERSISTENCE_SUMMARY.md).

**Chat Pro default (`desktop`)**: `default = auto`, `fast_persistence = strong_only`; streaming via `POST /chat/stream`; Deep capsule when pack enables it.

### 3.2.2 `[resource_coordination]` (host control plane, not a blueprint field)

The distro policy defaults are:

| Field | Desktop default | Meaning |
|-------|-----------------|---------|
| `gpu_safety_reserve_mib` | `768` | VRAM that must remain after a newly admitted cold load |
| `pending_lease_ttl_ms` | `120000` | fallback expiry for an unactivated/cancelled reservation |
| `active_lease_ttl_ms` | `1800000` | diagnostics TTL for observe-only activity; managed resident runtimes release explicitly |
| `allow_unverified_admission` | `true` | permit a conservative built-in attempt when `nvidia-smi` is unavailable and report degraded state |

Environment overrides: `OCLIVE_GPU_SAFETY_RESERVE_MIB`, `OCLIVE_RESOURCE_ALLOW_UNVERIFIED`; adapter estimates may be overridden with `OCLIVE_LLAMA_GPU_RESERVATION_MIB` and `OCLIVE_COSYVOICE_GPU_RESERVATION_MIB`. `OCLIVE_GPU_DEVICE_INDEX` (then `CUDA_VISIBLE_DEVICES`) selects the target device. These adapter controls are not role-pack fields.

The implementation covers NVIDIA snapshots, concurrent pending reservations, managed llama-server cold start/release, observe-only Ollama foreground activity, and official bundled CosyVoice2 admission. Performance LLM resource suspension closes the Ollama fallback gate, drains in-flight fallbacks, and unloads models tracked by this runtime; ordinary failure fallback is unchanged, while explicit warm-up/runtime selection performs recovery. The host Resource Adapter Registry also exposes control mode, adapter-local profiles, residency support, truthful lifecycle operations, and current leases through resource diagnostics v2; registration describes capability but does not schedule it. Pure Plan Compiler / CLI diagnostics remain `not_evaluated`, while desktop diagnostics refresh runtime state. Adapters are not yet coordinator-selectable; Voice does not yet initiate suspension, failure rollback, or post-release recovery. Third-party registration, RAM/CPU, render adapters, and real shared-VRAM soak remain future work. See the [blueprint extension and resource coordination RFC](../rfc/RFC_BLUEPRINT_EXTENSION_AND_RESOURCE_COORDINATION.md).

### 3.3 Prompt / memory / post_process mapping

| Field | `full` (desktop) | `concise` (vscode example) |
|-------|------------------|----------------------------|
| `prompt.profile` | Full pack + engine anchors | Concise overlay |
| `memory.retrieval` | 8 memories | `light`: 4 |
| `post_process.chain` | `standard` | `minimal` (forces builtin `profile=minimal`) |
| `visual_presentation.mode` | Pack default | `off` / `image_only` / `stage_full` |
| `[theater].director_plugin` | unset | official theater director plugin id |

**Merge (reply post-process)**: `chain=minimal` → effective `builtin.profile=minimal`. **User identity**: DB override → profile `default_id` → catalog default.

---

## 4. Merge rules (P4 implementation)

In `effective_plugin_backends_for_session` (`host_backends.rs`):

1. **Role base**: `slot_registry` or legacy `plugin_backends`; `directory_plugins` from pack.
2. **User LLM / env** overrides.
3. **Distro profile**: If `[plugin_backends]` declared → **`profile_override` replaces entire six-slot table** (`directory_plugins` **not** overwritten).
4. **`host_flags`**: `skip_agent` → `agent = none`; skip complex emotion flags.
5. **Session**: `PluginBackendsOverride` on top.

**Not** an intersection ceiling model — stable distros (vscode/theater) **lock** matrix via explicit `[plugin_backends]`; experimental desktop-chat **omits** it. See [DISTRO_DEFAULT_PLUGINS.md](DISTRO_DEFAULT_PLUGINS.md) §2.

**Single process multi-host (v1)**: `HostProfile` is process-wide; profile conflict → **replace restart**, not hot switch.

---

## 5. Bundled kernel vs shared fallback

| Scenario | Behavior |
|----------|----------|
| Cold start | Spawn **distro bundled** `oclive-kernel-server` first |
| Bundled fails | Spawn **shared** fallback with same `OCLIVE_APP_DATA` + profile + roles; plugins under `{app_data}/distros/chat-pro/plugins/` reused |
| `promote` | Developer maintenance — not default end-user path |

See [DISTRO_KERNEL_LIFECYCLE.md](DISTRO_KERNEL_LIFECYCLE.md).

---

## 6. Examples

| Distro | Path |
|--------|------|
| Desktop | [`examples/distro-profiles/desktop.oclive.toml`](../../examples/distro-profiles/desktop.oclive.toml) |
| VS Code | [`examples/distro-profiles/vscode.oclive.toml`](../../examples/distro-profiles/vscode.oclive.toml) |

---

## Related

- [DISTRO_KERNEL_LIFECYCLE.md](DISTRO_KERNEL_LIFECYCLE.md)
- [DISTRO_DEFAULT_PLUGINS.md](DISTRO_DEFAULT_PLUGINS.md)
- [KERNEL_SCHEDULER_RESCOPE.md](../../handoff/KERNEL_SCHEDULER_RESCOPE.md)
- [Blueprint extension and resource coordination RFC](../rfc/RFC_BLUEPRINT_EXTENSION_AND_RESOURCE_COORDINATION.md)
- [CROSS_HOST_MEMORY.md](../role-pack/CROSS_HOST_MEMORY.md)
- [OCLIVE_APP_DATA.md](OCLIVE_APP_DATA.md)
