# I18N progress (oclivenewnew)

## Locale wiring

- **Library**: `vue-i18n` v9+ (`legacy: false`).
- **Entry**: `src/i18n/index.ts` — `LOCALE_PREF_KEY = "oclive.appLocale"`, `getLocalePreference` / `setLocalePreference`, `resolveLocaleTag`, `fallbackLocale: "zh-CN"`.
- **Messages**: `src/i18n/locales/zh-CN.ts`, `en-US.ts` (+ `locales/fragments/*` for large trees).

## Phase 0 — CJK inventory (Vue/TS under `src/`)

Generated with: `rg -l "\\p{Han}" --glob "*.vue" --glob "*.ts" src` (Unicode class `Han`).

Hotspots (user-visible; comments / locale files / tests may still match):

- Views: `PluginManagerPanel.vue`, `PluginManagerV2Panel.vue`, `SettingsView.vue`
- Chat / time: `ChatMessageList.vue`, `ChatMessage.vue`, `VirtualTimeBar.vue`, `TimeDial.vue`
- Dev / plugin: `AgentDebugPanel.vue`, `RpcTester.vue`, `ProcessMonitor.vue`, `PluginScaffoldWizard.vue`, `DirectoryShellApp.vue`, `EnvVarManager.vue`, `PluginDebugPanel.vue`, `HotkeyHost.vue`, …
- Infra: `utils/tauri-api.ts` (API error strings → `apiErrors.*` + `i18n.global`)

## Sister repos

- **oclive-pack-editor** / **oclive-launcher**: align `LOCALE_PREF_KEY` and persistence with this file (see their `docs/I18N_PROGRESS.md`).
- **oclive-plugin-market**: `vue-i18n@11`, same `LOCALE_PREF_KEY`, `src/i18n/index.ts` + `locales/zh-CN.ts` / `en-US.ts` (see market repo doc).

## Build / test

- `npm run build`, `npm run test:unit`
- `cargo test --workspace` (Rust user-visible errors remain code-first; UI maps by `[CODE]`).
