# llama.cpp LoRA GGUF and `.ocadapter`

[中文](../../creator-docs/plugin-and-architecture/LORA_ADAPTER_PACKAGE.md)（规范 SSOT）

**Status:** v1 implemented for the managed local performance runtime.

## Scope

The main application loads only LoRA adapters already converted to **llama.cpp GGUF**. It does not interpret or convert Hugging Face/PEFT `adapter_config.json`, `.safetensors`, tokenizer data, or training-framework state.

- OCLive owns local import, validation, managed storage, adult-content acknowledgement, activation rollback, and `llama-server --lora`.
- `llama-server` owns weight loading and the final tensor/base-model compatibility decision.
- A later independent PEFT plugin may download or import PEFT packages and convert them into the GGUF/package contract below. Python/Transformers must not become a stable-kernel dependency.
- Existing `expert_routing` still selects declared directory LLMs. The single adapter selected under Local Model Settings does not change expert-routing semantics.

## Raw GGUF import

Raw `.gguf` imports must use GGUF v2 or v3 and contain `general.type=adapter` plus `adapter.type=lora`. Imports are capped at 16 GiB and hashed while streaming. Their deterministic id is `local.lora.<first 16 SHA-256 characters>` and their version is `0.0.0-local`.

## `.ocadapter` v1

The file is a ZIP with exactly one root `adapter.json` and exactly one referenced adapter GGUF:

```json
{
  "schemaVersion": 1,
  "id": "com.example.mumu-style",
  "name": "Mumu Style",
  "version": "1.0.0",
  "format": "llama.cpp-lora-gguf",
  "adapterFile": "weights/adapter.gguf",
  "adapterSha256": "64-character-lowercase-sha256",
  "baseModel": "optional human-readable base model",
  "architecture": "llama",
  "contentRating": "general",
  "description": "optional",
  "license": "optional SPDX id or short label",
  "source": "optional source URL"
}
```

`baseModel` is human-readable training provenance or import context for compatibility review and traceability. It does not bind the adapter to a particular local base file. Actual activation still depends on GGUF architecture, integrity validation, and the `llama-server` load result.

`id` is 1–96 ASCII letters, digits, `.`, `_`, or `-` and cannot start with `.`. `name` is 1–160 characters; `version` is 1–64 characters. `format` is fixed to `llama.cpp-lora-gguf`. `adapterFile` is a relative `.gguf` path with no absolute or parent traversal and no duplicate entry. `contentRating` is `general` or `adult` and defaults to `general`. When both manifest and GGUF contain `architecture`, they must match. OCLive writes `installedAt`; packages may omit it.

## Storage and transactions

Installed files live at `<canonical models>/adapters/<id>/{adapter.json,adapter.gguf}`. Import uses a same-filesystem staging directory and atomic rename. Replacement requires explicit user consent, backs up the previous directory, and restores it if commit fails.

Activation requires a saved GGUF base model and the managed performance distro runtime. OCLive revalidates the adapter metadata and SHA-256, checks architecture when available, then starts:

```text
llama-server -m <base.gguf> --lora <adapter.gguf>
```

An external server occupying the endpoint is never reported as having loaded the selected adapter. Failed activation restores the previous database values, environment, and managed runtime selection. Adult-rated adapters require explicit acknowledgement, and active adapters must be deactivated before deletion.

`OCLIVE_LOCAL_LLM_LORA_PATH` is an internal database-to-process bridge, not the recommended manual configuration surface.

## Independent base models and content rating

An independent base in a first-level directory under canonical `models/` is listed through a sibling `<file>.ocmodel.json`. Loose GGUF/BIN files at the root remain general-rated legacy entries. Files in child directories require a sidecar, and `adapters/` plus `downloads/` are never scanned as base models.

```json
{
  "schemaVersion": 1,
  "kind": "oclive.local-base-model",
  "fileName": "example.Q4_K_M.gguf",
  "name": "Example full base",
  "contentRating": "adult",
  "description": "optional",
  "license": "Apache-2.0",
  "source": "optional source URL",
  "sha256": "optional 64-character SHA-256"
}
```

`fileName` must match the adjacent model file, and `contentRating` is `general` or `adult`. When `sha256` is declared, OCLive recalculates it on a background worker and requires a match before switching to that base. Selecting a new adult-rated full base requires acknowledgement in both the UI and kernel.

**Independent bases and LoRAs are never permanently bound.** Users may freely select any pairing that passes architecture, file-integrity, and runtime-load checks; provenance is recorded for traceability rather than binding. Labels such as “first setup” or “second setup” are only test shorthand and are not part of the persistent contract or product semantics.

Whenever the base-model path changes, OCLive automatically deactivates the current LoRA; the user must explicitly activate the adapter intended for the new base. This prevents a same-architecture but wrong-origin adapter from leaking into another combination, including accidental abliteration-LoRA stacking on an already abliterated full base.

Sidecar metadata records provenance and rating; it does not establish enterprise rights to third-party weights. Distributors still need to verify license, training-data rights, and applicable law.

## Deferred from v1

Hugging Face Hub and PEFT conversion, multiple/scaled adapters, revision/tokenizer/chat-template fingerprints, role/expert binding UI, and package signatures remain deferred under `V-LORA-PACK-03` and the independent `V-LORA-PEFT-04`.
