# User Identity & Reply Post-Processor — Phase 2

**Status**: **closed** (Phase 2 delivered + v0.3.0 closure 2026-06-07).  
**RFC**: [creator-docs/rfc/RFC_USER_IDENTITY_AND_REPLY_POST_PROCESSOR.md](../creator-docs/rfc/RFC_USER_IDENTITY_AND_REPLY_POST_PROCESSOR.md)

## Delivered in Phase 2

| Item | Surface | Notes |
|------|---------|-------|
| **HostProfile merge** | `host_profile.rs` · `distro.oclive.toml` | `[user_identity].default_id` / `allowed_ids`; `[post_process].chain` → effective builtin profile |
| **Remote Reply Post-Processor** | `reply_post_process_http.rs` | JSON-RPC `reply_post_process.process`; fallback to builtin |
| **Directory Reply Post-Processor** | `reply_post_process_directory_http.rs` · `provides: reply_post_process` | Example: `examples/directory-plugin-reply-post-process-minimal/` |
| **HTTP identity API** | `http_api.rs` | `POST /user_identity/set`, `POST /user_identity/scene_set`, `GET /user_identity/state` |
| **Desktop identity switcher** | `RoleRuntimePanel.vue` · `distros/shared/src/api/role.ts` | When `user_identities/` catalog non-empty |
| **VS Code identity switcher** | `oclive-vscode` · `kernelClient` + command `oclive.selectUserIdentity` | Depends on HTTP routes |
| **`raw_reply` DTO** | `SendMessageRequest.include_raw_reply` · `SendMessageResponse.raw_reply` | `schema` **14**; opt-in only |
| **Validation** | `oclive_validation` | `backend=remote` + `enabled` requires non-empty `remote.url` |
| **Desktop UI post-processor status** | `RoleRuntimePanel.vue` · `RoleInfo.reply_post_processor_*` | Read-only; `GET /role_info` for VS Code |
| **VS Code status bar context** | `statusBar.ts` · `kernelClient.fetchRoleInfo` | Identity + post-process from HTTP |

## v0.3 closure (2026-06-07)

- Documentation: ROLE_PACK_SPEC §1.1 / §9.7, OCLIVE_ARCHITECTURE_OVERVIEW, AGENTS, USER_MANUAL, RFC §8
- `distros/chat-pro/roles/mumu` keeps **no** `reply_post_processor` (identity demo only)
- Desktop spawn passes `OCLIVE_DISTRO_ID=desktop` when profile unset
- Release **0.3.0** (desktop + VS Code extension)

## v0.3 baseline (unchanged)

- `user_identities/` load + `resolve_active_user_identity` + `build_prompt` injection
- `config.json` → `reply_post_processor` with **builtin** backend
- Tauri: `set_user_identity`, `set_scene_user_identity`, `get_user_identity_state`

## Deferred (Phase 2.1 / studio)

| Item | Notes |
|------|-------|
| **pack-editor UI** | `oclive-pack-editor` — `user_identities/` editor; `reply_post_processor` config panel |
| **OOCP S15** | Optional remote post-processor HTTP scenario |

## Tests

- `distros/desktop-tauri/tests/reply_post_processor_roundtrip.rs` — builtin truncate
- `distros/desktop-tauri/tests/reply_post_processor_remote_roundtrip.rs` — mock HTTP + fallback
- `distros/desktop-tauri/tests/user_identity_host_profile.rs` — distro default identity + chain merge
- `kernel/crates/oclive_kernel_host/src/domain/host_profile.rs` — TOML parse
- `kernel/crates/oclive_kernel_host/src/domain/reply_post_processor.rs` — effective config merge
