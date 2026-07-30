# RFC: Blueprint Extension Envelope and Unified Resource Coordination

[中文](../../creator-docs/rfc/RFC_BLUEPRINT_EXTENSION_AND_RESOURCE_COORDINATION.md)

| Metadata | Value |
|----------|-------|
| Status | **Boundary accepted · v4 envelope and Capability/Plan diagnostics implemented · Resource Adapter Registry and first LLM/voice slice implemented** |
| Last updated | 2026-07-29 |
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

Current implementation boundary (2026-07-29):

- Rust/JSON Schema, CLI/doctor, Host, and the pack editor implement the v4 envelope, path safety, and opaque payload round-trip.
- The host implements a directory-Provider Capability Registry, deterministic Provider selection, permission/dependency/enablement checks, required/optional activation gates, and read-only structured diagnostics through Tauri and the CLI. The same pack can produce different plans under different `HostProfile`s.
- A capability becomes active only when the host has registered a real consumer. The first registration is Chat Pro `voice.asr`; an arbitrary manifest `provides` entry cannot expand kernel behavior.
- `ExecutionPlan` still resolves capabilities and effective six-slot backends without starting Providers or rewriting packs. Pure compilation and the CLI doctor retain `resource_coordination: not_evaluated`; desktop Tauri diagnostics refresh the host coordinator and report `ready | degraded | blocked`.
- The host exposes NVIDIA multi-device snapshots plus admission, lease, priority, pressure, and resource diagnostics v2. `HostProfile` supplies the safety reserve and lease TTL policy. The Resource Adapter Registry reports control mode, adapter-local profiles, residency support, lifecycle operations, and current leases without scheduling them.
- The first adapter loop covers managed llama-server cold starts, observe-only Ollama/LLM foreground activity, and official bundled CosyVoice2 `voice.warm` / `voice.speak`. Leases carry `profile_id`, and registered adapters reject unknown profiles. Cloud TTS, user-hosted HTTP TTS, and community plugins are not misclassified as host-managed resources.
- Existing profiles remain `coordinator_selectable=false`; automatic switching is not claimed. Third-party registration, fair queues/preemption and recovery, real multi-profile transitions, RAM/CPU, render adapters, and shared-VRAM stress/soak remain future work.

## Capability Provider versus Resource Adapter

| Example | Capability Provider | Resource Adapter |
|---------|---------------------|------------------|
| Text formatter | Required | No |
| Distro-specific content injection | Required | Usually no |
| Local LLM | Required | Yes |
| Local TTS | Required | Yes |
| Cloud TTS | Required | Usually no |
| Live2D renderer | Required | Yes when it shares managed GPU resources |

Current directory manifest `schema_version: 1` contributes `provides`, Provider `version`, `process`, dependencies, and `permissions` to Registry diagnostics; [`PLUGIN_V1`](../plugin-and-architecture/PLUGIN_V1.md) owns those field semantics. Host/API semver ranges, third-party Resource Adapter entry points, and manifest resource declarations are not implemented. Built-in LLM/voice adapters do not expand the directory-plugin schema. A displayed Provider version is not an API-compatibility negotiation.

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

### Current first-slice boundary

- `NvidiaSmiResourceSnapshotSource` provides best-effort snapshots and an explicit unavailable state instead of invented zero-capacity data.
- Cold-start admission atomically evaluates the target GPU, current free VRAM, concurrent pending reservations, and `gpu_safety_reserve_mib`. `OCLIVE_GPU_DEVICE_INDEX` / `CUDA_VISIBLE_DEVICES` may select the device.
- Managed llama-server acquires a lease before spawn and releases it on model changes, startup failure/timeout, degradation, or suspension. External Ollama records foreground activity without adding an `nvidia-smi` process to the token hot path.
- Only official bundled CosyVoice2 is host-admitted. The host lease replaces the old fixed mixed-fp16 cold-load gate; FP32 expansion keeps the stricter sidecar-local gate.
- RAII releases pending leases on cancellation/failure. When TTS is disabled or bundled synthesis is left, the host sends an `unload` carrying the adapter and runtime profile; the sidecar waits for current synthesis, validates the target model, and releases it. The host removes the voice lease only after a matching success acknowledgement; otherwise it conservatively retains the lease with `resource_release_unconfirmed`. Cold-start RPCs and configuration transitions share the per-adapter serialization lock, closing races between release acknowledgement, lease reuse, and consecutive settings saves.
- This slice does not kill an active LLM request to load voice and does not claim control over arbitrary external Ollama processes. Automatic preemption/recovery and shared-VRAM stress/soak remain debt.
- Automatic llama-server → Voice preemption remains disabled: after suspending the managed primary, current requests can fall back to observe-only Ollama, and the host cannot yet guarantee that it will not reclaim the same GPU. A coordinated fallback gate and recovery policy are prerequisites.
- The host Resource Adapter Registry records truthful control, profile, residency, and lifecycle support for llama-server, Ollama, the performance activity observer, and bundled CosyVoice2. Observe-only entries may claim only `observe`; generic lifecycle/profile commands are not dispatched yet.

## Control messages

Public DTOs in `oclive_kernel_types::resource_coordination` now cover snapshots, adapter descriptors/profiles, admission, leases, priorities, pressure, and diagnostics; `oclive_kernel_contracts::ResourceSnapshotSource` owns device collection. Active degrade/suspend/resume requests and generic runtime-state messages remain future adapter protocol. Tokens, PCM, image frames, and renderer parameters stay on their existing domain channels.

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
