# RFC: Blueprint Extension Envelope and Unified Resource Coordination

[中文](../../creator-docs/rfc/RFC_BLUEPRINT_EXTENSION_AND_RESOURCE_COORDINATION.md)

| Metadata | Value |
|----------|-------|
| Status | **Boundary accepted · v4 blueprint envelope implemented · resource diagnostics v5 and the generic coordination control plane implemented** |
| Last updated | 2026-08-01 |
| Audience | Kernel, editor, distro, directory-plugin, and commercial extension developers |
| Scope | Minimal blueprint extension envelope, capability resolution, `ExecutionPlan`, Resource Coordinator, and adapter ownership |

## Decisions

1. OCLive owns a small, strict, migratable extension envelope. Extension authors own payload schemas, implementations, UIs, migrations, documentation, and support.
2. A blueprint is design intent. It must not directly issue process, VRAM, unload, or preemption commands.
3. The host compiles blueprint intent, `HostProfile`, user settings, the capability registry, and device state into an internal `ExecutionPlan`.
4. Resource coordination uses centralized policy and domain-specific adapters. GPU is the first managed resource, but the contract also leaves room for RAM, CPU, multiple devices, and managed processes.
5. Resource-control messages are separate from data paths. Tokens, PCM, image frames, and Live2D parameters do not travel through the coordinator.
6. Every new capability needs a Capability Provider. Only capabilities that consume or control shared resources need a Resource Adapter.

The design follows the same ownership principle as Chat Pro's `adult_extension.json`, but content extensions and blueprint capability extensions remain different contracts.

## Stable v4 envelope

This envelope is implemented by **`schema_version: 4`**. v4 is the Stable successor to v2. Frozen dual-core Beta v3 remains separate; v4 does not inherit v3 `pipeline`, `zone`, or `dual_core`.

```json
{
  "extensions": {
    "com.example.live2d.main": {
      "capability": "render.live2d",
      "provider": "com.example.live2d",
      "required": false,
      "config_schema_version": 1,
      "config_ref": "blueprint/extensions/com.example.live2d.main/config.json"
    }
  }
}
```

The core owns the instance id, `capability`, optional `provider`, `required`, positive payload version, and safe package-relative `config_ref`. The extension author owns the referenced payload. Large inline configuration and arbitrary root keys are intentionally excluded.

If an optional extension is unavailable, the host preserves it, removes it from the effective plan, and reports a visible degradation. If a required extension is unavailable, role metadata may remain inspectable for repair, but the blueprint cannot be activated. Editors and CLIs must round-trip unknown optional extensions without deleting their payload.

Current implementation boundary (2026-07-31):

