# Changelog (English)

> **Chinese mirror**: [CHANGELOG.md](CHANGELOG.md) — keep user-facing entries in sync between both files.

## [Unreleased]

### Added

- **Scaffold Package v1, bounded generation, and a focused CLI surface**: added the independent `oclive_scaffold` contract crate, strict JSON Schemas, project/user/official local discovery, configurable priority, version rejection, source plus SHA-256 locking, path-escape checks, and atomic `.oclive/scaffold.lock.json`; four compiled official fallback packages only describe existing generators. Stage 2B adds `oclive scaffold generate`, which only materializes digest-pinned declarative text/copy files into a new directory and requires an exact lock, per-invocation untrusted acknowledgement, declared `project.write`, transactional placement, and value-free provenance; dry-run performs no writes and official built-ins only return domain-command delegation. It still has no network, third-party command/script/hook execution, or composition runtime, and it rejects `ci.*` capabilities plus third-party `com.oclive.*` namespaces. Default help exposes 15 stable entries while ten experimental commands and legacy project-archive `template` remain callable but hidden; domain-aware CI registers a separate `oclive.scaffold` module whose impact only propagates to `oclive.cli`.
- **Domain-aware CI Stage 1 shadow planning**: added strict versioned `oclive.module.json` descriptors, a central impact map, and a trusted validation catalog. A pure Rust planner combines path-based direct ownership with semantic impact closure and emits stable, explainable `plan.json`; unknown paths, broken descriptors, unsupported required extensions, and central high-risk rules fail safe to the active policy's full set. `oclive ci plan/explain` accepts Git diffs or explicit paths and renders Markdown. GitHub's `ci-impact-plan` only uploads a Job Summary/artifact, remains `continue-on-error`, and cannot skip existing jobs. The scaffold is intentionally unchanged until it can generate and preflight the same stable metadata.
- **Unified Resource Coordinator, candidate plans, and adapter registry**: the desktop Host owns one coordinator for NVIDIA VRAM, system RAM, CPU topology, priorities, atomic reservations, and lease diagnostics, while HostProfile expresses safety reserves, queue/aging policy, automatic-preemption policy, and finite scheduling intent. Managed `llama-server`, observed external Ollama activity, the performance activity observer, and bundled CosyVoice2 register through the Resource Adapter Registry. Resource diagnostics v5 distinguish control ownership, registration provenance, adapter-local profiles, residency support, truthful lifecycle actions, and the current `profile_id`, then compile a versioned read-only candidate plan with capacity decisions, proposed transitions, and rollback information from live state. Performance llama now exposes three real runtime tiers—`gpu_full`, `gpu_balanced`, and `cpu_compatibility`—which change `llama-server --n-gpu-layers` and degrade through admission failures. The admission queue provides priority ordering, fair aging, timeout, and cancellation cleanup. Automatic preemption applies only to lower-priority managed adapters that declare a reversible operation and have exact transition grants, and restores victims in reverse order. The Host also exposes an owner-namespaced in-process registration port for third-party Resource Adapters/Controllers plus `render`, `compute`, and `hybrid` resource descriptions; directory-manifest auto-registration and an actual bundled Live2D runtime are not implemented. Blueprints and role packs still describe capabilities and desired configuration without controlling physical resources directly.
- **Stable v4 blueprint extension envelope**: `pipeline.ocblueprint` now has a strict minimal `extensions` declaration map with safe external JSON payload references. v4 is the Stable successor to v2 and intentionally excludes frozen v3 dual-core fields. The host, CLI, doctor, plugin slot-registry writes, and pack editor dispatch exactly across v2/v3/v4; unknown versions no longer fall back to v2. Optional extensions round-trip but remain inactive, while required extensions block role activation until the Capability Registry lands. The editor creates v4 by default, the CLI adds `pack create --format-blueprint-v4`, and existing v2 packs remain compatible without automatic rewriting.
- **Local llama.cpp LoRA GGUF workflow**: Local Model Settings shared by Chat Pro and AI Theater can now import raw LoRA GGUF or `.ocadapter` v1 packages. The kernel validates ZIP paths and size, SHA-256, GGUF adapter metadata, and base-model architecture; managed copies use staging plus atomic replacement and activate through managed `llama-server --lora`. Independent full bases may declare provenance and adult rating through a matching `.ocmodel.json`; bases and LoRAs are not permanently bound, so users may choose any compatibility-verified pairing. Adult bases and adapters require acknowledgement, and changing the base automatically deactivates the previous LoRA so a same-architecture adapter or abliteration LoRA cannot leak into the new combination. Failed activation restores the previous database/environment/process selection, and Hugging Face/PEFT remains an independent future converter plugin.
- **DeepSeek-inspired example role pack**: added the community-created, unofficial `deepseek` Portable Core pack with a core persona, an empty read-only memory seed, three user identities, a cognition boundary, a default scene, and seven transparent portraits; the pack does not imply official DeepSeek authorization or endorsement.
- **Independent cross-distro Persona / Memory transfer**: added versioned `.ocpersona` and `.ocmemory` JSON contracts, shared validation, and desktop-host import/export APIs. Role packs may provide read-only creator-authored `memory_seed.json`; persona import restores only the mutable profile without overwriting core persona, while memory import merges LTM and excludes chats, short-term cache, and ephemeral situation state. Bundled role packs, the Robot Soul example, and CLI create/init scaffolds now generate this seed container consistently.
- **Chat Pro adult role extension v1**: optional root-level `adult_extension.json` is isolated from the Portable Core base. Chat Pro adds local adult acknowledgement, global and per-role gates, import prompting, a dedicated management page, structured dialogue plus silent-narration bubbles, natural exit, and automatic beat pacing. Adult memories use a separate `content_scope`, while ordinary chat keeps only a non-explicit relationship bridge. Voice reads dialogue only; pacing waits for both the display interval and current playback, and voice failures downgrade only the active interaction to text. Continuous background pre-generation now uses cancellable, recoverable staged beats: the background stores structured text only, then commits, renders, and voices one ordered beat after returning to that chat; all chats share a user-configurable capacity, and new user input preempts and discards unshown beats. The pack editor adds an independent adult-extension page gated by complete base-pack validation and round-trips it in the combined pack.
- **Verifiable real-time soak**: `oclive bench --soak` now distinguishes the 8–120 second accelerated smoke clock from explicit `--soak-real-time` wall-clock runs, with fractional hours and configurable sampling. It establishes a steady-state RSS baseline after one warmup and runs chat load independently from the sampling clock. Schema v2 samples the Release kernel PID directly and records RSS/CPU, request failures, early exit, worker join, and child reaping; failed criteria return non-zero so cold allocation, a Cargo wrapper, or nominal 72h smoke cannot be mistaken for leak evidence. CLI loopback HTTP bypasses system proxies, and Windows cross-drive `--kernel-source` scaffolds preserve valid absolute Cargo paths.

