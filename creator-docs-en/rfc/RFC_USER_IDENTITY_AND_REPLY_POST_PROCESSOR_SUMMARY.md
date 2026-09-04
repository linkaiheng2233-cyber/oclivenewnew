# User identity & reply post-processor — English summary

[中文](../../creator-docs/rfc/RFC_USER_IDENTITY_AND_REPLY_POST_PROCESSOR.md)

Full RFC (Chinese SSOT): [RFC_USER_IDENTITY_AND_REPLY_POST_PROCESSOR.md](../../creator-docs/rfc/RFC_USER_IDENTITY_AND_REPLY_POST_PROCESSOR.md).

**Status:** Phase 2 delivered (v0.3.0 · builtin/remote/directory post-processing · HTTP identity API · desktop/VS Code UI).

## Two side-channel modules

Both are **side-channel capability enhancement modules** (registry: [RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS_SUMMARY.md](./RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS_SUMMARY.md)) — not six host slots, not numbered facility submodules:

1. **User Identity Prompt Template** — switchable prompt fragments defining **who the user is**, merged at **`build_prompt`** (pre-LLM). Stored in the **role pack** (`user_identities/`); separate from role persona `prompts/`.
2. **Reply Post-Processor Plugin** — trait + `builtin` / `remote` / `directory` backends, invoked after built-in semantic/state consumers and **before chat append and response assembly**. Config in pack **`config.json`** (parallel to `memory`), not under `slot_registry`.

## Pipeline order

```text
ordinary Stable path: pre-LLM identity injection → applicable slot stages + LLM → clean semantic reply → state consumers/persistence → post-processor → display/transcript chat append → response
```

Agent `handled=true` returns through `minimal_response`; it does not enter this post-processing chain.

| Stage | User identity | Reply post-processor |
|-------|---------------|----------------------|
| Pre-LLM / `build_prompt` | Active identity template merged into prompt | — |
| Stable turn pipeline + LLM | — | — |
| `post/post_llm.rs` semantic/state block | — | Not run yet; state consumers use the clean semantic reply |
| Before chat append | — | `process_reply` produces the display/transcript reply; failure keeps the unmodified text |
| Chat + response | — | Chat stores display/transcript text; final **`reply`** is the display reply |

## Disambiguation

- **User identity** ≠ **role identity** (`prompts/`, `core_personality.txt`)
- **Reply Post-Processor** ≠ **distro post-process chain profile** (`distro.oclive.toml` `[post_process].chain` is policy enum; plugin is implementation)
- **Reply Post-Processor** ≠ **`dual_pipeline`** / Experimental core
- **Reply Post-Processor** ≠ Prompt slot (slot assembles prompt; post-processor edits LLM output **after** generation)

## Configuration surfaces

| Module | Pack | Distro / host |
|--------|------|---------------|
| User identity | Template catalog in `user_identities/*.md` + `index.json`; selected ids are SQLite role-runtime state | `[user_identity]` in `distro.oclive.toml` |
| Reply post-processor | `config.json` → `reply_post_processor` | `[post_process].chain`; directory `provides: ["reply_post_process"]` |

## Code anchors

- Identity: `domain/user_identity_loader.rs` · `resolve_active_user_identity` · `PromptBuilder` user-identity section
- Post-processor: `kernel/crates/oclive_kernel_host/src/domain/reply_post_processor.rs`
- Turn pipeline: `kernel/crates/oclive_kernel_host/src/domain/chat_engine/turn_pipeline/post/post_llm.rs`

## Related

- Side-channel registry: [RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS_SUMMARY.md](./RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS_SUMMARY.md)
- Post-process chain (draft): [RFC_OCLIVE_POST_PROCESS_CHAIN_SUMMARY.md](./RFC_OCLIVE_POST_PROCESS_CHAIN_SUMMARY.md)
- Pack spec: [ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md)
