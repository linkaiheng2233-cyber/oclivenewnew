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
- Dev / plugin: `RpcTester.vue`, `ProcessMonitor.vue`, `DirectoryShellApp.vue`, `EnvVarManager.vue`, `PluginDebugPanel.vue`, `HotkeyHost.vue`, …（`AgentDebugPanel.vue` / `PluginScaffoldWizard.vue` 已移除）
- Infra: `utils/tauri-api.ts` (API error strings → `apiErrors.*` + `toFriendlyErrorMessage`)

**2026-05-15 核对**：上述热点路径中，**模板内用户可见串**多数已走 `t()` / `virtualTime.*` / `apiErrors`；`rg` 命中汉字仍以 **注释与样式说明** 为主。后续改文案优先搜不带 `t(` 的引号串。

## Sister repos

- **oclive-pack-editor** / **oclive-launcher**: align `LOCALE_PREF_KEY` and persistence with this file (see their `docs/I18N_PROGRESS.md`).
- **oclive-plugin-market**: `vue-i18n@11`, same `LOCALE_PREF_KEY`, `src/i18n/index.ts` + `locales/zh-CN.ts` / `en-US.ts` (see market repo doc).

## Build / test

- `npm run build`, `npm run test:unit`
- `cargo test --workspace` (Rust user-visible errors remain code-first; UI maps by `[CODE]`).