- **Local HTTP API authentication**: the desktop host now generates and injects a random `OCLIVE_API_TOKEN` when spawning the kernel; headless `--api` now also requires that variable by default, and every route except public readiness `GET /health` requires `x-oclive-api-token`. Only isolated local development may explicitly opt out with `OCLIVE_API_ALLOW_UNAUTHENTICATED=1`; CORS is limited to local-development/Tauri origins, and OOCP plus restart smoke tests attach the token automatically.

### Fixed

- **CI supply-chain audit restored to zero vulnerabilities**: upgraded PostCSS / Nanoid, Undici, and WebDriverIO 9.30.1 to clear newly disclosed zero-length custom-ID loops, source-map arbitrary-file reads, and HTTP response/cache/header-processing flaws. WebDriverIO's browser-management dependency is selectively pinned to `@puppeteer/browsers` 3.2.0, removing the unpatched `extract-zip` symlink path-traversal chain while retaining the existing `remote` API. Both production and full npm audits now report zero vulnerabilities.
- **Follow-up fixes for the emotion engine and large-file splits**: restored the shared button and help-component imports lost while splitting `ModelManagerBody`, with a real-child mount regression test so stubs cannot hide template-resolution failures. The complex-emotion backend now follows the role's effective slot: an omitted slot or `none` no longer reads, injects, or writes a narrative hint, while valid main-LLM emotion labels still update the bot's six-slot state. Remote and directory fallback labels also drive current emotion and events. Marker and plugin hints share a 200-Unicode-character hard cap with an additional persistence-layer guard, and trailing unclosed `[EMO]` content is fully stripped. English lexicon matching now handles punctuation, newlines, tabs, and `no` / `not` / `never` / `n't` negation without matching embedded words such as `unhappy` or `glove`.
- **CosyVoice2 streaming first-chunk state and segmented diagnostics**: restore the upstream model's initial `token_hop_len` before every synthesis so a prior stream cannot leave the next request at a 50/100-token threshold that reduces chunk count and inflates TTFC. The sidecar, shared playback layer, and stress harness now carry versioned stage timings, prompt-cache state, and client-delivery overhead; the candidate 72-hour hardware-soak flow atomically refreshes a bounded checkpoint and retains the last progress after interruption. The existing 8-second voice gate and VRAM safety margin remain unchanged.
- **8 GB voice coexistence profile now preserves cold-load headroom**: lowered the `gpu_balanced` and shared-GPU stress-tool default cap from 24 to 22 layers. On the RTX 5060 Laptop 8 GB reference system, ordinary system variance left the 24-layer run at **2559 MiB < 2560 MiB**, correctly denying the CosyVoice cold load; 22 layers completed the mixed-FP16 warm-up and retained **1370 MiB** peak headroom during a five-minute LLM/TTS coexistence run. `gpu_full` and explicit environment overrides remain available.
- **Real bounded Loom models restored in Nightly**: replaced the dependency-wide global `cfg(loom)` with the desktop host's package-scoped `loom-tests` feature. `oclive test --loom`, the validation catalog, and Nightly now run both real interleaving models instead of treating the disabled-fixture smoke as concurrency evidence. `oclive ci init` no longer emits an invalid Loom job tied to the main repository's desktop path for independently generated projects.
- **npm development-tool supply-chain closure**: upgraded to ESLint 10 / Antfu 9.2 and WebDriverIO 9.30, clearing the `brace-expansion` and `fast-xml-parser` findings plus the Unicorn peer-contract conflict; removed `vue3-sfc-loader` and its bundled Vue 2/PostCSS chain in favor of the official Vue SFC compiler loaded only in explicitly unsafe DEV mode, with directory-plugin script imports restricted to `vue`. Both full and production npm audits are hard gates, and every checked-in official/example directory-plugin SFC is covered by a compile regression test.
- **CI execution-lane split**: moved `loom`, `fuzz`, `cli-bench`, `visual-presentation-smoke`, and native `e2e-tauri` out of the PR/push workflow into `nightly-advisory.yml`, with a daily schedule and targeted manual dispatch. Failures now turn the advisory workflow red without gating main, while CLI benchmark reports and fuzz/E2E failures retain artifacts; main CI keeps required gates plus the non-blocking Stage 1 shadow planner.
- **Shadow-planning scenario contract**: added a strict, versioned 11-scenario simulation corpus and `ci:shadow-samples` evidence collector covering docs, shared frontend, role packs, directory plugins, kernel contracts, scaffolds, examples, the Nightly tier, CI control-plane changes, lockfiles, and unknown paths. Repository tests lock module closure, validators, workflow coordinates, and fail-safe reasons; generated evidence explicitly states that it is simulation only, executes no validator, and cannot prove a zero false-negative rate.
- **Official CI scaffold and audit-diagnostic alignment**: workflows generated by `oclive ci init` now use Node 22, make Cargo audit failures blocking by default, and quote the Loom step name that previously produced invalid YAML. `oclive lint --audit-ci` now parses YAML job/step ownership, accepts either a direct `cargo audit` step or the Dimension 5 owner, and scopes `continue-on-error` to that owner instead of being confused by unrelated advisory jobs.
- **Adult staged-beat structured output and real-device capacity calibration**: fixed the universal prompt's trailing “dialogue only” boundary overriding Chat Pro's adult JSON contract. Adult turns now end with a dedicated final output contract; a real Qwen2.5 7B GGUF completed 15/15 structured beats with zero fallback across queue depths 1/2/4/8. Added the reproducible `scripts/measure-adult-stage.mjs` benchmark and retained default 2 / recommended 2–4 after RTX 5060 Laptop 8 GB plus CosyVoice2 shared-VRAM testing.
- **Engineering paths and startup diagnostics**: fixed `oclive-cli init --kernel-source` still generating the retired `src-tauri` and root `crates` paths, with assertions against the real repository layout. Desktop, headless, and generated hosts now share one `--port` parser; missing, zero, or invalid values produce a stable diagnostic and exit code 2. App-data initialization, shared-kernel backup/rollback, and cloud-token file-backup failures are no longer silent, and shared-kernel promotion no longer runs twice.
- **Directory-plugin UI and voice-sidecar startup**: fixed voice/sidebar slots rendering `unknown uri` when Windows/WebView2 returns `ocliveplugin://localhost/...`. Full shells and every UI slot now receive a platform-correct protocol URL: Linux/macOS/iOS use `ocliveplugin://localhost/...`, while Windows/Android use the host-mapped `https://ocliveplugin.localhost/...`, preventing non-Windows releases from mounting an empty iframe. Constrained iframes receive the bridge before plugin scripts run and register through one parent-side broker, fixing voice recognition's `OCLive bridge unavailable`. Microphone capture is now proxied by the trusted parent exclusively for the official Voice toolbar, avoiding opaque-origin iframe `Invalid security origin` failures without enabling `allow-same-origin`; capture commands use a serialized state machine, so cancellation during permission grant cannot leak a recording stream. Voice v0.5 transcription submissions carry idempotency ids, coalesce short-window duplicate events from legacy plugins, and restore host-internal warmup/streaming speech events independently of plugin subscription allowlists. Streaming TTS now subtracts only audio that reached playback completion, avoids three-character synthesis fragments, and does not block the first phrase on an extra directive RPC; changing messages or roles aborts stale synthesis and scheduled PCM. The production desktop profile enables the benchmarked Deep prompt-prefix cache. Mumu fallback cards repair broker registration on the first iframe load, retain displayed content during state refresh, and update in place on role/identity events; plugin bootstrap, identity, polling, and card refreshes reject stale role/generation results. Directory-plugin children now receive persisted `config.json` at startup, and streaming TTS only fetches a confirmed sidecar endpoint before falling back directly to RPC. Protocol/config failures carry stable identifiers in `oclive_plugin` logs.
- **Reply echo and model-error presentation**: the standard reply post-processor safely removes an exact leading copy of the current user message while preserving natural continuations such as `hello` → `hello there`; structured `LLM_ERROR` payloads from SSE and IPC now map to actionable localized guidance instead of leaking raw JSON into chat UI.
- **Pure-blueprint role-pack import loop**: Chat Pro preview and installation for `.ocpak`, `.zip`, and extracted directories now recognize `pipeline.ocblueprint` directly and prefer blueprint `meta` over a same-level legacy `manifest.json`. Runtime exports now use a `{role_id}/...` top-level directory, fixing in-app imports of v2 packs produced by the editor.
- **Loopback endpoint proxy compatibility**: IPv4/IPv6 loopback endpoints for remote plugins, remote agents, and OpenAI-compatible LLMs no longer inherit user HTTP proxies, preventing `localhost` calls from being redirected into 502s/timeouts; the test mock now validates and echoes the JSON-RPC request id.
- **Active-doc truth convergence**: product execution and release links now point to current SSOTs; `check-stale-paths` adds a G3/G12 guard against restoring `handoff/archive/*` product checklists as current truth.