- Rust/JSON Schema, CLI/doctor, Host, and the pack editor implement the v4 envelope, path safety, and opaque payload round-trip.
- The host implements a directory-Provider Capability Registry, deterministic Provider selection, permission/dependency/enablement checks, required/optional activation gates, and read-only structured diagnostics through Tauri and the CLI. The same pack can produce different plans under different `HostProfile`s.
- A capability becomes active only when the host has registered a real consumer. The first registration is Chat Pro `voice.asr`; an arbitrary manifest `provides` entry cannot expand kernel behavior.
- `ExecutionPlan` still resolves capabilities and effective six-slot backends without starting Providers or rewriting packs. Pure compilation and the CLI doctor retain `resource_coordination: not_evaluated` and do not produce a device plan. Desktop Tauri diagnostics refresh the host coordinator and attach the same read-only `candidate_plan` as `ExecutionPlan.resource_plan`.
- The host exposes NVIDIA multi-device, system-RAM, and CPU snapshots plus atomic admission, leases, priorities, pressure, and resource diagnostics v5. `HostProfile` supplies per-resource reserves, lease TTLs, queue/aging policy, automatic-preemption policy, and finite scheduling intent. The Resource Adapter Registry reports control mode, registration provenance, adapter-local profiles, residency support, lifecycle operations, and current leases, and validates scheduling intent without executing it.
- The first adapter loop covers managed llama-server cold starts, observe-only Ollama/LLM foreground activity, and official bundled CosyVoice2 `voice.warm` / `voice.speak`. Leases carry `profile_id`, and registered adapters reject unknown profiles. Cloud TTS, user-hosted HTTP TTS, and community plugins are not misclassified as host-managed resources.
- The candidate-plan compiler deterministically derives profile selections, proposed transitions, capacity results, rollback actions, and stable reasons from intent plus current GPU/RAM/CPU facts. The result carries `compiled_from_revision`; reading it never dispatches lifecycle operations.
- The generic execution foundation registers one authoritative `ResourceAdapterController`, an owner-namespaced third-party `ResourceAdapterRegistrar`, exact requester → target → operation grants, per-adapter locks, stale-plan rejection, and reverse rollback. The in-process registration port does not load plugins, interpret untrusted manifests, or grant cross-adapter authority; directory-manifest resource declarations remain unimplemented.
- Performance llama exposes three real tiers—`gpu_full`, `gpu_balanced`, and `cpu_compatibility`—that change `llama-server --n-gpu-layers` and fall through after admission denial. The admission queue provides priority ordering, fair aging, timeout, and cancellation cleanup. Automatic preemption touches only lower-priority, reversible, exactly authorized managed adapters and restores them in reverse order.
- Generic `render`, `compute`, and `hybrid` resource domains plus a third-party Render adapter capacity/preemption test prove that the control plane is not hard-coded for LLM/voice. Chat Pro does not yet bundle a Live2D runtime; `Live2DStageAdapter` continues to statefully fall back to PNG. Direct-adapter shared-VRAM short stress and deterministic failure/concurrency plus a 10,000-transition in-process soak have been exercised. The CLI real-wall-clock kernel-process harness has completed short calibration, and PR #147's remote main CI plus strict audit both passed. Long bundled LLM/CosyVoice/future-Live2D shared-hardware soak, the full failure matrix for uncontrollable external processes, and a real bundled Live2D runtime remain open.

## Capability Provider versus Resource Adapter

| Example | Capability Provider | Resource Adapter |
|---------|---------------------|------------------|
| Text formatter | Required | No |
| Distro-specific content injection | Required | Usually no |
| Local LLM | Required | Yes |
| Local TTS | Required | Yes |
| Cloud TTS | Required | Usually no |
| Live2D renderer | Required | Yes when it shares managed GPU resources |

Current directory manifest `schema_version: 1` contributes `provides`, Provider `version`, `process`, dependencies, and `permissions` to Registry diagnostics; [`PLUGIN_V1`](../plugin-and-architecture/PLUGIN_V1.md) owns those field semantics. A host extension may register an adapter descriptor and optional authoritative controller through the in-process `ResourceAdapterRegistrar`; its canonical adapter id must equal or descend from the registration owner, and it cannot claim `builtin.*`. Host/API semver ranges, manifest resource declarations, and automatic manifest-to-adapter assembly are not implemented. Built-in LLM/voice adapters do not expand the directory-plugin schema. A displayed Provider version is not an API-compatibility negotiation.

## `ExecutionPlan`

`ExecutionPlan` is an in-memory, host-owned normalized plan:

```text
role content + blueprint intent
  + HostProfile + user/session settings
  + Capability Registry + resource snapshot
                     ↓
                Plan Compiler
                     ↓
                ExecutionPlan
```

It resolves providers, permissions, dependencies, required/optional degradation, registered stable templates, and resource claims. It is not persisted and is not a third-party schema.

The current `co_present_stable` plan contains effective six-slot backends, extension Provider/version selection, candidates, permission/dependency reasons, and activation status. Device and lease evaluation remains a separate host-owned Resource Coordinator concern and is never written into the blueprint. Pure Plan Compiler / CLI diagnostics do not probe devices; desktop diagnostics do.

The current Stable order remains owned by `process_message` / `turn_pipeline`. Future limited freedom must use registered templates or constrained partial orders, not arbitrary `steps[]`. This RFC does not reuse or unfreeze v3 `pipeline.stable` / `pipeline.experimental`.

## Resource Coordinator

The Resource Coordinator is an in-process control plane. It is not the existing kernel attach/replace scheduler, a blueprint executor, an LLM-output merger, or a business-data bus.

The coordinator owns:

