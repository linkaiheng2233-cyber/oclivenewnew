# Surface pack · Chat Pro frontend (EN summary)

> Full checklist (ZH): [`human-docs/modules/surfaces/frontend-chat-pro.md`](../../../human-docs/modules/surfaces/frontend-chat-pro.md)
> Invoke contract: [tauri-invoke](tauri-invoke.md)

**You plug in**: `distros/chat-pro/` + `distros/shared/` · send chain `distros/shared/src/api/chat.ts` → `send_message` · state `distros/shared/src/stores/chatStore.ts` · **not** `process_message.rs`.

**Default interface**: `App.vue` maps the `fluent` identifier to `RoleWorkspaceShell.vue`, rather than legacy `FluentShell.vue`. Tool is an explicit alternative. [MODULE_MAP §13.1](../../../handoff/MODULE_MAP_AND_HANDOFF.md#131-chat-pro-壳与互动模式-ia) owns responsibilities and deferred entries.

**Do**: Vue components · Pinia stores · styles · `distros/shared/src/api/` camelCase wrappers · plugin/model manager panels.

**Don't**: Second LLM call in UI (portrait / chat bypassing slots) · use `response` instead of DTO **`reply`** · stack business logic in `lib.rs`.

**Read next**: [paths/frontend](../../paths/frontend.md) · [tauri-invoke](tauri-invoke.md) · [facilities/visual-stage](../facilities/visual-stage.md) · [NAMING §8](../../../creator-docs/NAMING_CONVENTIONS.md).

**Internationalization**: [the shared i18n entry](../../../distros/shared/src/i18n/index.ts) uses `legacy: false`, the `oclive.appLocale` preference key, and `zh-CN` fallback. Read preferences with `getLocalePreference()`, resolve `system` with `resolveLocaleTag()`, and update with `setLocalePreference()`. Maintain shared `zh-CN` / `en-US` messages and fragments together; run `npm run verify:shared-i18n` and the relevant frontend tests or build. Historical CJK inventories do not establish current translation coverage; check sister repositories independently.