### Removed

- **Completed maintenance scripts**: removed one-shot layout migration, file-splitting, and bulk-fix scripts, together with experimental local-TTS installers that never became a supported workflow. Current paths remain guarded by `check-stale-paths`, Markdown-link checks, and the supported CLI; Git retains the historical implementations.
- **Bundled role-pack adjustment**: removed the `shimeng` role pack; tests and examples that require an on-disk bundled role now use a remaining shipped pack or an isolated fixture.

---

## [0.5.0] - 2026-07-10

**Desktop host `0.5.0`** · **Pack editor `0.5.0`** · voice side-channel v0.4 · Turn Thinking E/F · Fluent default shell · Apache-2.0

### Breaking

- **License**: host relicensed from AGPL-3.0 + plugin exception to **Apache-2.0** (root `LICENSE` + `NOTICE`).
- **Turn Thinking default**: `desktop.oclive.toml` enables `fast_persistence = "strong_only"` (Fast casual chat skips favor/long-term memory; strong relationship events still consolidate).
- **Affect display**: legacy scalar favor fields on `RoleInfo` / `SendMessageResponse` are deprecated; UIs should read `display_metrics`.
- **Voice**: Piper removed from the product path (dev loop `--tts-sherpa` retained).

### Added

- **Roadmap merge wave (2026-07-10)**: CI dimension5 Python 3.11 for voice TTS ratchet; loom via `RUSTFLAGS --cfg loom`; `preferred_tts_profile` role-switch sync; `prompt_extra_sections` production wiring; doc Wave-3 (EN mirrors Done, dual-core stubs, ai-package EN, USER_MANUAL §3.6 voice); e2e-tauri readiness; Vitest smoke (Fluent default shell, stream toggle).

- **[docs] English mirror wave 2**: `README.en.md` aligned with Chinese homepage (four examples · three distros · ecosystem · roadmap); `creator-docs-en` adds CREATOR_GOLDEN_PATH, eight role-pack deep guides, dual-core, PLUGIN_MARKET, RELEASE_VERSIONING, RFC summaries; `human-docs-en` adds full modules/paths mirrors; path normalization (`NAMING_CONVENTIONS`, `development/LIGHTWEIGHT_PROFILE`, merged `APPLICATION_SCENARIOS`); new **`scripts/check-doc-mirror.mjs`** wired into `npm run check:rust` and dimension5.