- device snapshots and global budgets;
- leases and releases;
- foreground, warm-up, and persistent-render priorities;
- fair queues, timeouts, cancellation, preemption, and recovery;
- degradation decisions such as GPU layers, precision, frame rate, CPU/cloud fallback, or disablement;
- user-visible diagnostics.

LLM, voice, render, and future adapters own runtime discovery, static/dynamic estimates, telemetry, lifecycle actions, and truthful completion reporting. External runtimes must report whether they are `managed` or `observe_only`; the coordinator must not assume it can evict every GPU process.

Blueprints may express quality and degradation intent, but they do not hard-code executable VRAM allocations. Actual claims combine adapter estimates, the selected model/configuration, telemetry, and host/user policy.

### Finite scheduling intent

Scheduling freedom uses a small, composable, statically validated vocabulary rather than arbitrary DAGs, scripts, or VRAM-number commands. A distro may currently declare one of four objectives under `HostProfile` `[resource_coordination]`: `compatibility_first`, `primary_first`, `latency_first`, or `custom`. It may combine six constraint forms: `require`, `residency`, `coexist`, `exclusive`, `yield_then_run`, and adapter-local `fallback`.

These “commands” are declarative constraints and preferences, not lifecycle calls sent to a process. Authority is ordered as follows: physical safety and confirmed device state; adapter `managed` / `observe_only` boundaries; HostProfile and user policy; then blueprint intent. Lower layers cannot override higher-layer facts.

Resource diagnostics v5 separates two questions. `scheduling.state = ready | degraded | blocked` says only whether the registry accepts the declaration. `candidate_plan.state` then combines the declaration with current leases, GPU/RAM/CPU capacity, truly selectable profiles, registered controllers, and rollback support. `candidate_plan.executable=true` means that required profile selection is complete and every proposed transition has a controller plus rollback operation; it does not mean that anything ran. Execution must recheck `compiled_from_revision`, and final adapter admission remains the physical-capacity authority.

A future blueprint may reference registered finite templates or equivalent constrained preferences. It must not write HostProfile values, VRAM quantities, or direct `start` / `unload` actions. No role-pack resource-scheduling field is added in this slice.

### Current first-slice boundary

- `NvidiaSmiResourceSnapshotSource` provides best-effort GPU snapshots and an explicit unavailable state instead of invented zero-capacity data. The same snapshot uses `sysinfo` for available/total system RAM and logical/physical CPU counts, so a GPU probe failure does not become a false zero-capacity machine.
- Cold-start admission follows a fair queue and atomically evaluates target GPU, system RAM, CPU, reserves, and concurrent pending claims. Priority ordering plus wait aging prevents starvation; timeout and cancellation remove waiters safely. `OCLIVE_GPU_DEVICE_INDEX` / `CUDA_VISIBLE_DEVICES` may select the device.
- Managed llama-server selects `gpu_full`, `gpu_balanced`, or `cpu_compatibility` from HostProfile/environment before spawn and falls through on capacity denial; every tier changes the real `--n-gpu-layers` value. It releases its lease on model changes, startup failure/timeout, degradation, or suspension. External Ollama records foreground activity without adding an `nvidia-smi` process to the token hot path.
- Only official bundled CosyVoice2 is host-admitted. The host lease replaces the old fixed mixed-fp16 cold-load gate; FP32 expansion keeps the stricter sidecar-local gate.
- RAII releases pending leases on cancellation/failure. When TTS is disabled or bundled synthesis is left, the host sends an `unload` carrying the adapter and runtime profile; the sidecar waits for current synthesis, validates the target model, and releases it. The host removes the voice lease only after a matching success acknowledgement; otherwise it conservatively retains the lease with `resource_release_unconfirmed`. Cold-start RPCs and configuration transitions share the per-adapter serialization lock, closing races between release acknowledgement, lease reuse, and consecutive settings saves.
- This slice does not kill an active LLM request to load voice and does not claim control over arbitrary external Ollama processes. Automatic preemption considers only lower-priority active managed leases after GPU/RAM/CPU capacity denial. A target must declare a reversible `automatic_preemption` operation, its corresponding recovery operation, and an exact requester → target grant; equal/higher-priority and observe-only resources are untouched. Failure rolls back in reverse order, and successful callers restore victims in reverse order. Batch execution is not exposed to role packs, plugins, or HTTP clients. Direct-adapter short-stress results are recorded in [`handoff/TTFT_BENCHMARK.md`](../../handoff/TTFT_BENCHMARK.md).
- Performance LLM now has one coordinated request gate. A resource transition rejects new generation/probe requests, drains admitted primary and Ollama fallback calls, then stops the host-managed llama-server and unloads only Ollama models tracked by this OCLive runtime. Ordinary primary failures and cloud-provider suspension preserve their previous fallback behavior. `generate`, tag generation, opts, streaming, and startup probes share the gate. Concurrent explicit warm-up/runtime selection is recorded as a superseding recovery; the gate does not reopen before drain/unload completes, and recovery warm-up waits for the gate to become open.
- Official bundled CosyVoice `voice.warm` never preempts LLM. `voice.speak` may preempt only for a local provider after a headroom denial, with no active Performance request and controllable local residency. Chat Pro defers speech while text is still generating, then forces the final retry through host RPC. Before that RPC returns full audio, the plugin performs a matching CosyVoice unload; only a confirmed release lets the host remove the Voice lease and recover Performance LLM. An unconfirmed or cancelled call conservatively retains/promotes the Voice lease and suspension with a stable reason so later speech or configuration release can recover; model recovery also refuses to overlap retained Voice residency. The bundled direct-stream endpoint is exposed only after a host-admitted warm succeeds, preserving the existing low-latency path on machines where both runtimes fit.
- The host Resource Adapter Registry records truthful control, profile, residency, and lifecycle support for llama-server, Ollama, the performance activity observer, and bundled CosyVoice2. Observe-only entries may claim only `observe`. Only the three host-controlled llama tiers declare `coordinator_selectable=true`; desktop-owned CosyVoice and external Ollama are not misrepresented as kernel-controlled.
- `ResourceAdapterRegistrar` lets HostExtension/directory-plugin bridge code register third-party facts and a controller under an owner namespace. This is an in-process contract, not a directory-manifest resource field or arbitrary plugin loader. `render`, `compute`, and `hybrid` resource domains and a third-party Render test exist, but a bundled Live2D Provider/runtime does not.

## Control messages

Public DTOs in `oclive_kernel_types::resource_coordination` now cover GPU/RAM/CPU snapshots, adapter descriptors/profiles/registrations, finite scheduling intent, candidate plans, admission queues, leases, priorities, pressure, preemption, optimistic-version transitions, and diagnostics. `oclive_kernel_contracts` owns `ResourceSnapshotSource`, the authoritative `ResourceAdapterController`, and the owner-constrained `ResourceAdapterRegistrar` ports. Batch candidate-plan execution remains host-internal and is not a plugin or role-pack protocol. Tokens, PCM, image frames, and renderer parameters stay on their existing domain channels.

## Compatibility and rollout

- Write `extensions` only in v4; strict v2/v3 contracts still reject it.
- Frozen v3 remains separate. v4 does not reuse or unfreeze arbitrary `steps[]` / `zone` semantics.
- Keep the core schema strict and payload schemas namespaced and extension-owned.
- A role pack may declare minimum capabilities but must not trigger self-updates.
- First converge current schema/runtime drift, then implement envelope round-trip, capability resolution, a diagnostic Plan Compiler, and finally the LLM/voice resource-coordination slice.
- Add Live2D/3D only after the first two adapters prove that the coordinator is not hard-coded for model inference.

Progress state is maintained only in the [technical-debt inventory](../../handoff/TECHNICAL_DEBT_INVENTORY.md).

## Related

- [Role pack and blueprint boundary](../../handoff/ROLE_PACK_BOUNDARY.md)
- [Blueprint folder layout](../../handoff/BLUEPRINT_FOLDER_LAYOUT.md)
- [Module registry](../../handoff/MODULE_MAP_AND_HANDOFF.md)
- [Kernel scheduler rescope](../../handoff/KERNEL_SCHEDULER_RESCOPE.md)
- [Distro capability profile](../../creator-docs/kernel/DISTRO_CAPABILITY_PROFILE.md)