- **Voice first-utterance latency**: streaming TTS uses `streamingVoiceChunker` to skip aside/narration/action lines; earlier first chunk; CosyVoice2 `/warm` runs a default **prime** dummy synthesis; role switch triggers sidecar warm + directive prefetch; plugin manifest `rpcTimeoutsMs` for long RPC timeouts; CSP `connect-src` narrowed to default sidecar port `50000`.
- **Voice expansion v0.4 (emotional TTS · optional)**: `com.oclive.voice.asr` bumped to v0.4 · text-only default; when `tts_expansion_enabled`, CosyVoice2 sidecar + model DLC (`voice_model_pack.json`) + `synth_provider` (bundled / local_http / cloud); new RPC `voice.probe_tts` · `voice.warm` · `voice.list_model_packs`; `rules-v1` emits `emo_text` + role-pack `ref_map`; streaming first-sentence `voice:stream-sentence` for earlier TTS; **Piper removed from product path** (dev loop `--tts-sherpa` retained). See plugin [`README.md`](distros/chat-pro/plugins/com.oclive.voice.asr/README.md) · [`TRACK_VOICE`](human-docs/team/TRACK_VOICE_RECOGNITION.md).
- **Unified keybindings system (Phase 1–4)**: Settings → General → Advanced adds “Keybindings” (single UI for in-app + global shortcuts); plugin global shortcuts still reuse `save_hotkey_bindings` to register OS-level listeners; `ShortcutHelp` now renders the current bindings dynamically; voice plugin adds **V hold-to-talk** (`voice.holdToTalk`, window-focused only, won’t steal keys when an input is focused).
- **Chat Pro Windows 98 easter-egg skin**: Konami unlock → `data-skin=win98` (`oclive-runtime-skin`); Settings → General toggle; orthogonal overlay on Fluent + Tool for `data-theme` / `data-shell` / UI scale; synthetic Win98 title bar (`Win98TitleBar` + Tauri `setDecorations`) and 3D dialog chrome; see [`MODULE_MAP_AND_HANDOFF.md`](handoff/MODULE_MAP_AND_HANDOFF.md) §13.2.
- **Side-channel `voice.asr` (Windows delivered · v0.2–0.3)**: official directory plugin [`distros/chat-pro/plugins/com.oclive.voice.asr/`](distros/chat-pro/plugins/com.oclive.voice.asr/) · `provides: voice.asr` · **does not** enter six slots / `process_message`; `chat_toolbar` hold-to-talk + `plugin_rpc_invoke` (`voice.probe` / `voice.transcribe` / `voice.import_model` / `voice.list_profiles` / `voice.speak` / **`voice.build_directive`**) → `com.oclive.voice.asr:submit` → `send_message` or `chat:set_input_draft` (`mode: fill`); **v0.3** adds TTS `tts_profile` · `auto_tts` · `rules-v1` director · optional role-pack `voice_profile.json`; sherpa-onnx engines SSOT in [`examples/voice-loop-minimal/asr/`](examples/voice-loop-minimal/asr/) · [`tts/`](examples/voice-loop-minimal/tts/) spawned by `rpc_server.mjs`; experimental synth adapters: `edge-tts` · `pilot-tts` · `cosyvoice`; official role packs seed toolbar/settings slots in `ui.json`; Win98 overrides in `win98/component-plugin-toolbar.css` / `component-voice-settings.css`; `plugin_bridge` RPC whitelist unit tests; Linux/macOS profiles return `unsupported_platform`; registry in [`RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md`](creator-docs/rfc/RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md) §4.1.
- **Domain layering ports (#101 unblock)**: `LlmClient::supports_prefix_cache` / `generate_with_opts` / `generate_stream_with_opts`; `TurnThinkingStatePort`; removed domain→infra imports from `co_present` / `slot_runner` / `post`; `npm run check:rust` now runs layering + CHANGELOG parity gates first.
- **Affect display channel `display_metrics`**: `RoleData` / `RoleInfo` / `SendMessageResponse` expose UI-only metrics (`favor` / `traits[7]` / `relation_summary`); legacy scalar fields deprecated; frontend `roleStore` prefers the new field.
- **CI flake auto-rerun**: `.github/workflows/ci-rerun-flake.yml` now recognizes `rust (<matrix-os>)` names and invokes one `gh run rerun --failed` only when every failure belongs to the rust matrix; any other failed job preserves the original evidence instead of being masked by a retry.
- **Affect WS4.2–4.4 (simulation/display split)**: `apply_profile_evolution_atomic` commits archive + seven-dim together; deep profile LLM gate (strong event OR every N turns OR radar `radar_deep_pending`, default N=3); `get_display_metrics` GET-only (Tauri + HTTP `/display_metrics`); Tauri `affect:metricsChanged` push + `roleStore` listen.
- **RFC affect drift ratchet**: `scripts/check-rfc-affect-drift.mjs` wired into dimension5.
- **Wave E · Turn Thinking persistence split**: `[turn_thinking] fast_persistence = "strong_only"` (default `legacy`); Fast casual chat skips long_term / favor / profile evolution; **Quarrel / Apology / Confession / Praise** still persist. RFC: [`RFC_TURN_THINKING_PERSISTENCE_SUMMARY.md`](creator-docs-en/rfc/RFC_TURN_THINKING_PERSISTENCE_SUMMARY.md) (full: [Chinese](creator-docs/rfc/RFC_TURN_THINKING_PERSISTENCE.md)).
- **Wave F · Turn Thinking pack routing**: `config.json` → `turn_thinking` (OR/AND · Deep latch · ephemeral situation summary TTL); migration `035_turn_thinking_runtime.sql`; this-turn rule event prepass before router. RFC §8–12.
- **Chat Pro stream cancel**: new sends abort the in-flight SSE via `AbortController` and remove dangling `streaming` bubbles.
- **Chat Pro stream toggle**: Settings → General → Advanced “Streaming replies” (`localStorage` `oclive.chat.streamEnabled`, default on).
- **Monorepo layout (kernel / distros)**: Rust crates under `kernel/`; desktop distros under `distros/{shared,chat-pro,theater,desktop-tauri}`; RFC: [`handoff/distros/ARCHITECTURE_DECOUPLING_RFC.md`](handoff/distros/ARCHITECTURE_DECOUPLING_RFC.md).
- **Theater Track A engineering hygiene (round 16)**: [`handoff/theater/MODE2_UNFREEZE.md`](handoff/theater/MODE2_UNFREEZE.md) Mode 2 unfreeze checklist; `theater-prompt-drift` wired into `dimension5-acceptance.mjs` and `test:theater:smoke`; self-contained `prompts/` in minimal director example; `data/plugins.json` entry for `com.oclive.theater_director_official`.
- **`CODE_OF_CONDUCT.md`** (Contributor Covenant).
- **`human-docs-en/`** minimal set (L0–L3 + 08/09/10 English summaries).
- **`human-docs/08_PR_GATE_MATRIX.md`**, **`03_GLOSSARY.md`**, **`10_SETUP_WINDOWS.md`**.
- **`handoff/GOOD_FIRST_ISSUES.md`** curated issue table.
- **`npm run check:ci-local`**; `package.json` `engines.node >=22`, **`.nvmrc`**.
- Frontend: `distros/shared/src/api/plugin/*`, `useMainShell*`, `useChatStorageSettings`, `chatStoreSend`.

### Changed

- **Node runtime baseline 20 → 22**: the current ESLint/Vue i18n toolchain requires Node 22; `.nvmrc` is now the single version source for GitHub Actions `setup-node`, and a repository contract test prevents workflow, `package.json`, and developer-documentation drift.
- **Voice ASR v0.2.1 (recognition quality)**: chat toolbar WebM/Opus is decoded and resampled to **16 kHz mono WAV** via `audioCapture.ts` before sherpa (fixes mis-decoded PCM garbage); mic constraints use echoCancellation / noiseSuppression / autoGainControl; minimum hold 350ms; engine caches recognizers, rejects too-quiet audio (`audio_too_quiet`), optional **ffmpeg** compressed-audio fallback; **medium** ASR profile stub (switch in settings after importing models).

- **Win98 skin CSS layered refactor**: monolithic `theme-win98.css` split into `distros/shared/src/styles/win98/` (L0 tokens · L1 primitives · L2 shells · L3 panel/component co-located unscoped imports); maximize edge-to-edge (no teal gap), 2px main-window radius, dialog navy caption bars flush to frame; see [`MODULE_MAP_AND_HANDOFF.md`](handoff/MODULE_MAP_AND_HANDOFF.md) §13.2 dependency table.
- **Win98 skin polish**: `modal-backdrop` / `TimeDial.backdrop` dimming; Tool `UiSidePanel` navy caption + Win98 ✕; synthetic title bar uses OCLive app icon (`public/oclive-icon.png`).
- **Fluent “More” panel IA**: action buttons reordered to Settings → Models → Plugins → Market → Shortcuts; tiles grouped as Core / Plugins / Scene / Dev with Debug last; removed empty Shortcuts placeholder from Settings → General (help remains under More and Ctrl long-press). Panel tiles use a responsive grid (`auto-fill minmax`) ordered by footprint left-to-right (Settings / Scene span two columns, others one), tidy in both Daily-chat and Story modes.
- **Chat Pro default shell Fluent**: `resolveOcliveShell()` fallback is now **`fluent`** (quiet living-room shell); set `VITE_OCLIVE_SHELL=tool` for ToolShell; early-boot `index.html` `data-shell` aligned; dark brand teal matches light hue (`--fluent-accent`); role `primaryColor` light tint (focus / user bubble / runtime rail only); FluentShell mounts `InteractionModeBar` as the sole in-shell mode switch; interaction-mode IA in [`MODULE_MAP_AND_HANDOFF.md`](handoff/MODULE_MAP_AND_HANDOFF.md) §13.1.
- **Prompt mechanics fully text-driven (RFC #2 deepened)**: `PromptBuilder` no longer injects any favor/relation numerics, relation stage, event block, boundary-tone guideline, or seven-dim numeric-derived tone into the dialogue prompt (removed `build_event_relation_state` / `build_boundary_tone_guideline` / `build_current_state` and their numeric helpers); persona and tone are driven entirely by the core archive + `mutable_personality` narrative + user-emotion cue, with only a number-free "authenticity constraint" anti-fabrication guard retained; seven-dim/favor are demoted to read-only `display_metrics`.
- **Chat Pro default profile**: `desktop.oclive.toml` enables `fast_persistence = "strong_only"` (Fast casual chat skips favor/long-term memory; strong relationship events still consolidate). Existing session data is not rolled back.
- **Repository layout**: root `crates/`, `src-tauri/`, and `src/` moved to `kernel/crates/`, `distros/desktop-tauri/`, and `distros/{shared,chat-pro,theater}/`; root `npm run tauri:dev` / `tauri:dev:theater` unchanged.
- **Theater doc SSOT sweep**: `theater_director` unified as **shipped (2026-06)** (DISTRO_DEFAULT_PLUGINS · ARCHITECTURE · NAMING · ROADMAP §7 · IA); [`TECHNICAL_DEBT_INVENTORY.md`](handoff/TECHNICAL_DEBT_INVENTORY.md) round 16; acceptance chain → [`PLAYTEST_MATRIX.md`](handoff/theater/PLAYTEST_MATRIX.md).
- **Hybrid chat mirror**: `rebuild_mirror_best_effort` / `delete_mirror_best_effort` (K-ROBUST-01).
- **`canonical_llm_sync` / `plugin_state` / MCP·Ollama downgrade**: `tracing::warn!` (K-ROBUST-02/03).
- **Kernel snapshot & storage capability degraded UI** (`kernel.ts`, `useKernelStatus`, `ChatStorageSettingsPanel`).
- **`process_message` readability closure**: `preflight_turn` / `PostLlmCtx` / `PreLlmOutput` grouping; `events.rs` / `blueprint_v2_slot_registry.rs` module extraction; `SettingsView` tab subcomponents; `role_runtime` submodules; `blueprint_v2` tests moved to `tests/`.
- **handoff layout**: `THEATER_*` → `handoff/theater/`, `VSCODE_*` → `handoff/vscode/`; added `handoff/launcher/`, `handoff/pack-editor/`, `handoff/studio/` distro doc indexes; fixed broken links after theater greenfield reset.
- **README / CONTRIBUTING / SECURITY** community infra updates; PR template links to PR gate matrix; optional `scripts/setup-dev.ps1`.
- **Five-dimension review closure (Batch 1–3)**: architecture overview co-presence chain aligned with Stable code; VS Code / cross-host docs policy-first; `user_identities` validation matches `load_role`; `reply_post_processor` requires non-empty `plugin_id` when `enabled` + `directory`; `ProcessMessageError` stage preserved on outward `AppError` messages; chat turn `role_runtime` preflight merge, single CASE `memory decay` UPDATE, `SessionCache` skips repeat interaction_mode seed; workspace `default-members` excludes `fuzz`; `chatStore` load / `addMessage` micro-optimizations.
- **`chat_storage` pack validation**: `oclive pack validate` now checks `config.json` → `chat_storage` (backend / location / positive integers / replay threshold 0.1–1.0), parity with `reply_post_processor`; `CHANGELOG.md` parity synced.
- **Prompt guardrails elevation & footer dedup**: `KERNEL_DIALOGUE_GUARDRAILS` always includes state continuation, vent-first, and length-by-input (pack `reply_quality_anchor` cannot override); removed standalone `【回复结构】` block; tone block no longer exposes `warmup_level` / normalized impact-factor jargon; official mumu/shimeng/枫侵月 anchors slimmed to persona-only deltas.
- **License change**: host relicensed from AGPL-3.0 + plugin exception to **Apache-2.0** (root `LICENSE` + `NOTICE`); enables closed-source commercial distros and embedded downstreams to combine the kernel freely; `LICENSE_POLICY.md` updated.
- **Official distro · Daily chat / Story mode split**: `distro.oclive.toml` adds `[interaction]`; new `desktop-chat` profile; `desktop` / `vscode` default `pure_chat`; first-run seed order is distro → role pack → `pure_chat`.
- **Pure-chat UI slimming & settings tiers**: Daily chat hides scene/time/plugin sidebar and numeric favor; Settings split into Essentials / More options; Story-mode hint after N turns.
- **User-identity surprise unlock**: identity picker hidden on first screen; after 5 turns or keyword match, show “You could also be…” identity sheet.
- **Interaction mode default & dedup**: first run always Daily chat; user choice persists in `role_runtime`; mode switch unified in `InteractionModeBar` above chat input; removed duplicate locale/appearance/plugin tiles from top-bar More.
- **Pure-chat hides plugins**: Daily chat hides plugin slots, market, Ctrl+Shift+F, and Settings Plugins tab; Story mode restores full plugin UI.
- **Product narrative alignment**: README / AGENTS / positioning docs unified as "AI role assembly platform"; frozen items (dual_core, blueprint v3, expert_routing) phrased as "mechanism pre-wired, off by default"; chat storage clarifies hybrid as the production path.
- **Profile scheduling UX**: desktop status bar, Settings → Kernel & Connection, and the VS Code status bar share unified profile-adaptation wording (attach / mismatch / pin / replace / degraded).
- **Distro profile parse SSOT**: `distro.oclive.toml` parsed once via `oclive_kernel_runtime::distro_oclive_file` (K-PROFILE-01).
- **Host domain re-exports**: runtime engine modules deprecated at `oclive_kernel_host::domain`; `check-host-reexport-imports.mjs` ratchet (D-OPUS-05).
- **`resolve_*` naming adjudication (D-NAME-01)**: 35 non-policy functions renamed to `load_*` / `find_*` / `pick_*` / `build_*` / `merge_*` / `compute_*` / `invoke_*`; 22 cross-host / per-turn policy anchors kept; verb table in `NAMING_CONVENTIONS.md` §4.4.

### Added

- **Theater release packaging**: `npm run tauri:build:theater` · `OCLIVE_TAURI_SHELL=theater` · roles subset (`theater-breakfast-a/b`) via [`scripts/filter-theater-roles.mjs`](scripts/filter-theater-roles.mjs) into `src-tauri/resources/roles/`.
- **Theater 15s engineering proxy**: [`scripts/theater-stranger-proxy.mjs`](scripts/theater-stranger-proxy.mjs) · aggregated in `npm run test:theater:smoke` (CI `frontend` job).
- **Theater vision & roadmap SSOT**: [`handoff/theater/DEVELOPMENT_ROADMAP.md`](handoff/theater/DEVELOPMENT_ROADMAP.md) (Mode 1 greenfield; legacy `THEATER_*` docs removed).
- **Three-distro kernel smoke (Pro / Flash)**: `npm run test:distro:smoke` aggregates profile mirror · distro kernel e2e · Tauri bundled-first; `e2e-distro-kernel` adds a **theater** scenario; CI **`cross-host-e2e`** adds `e2e-tauri-bundled-kernel` and VS Code profile diff. Closure SSOT: [`handoff/THREE_DISTRO_KERNEL_CLOSURE.md`](handoff/THREE_DISTRO_KERNEL_CLOSURE.md).
- **Chat Pro bundled-first spawn (K-SCHED-05/01)**: Tauri `bundle-kernel-for-tauri.mjs` · `pick_best_for_spawn` bundled → shared → dev; `binary_upgrade` replace off by default.
- **VS Code Flash profile mirror**: `examples/distro-profiles/vscode.oclive.toml` ↔ sibling `distro.oclive.toml` · `npm run test:distro-profile-mirror`.
- **Contract extension envelope (V-CONTRACT Phase 0)**: `SlotExtension { schema_id, data }`; optional `extension` on `EmotionResult` / `ComplexEmotionOutput`; `PromptInput.extra_sections` injects generic blocks before the quality anchor; evolution rules in `EXTENSION_POINTS.md`.
- **Hot-path stage tracing (K-PERF-02)**: `oclive_turn` target logs per-`ChatStage` `elapsed_ms`; sample table in `creator-docs/getting-started/PERFORMANCE.md` §6.
- **CHANGELOG CI gate (K-DOC-02)**: `scripts/check-changelog-parity.mjs` wired into `dimension5-acceptance.mjs`.
- **AI Theater v0 (`theater` distro)**: `examples/distro-profiles/theater.oclive.toml`; `TheaterShell` first screen (hides six-slot/blueprint UI); breakfast scene + dual contrast role packs + pre-generated `skeleton.json`; 3 poke chips + local Ollama beat patch (graceful fallback).
- **Product freeze**: no kernel expansion until Theater v0 stranger validation — see [`handoff/theater/DEVELOPMENT_ROADMAP.md`](handoff/theater/DEVELOPMENT_ROADMAP.md) §4.8.
- **Creator golden path**: `creator-docs/getting-started/CREATOR_GOLDEN_PATH.md` (separate from kernel docs).

### Performance

- **Batched memory-decay writes (K-PERF-01/06)**: `DbManager::persist_memory_decay_batch` changed from one independent `UPDATE` per memory to a single batched transaction; after rank, one call per turn merges decay write-back and `accessed_at` touch. See `long_term_memory.rs` and `turn_pipeline/pre.rs`.
- **Lazy shell loading (K-PERF-09)**: `App.vue` now imports `FluentShell` / `ToolShell` via `defineAsyncComponent`, loading only the shell selected by `resolveOcliveShell()`; the non-rendered shell no longer ships in the first-screen main chunk.
- **Hot-path DB merge (K-PERF-03~06)**: one `EffectiveSessionConfig` per turn; single `get_role_runtime_snapshot` read; shared `TurnPrefetch` / skip agent DB when `agent=none`; one memory-decay transaction. Baseline: `handoff/OPUS_48_PERF_BASELINE.md`.
- **Long-lived memory/SQLite (K-PERF-07/08/12)**: `SessionCache` six-map cap+TTL; `personality_vector` composite index migration `033`; `hybrid_store` drops redundant `get_chat_session`; `role_cache` LRU(32); LLM startup probe runs in background.
- **In-shell lazy panels + poll backoff (K-PERF-10/11)**: non-first-screen panels in `FluentShell`/`ToolShell` via `defineAsyncComponent`; `useKernelStatus` backs off to 60s when the tab is hidden.
- **RoleRuntimeSnapshot downstream reuse (K-PERF-20)**: `relation_snapshot` / `post` / `pre` Profile paths share one snapshot; at most one `get_current_emotion` per turn (except post-write refresh).
- **Ollama model settings batch read (K-PERF-21)**: `resolve_effective_ollama_model` uses a single `get_app_settings([provider, remote_model])` round trip.
- **Chat session list + upsert (K-PERF-22)**: session list snippet via window-function JOIN; `upsert_chat_session` `RETURNING` removes post-write re-read.
- **Long-term memory & operation-log indexes (K-PERF-23)**: migration `034_perf_indexes.sql` (`idx_ltm_role_content` / `idx_operation_logs_role`).
- **Fewer post-phase Role clones (K-PERF-24)**: `TurnContext.role_arc` reused for profile evolution spawn.
- **`pre_llm` Wave 1 parallelism (K-PERF-14)**: `turn_pipeline/pre.rs` uses `tokio::try_join!` for five read-only paths (context / emotion / model / narrative hint / memories); `oclive_turn` logs `pre_llm_wave1` aggregate; see `PERFORMANCE.md` §6.

### Fixed

- **Chat history vanished in story scenes**: cold start uses unified `bootstrapChatForRole` (await fetch + `beginNewChatSessionOnRestart` fold); removed `interactionMode` watch `immediate` race; `loadedBucketKeys` prevents empty-placeholder short-circuit; role switch probes backend session scenes / pack scenes / IDB index fallback. Guards `chatStoreScene.test.ts`, `chatStoreLoad.test.ts`, see [`CHAT_STORAGE_ARCHITECTURE.md`](handoff/CHAT_STORAGE_ARCHITECTURE.md).
- **Ctrl+Shift+S did not open Settings**: `useGlobalHotkeys` referenced an unpassed `opts.openSettingsView` (`undefined` at runtime); now calls the local `openSettingsView`; the theater shell still emits `theater:settings`.
- **Voice plugin `get_plugin_settings_ui` bridge failure**: `ui_slots` invoked plugin settings read/write via `plugin_bridge_invoke`, but desktop `dispatch_local_bridge_command` did not route `get_plugin_settings_ui` / `set_plugin_settings_config` (surfaced as `unsupported bridge command`); now delegated to `plugin_config.rs`.

---

## [0.4.0] - 2026-06-12

### Added

- **Portrait catalog (A2/B1)**: `portrait_catalog.json` SSOT; seven fixed slots + advanced multi-entry; additive `visual_state_id` / `performance_directive` DTOs.
- **Performance director**: `pick_portrait_with_catalog` + complex-emotion `narrative_hint` closed loop; legacy `portrait_emotion` seven-tag zero regression.
- **Visual presentation v1**: `materialize_directive` (image/live2d/rig3d/procedural); distro `[visual_presentation].mode` gating (`off` / `image_only` / `stage_full`).
- **OOCP S16**: fixed disabled/enabled fixtures assert visual-field omission and `visual_state_id` + `performance_directive` output, so official role evolution cannot invalidate the protocol test.
- **Pack editor**: `PortraitCatalogEditor`, tiered export profiles (`desktop-full` / `vscode-lite` / `theater`), `visual_presentation` UI.
- **VS Code Flash**: HTTP parses `visual_state_id` / `performance_directive`; catalog path preferred over tag filenames.
- **Theater**: `TheaterStagePanel` + `Live2DStageAdapter` wiring (Cubism defer, PNG fallback).

### Changed

- RFC portrait/visual presentation status updated to Phase 1–4 delivered.
- `theater.oclive.toml` bundled profile synced to `stage_full`.

---

## [0.3.0] - 2026-06-07

**Desktop host `0.3.0`** · **VS Code extension `0.3.0`** · **`SendMessageResponse.schema` 14**

### Breaking

- **`SendMessageResponse.schema`** bumped to **14**: optional `raw_reply` when `include_raw_reply: true` and post-processor changed the LLM text.
- **`high_risk_grants.json`**: only canonical permission keys (`mcp:http`, `mcp:stdio`, `process:spawn`, `network:*`); legacy aliases no longer read — migrate files and re-grant.

### Changed

- **Experimental dual-core runtime**: `oclivenewnew-tauri` Cargo feature **`dual_core`** (default off); build with `cargo build -p oclivenewnew-tauri --features dual_core`.
- **Dual-core status wording**: runtime dual-core docs now use **Opt-in Beta (default off)**; Stable remains the default delivery path.

### Added

- **User Identity & Reply Post-Processor Phase 2 (closure)**: HostProfile merge; remote/directory backends; HTTP `/user_identity/*`; desktop & VS Code identity switch; `RoleInfo` / `GET /role_info` read-only post-processor fields; debug panel status line. See the [archived Phase 2 record](handoff/archive/USER_IDENTITY_REPLY_POST_PROCESSOR_PHASE2.md).
- **Docs**: ROLE_PACK_SPEC §1.1 / §9.7, architecture overview orthogonal units, USER_MANUAL §3.4–3.5, RFC Phase 2 acceptance checked.
- **Forgetting curve & relation evolution (`config.json`)**: Ebbinghaus long-term memory decay; mention reinforcement; immersive favorability estrangement; virtual time ratio; memory-shaped personality nudges. See [ROLE_PACK_SPEC §9](creator-docs/role-pack/ROLE_PACK_SPEC.md).
- **Dual-core quality hardening**: optional **S14** OOCP scenario; CI dual-core build + `--include-dual-core`; `dual_core_happy_path.rs` integration test.

#### Chat Storage (phase 3)

- **Pluggable backends**: `hybrid`, `file`, `sqlite`; env or `config.json` → `chat_storage.backend`.
- **Memory replay**: `replay_memory_extraction` / `get_replay_progress` with configurable similarity threshold.
- **File backend**: search, replay, `list_sessions_by_role`.
- **Capability detection & UI**: `get_chat_storage_capabilities`; gated storage panel.
- **Config**: optional `chat_storage.backend`, `replay_similarity_threshold`.
- **Developers**: extended `ConversationStore` trait. See [handoff/CHAT_STORAGE_ARCHITECTURE.md](handoff/CHAT_STORAGE_ARCHITECTURE.md).

---

## [0.2.0] - 2026-05-22

**Desktop host `0.2.0`** · **`oclive-cli` `0.1.0`** · **`oclive_kernel_runtime` `0.2.0`** (independent SemVer; see [RELEASE_VERSIONING.md](creator-docs/development/RELEASE_VERSIONING.md)).

### Breaking

- **Role pack v2:** new packs use **`pipeline.ocblueprint`** (`schema_version: 2`) as the sole config hub; **`oclive pack validate` defaults to v2**. Migration: [V1_TO_V2_MIGRATION.md](creator-docs/role-pack/V1_TO_V2_MIGRATION.md).
- **CLI:** removed top-level `publish`, `plugin search/update`, `registry login` (see [DEPRECATED_COMMANDS.md](./kernel/crates/oclive-cli/DEPRECATED_COMMANDS.md)).

### Added

- **Blueprint v2 & architecture graph:** `slot_registry`, session `set_session_slot_override`, persist **`save_role_slot_registry`**; golden packs such as **`roles/mumu`** migrated.
- **Dual-core:** `runtime_config.dual_core` + `pipeline.experimental` steps with silent fallback to stable `co_present` (off by default).
- **`oclive-cli` toolchain** (22 top-level subcommands): `init` (incl. **`--monolith`**), `build`, `bench` (`--matrix` / `--cold-start` / `--soak` / `--save`), `dev`, `pack`, `doctor`, `test --oocp`, `explain`, etc.; see [OCLIVE_CLI_GUIDE.md](creator-docs/cli/OCLIVE_CLI_GUIDE.md).
- **Monolith weld mode:** `init --monolith` → `build` → dual-binary **`bench`**; [RFC_OCLIVE_MONOLITH_MODE.md](creator-docs/rfc/RFC_OCLIVE_MONOLITH_MODE.md).
- **HTTP `--api`:** `GET /health`, `POST /chat`; CI **OOCP black-box S0–S11** + process-restart smoke test.
- **Agent / MCP**, directory-plugin high-risk grants, plugin HTML **`OclivePluginBridge`**, market index install.
- **Startup health** `startup_health`; **`oclive explain`** for all `AppError` codes; **`oclive doctor`** blueprint checks.
- **Orchestration:** `TurnContext`; `AppStateBuilder` + policy registry split; rolling file logs (`OCLIVE_LOG_DIR` / `--api`).

#### Role-pack chat storage location

- **`config.json` → `chat_storage.location`**: new `"role_pack"` / `"global"` (default `"global"`, backward compatible). With `"role_pack"`, chat logs live under the role pack `chats/` subdir; falls back to the global path with a warn if the role pack directory is not writable.
- **Init scaffold:** `oclive-cli init` adds a “Chat storage location” step (follow role pack / global).
- **Storage management panel:** shows the current location badge when a role is selected (follows role pack / global).
- **Export format change:** `export_role_chats` output changed from ZIP+base64 to combined JSON (`application/json`); content unchanged; `zip` dependency removed.

### Changed

- **Main orchestration:** Tauri and HTTP both use **`process_message`**; entry blueprint is **not** the first-turn DSL scheduler.
- **Pack format:** `pack validate` defaults to v2 (`--profile legacy` for old packs); manifest/settings top-level key allowlist tightened.
- **Tauri:** `generate_handler!` grouped by domain with comments; dropped `reqwest` `blocking` and direct `@tauri-apps/api/fs` (custom commands); plugin bridge script as frontend IIFE asset.
- **Architecture graph v2:** removed hand-drag connection composable (edges derived from `slot_registry`).
- **Frontend:** i18n domain split, modular `tauri-api`, Vite vendor chunks; `TopBarMorePanel` extracted from `App.vue`.

### Fixed

- **Errors:** unified **`AppError` / `KernelErrorBody` JSON** + frontend **`apiErrors`** (invoke and HTTP same shape).
- **SQLite:** WAL + pool (`sqlite_pool.rs`); Release profile tuning (`opt-level=3`, `codegen-units=1`).
- **Concurrency:** in-memory **`Cache`** read-first + capacity cap; role cold-load **`DashMap` inflight** (no `Arc::strong_count`).
- Plugin subscription races, custom events wrongly filtered by `bridge.events`, visible warnings when Remote URLs unset, etc.

### Performance

- Release binary sampling ~**12 MiB PE / 7.6 MiB .text** (see [PERFORMANCE.md](creator-docs-en/getting-started/PERFORMANCE.md)).
- Directory-plugin IPC in-flight dedupe (catalog / bootstrap / plugin_state); `pluginStore` refresh and slot memo optimizations.

### Engineering

- Workspace **`cargo clippy -D warnings`** aligned with CI; shared **`oclive_validation`**; **11** `invoke` hot-path integration tests.
- **`npm run check:release`** release gate; Playwright **`vite preview`** smoke (Ubuntu CI).

### Documentation

- [COMPATIBILITY.md](creator-docs-en/COMPATIBILITY.md), [PRODUCT_RELEASE_CHECKLIST.md](handoff/archive/PRODUCT_RELEASE_CHECKLIST.md), bilingual **creator-docs-en** mirror and blueprint v2 doc closure.

---

## [0.2.0] — 2026-04-02

(Earlier items in the 0.2.x cycle; summarized in the **0.2.0** release above.)

### Added

- Large pack import progress: backend `import_progress` events + frontend progress modal.
- Pre-import preview (`manifest.json` peek) and conflict dialog when role ID exists.
- Import **`.zip`** (same as `.ocpak`) and **extracted folders**; see `roles/README_MANIFEST.md`.
- Scene welcome after `switch_scene`; relation tier upgrade via `relation_state` on `send_message`.

### Changed

- Virtual scroll always on when messages exist; export filename default `{role_name}_{version}.ocpak`.

### API

- `send_message` adds `relation_state`; `emotion` remains user-input analysis.

---

## [0.1.0]

- Initial public baseline (first tagged version in repo).
