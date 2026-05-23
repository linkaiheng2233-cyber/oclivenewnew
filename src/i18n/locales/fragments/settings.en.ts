/** settings — en. */
export default {
  settings: {
    ariaDialog: 'Settings',
    ariaNav: 'Settings sections',
    title: 'Settings',
    closeAria: 'Close',
    tabGeneral: 'General',
    tabPlugins: 'Plugins',
    generalLeadHtml:
      'The top bar <strong>“More”</strong> groups entry points; <strong>Ctrl+Shift+S</strong> opens settings; '
      + '<strong>Ctrl+Shift+F</strong> opens the plugin manager (without “V2 preview” below it is <strong>advanced mode (V1)</strong>; '
      + 'with it, the same shortcut opens <strong>V2 preview</strong>, and V1 is reachable inside V2).',
    shortcutsLabel: 'Shortcuts',
    shortcutsHelp:
      'Ctrl+Shift+S opens settings; Ctrl+Shift+F opens the plugin manager (V1/V2 per the experimental toggle); Ctrl+Shift+D toggles the debug panel.',
    immersiveOnlyNote:
      'Virtual time and narrative scenes appear under “More” only in immersive mode.',
    envCheckTitle: 'Environment check',
    envCheckHelp:
      'Quick probe: Ollama reachability, roles root readability, app data dir writable; not a full startup health pass.',
    envCheckLead:
      'If chat or the model fails, run this first; see ERROR_CODES.md §1.5 for detailed codes.',
    envCheckRun: 'Run check',
    envCheckRunning: 'Checking…',
    envCheckDoneToast: 'Environment check finished.',
    envCheckOllama: 'Ollama ({url})',
    envCheckOllamaOk: 'reachable',
    envCheckOllamaFail: 'unreachable or error',
    envCheckRoles: 'Roles root',
    envCheckRolesMissing: 'missing',
    envCheckRolesUnreadable: 'exists but not readable',
    envCheckRolesOk: 'readable',
    envCheckRolesHint:
      'Path from OCLIVE_ROLES_DIR or default; must be the parent of per-role folders, each with manifest.json.',
    envCheckAppData: 'App data directory',
    envCheckAppDataOk: 'writable',
    envCheckAppDataFail: 'not writable',
    envCheckDetail: 'Detail:',
    envCheckOllamaPullNote:
      'Model download/pull progress is shown in the terminal (`ollama pull`). This check only probes reachability, not pull percentage.',
    sentrySectionTitle: 'Crash diagnostics (Sentry)',
    sentrySectionLead:
      'Shown only when this build ships with a DSN. Reports uncaught Vue errors (not chat text); Rust still relies mainly on local logs.',
    sentryOptOutLabel: 'Disable crash reporting',
    sentryOptOutHelp:
      'When checked, the Sentry client is closed immediately; preference is stored in localStorage (key oclive.telemetry.sentryOptOut). Uncheck and restart the app to re-enable reporting.',
    sentryDisabledToast: 'Crash reporting disabled.',
    sentryReenableRestartToast: 'Opt-out cleared; restart the app to resume reporting.',
    pluginCliLabel: 'Advanced plugin configuration',
    pluginCliHelp: 'Architecture graph, multi-slot blueprint, and disk writes are not in this UI.',
    pluginCliNote:
      'Use <code>oclive plugin manage</code> (optional <code>--tui</code>) for slot_registry; install with manifest <code>slot_attachment</code> for auto wiring. See creator docs.',
    remoteFallbackSectionTitle: 'Remote plugin failure policy',
    remoteFallbackLabel: 'Fall back to built-in when remote HTTP fails',
    remoteFallbackHelp:
      'When off, slots configured as remote (memory / emotion / event / prompt / LLM sidecars) return REMOTE_SERVICE_UNAVAILABLE if the sidecar is unreachable, instead of silently using built-in implementations. This complements high-risk network grants: grants gate whether outbound calls are allowed; this switch gates whether failures may degrade to built-in. The environment variable <code>OCLIVE_REMOTE_FALLBACK_TO_BUILTIN</code> overrides the effective in-process value (when set, this toggle is locked for the running process).',
    remoteFallbackEnvLocked:
      'An environment variable is set; the running process uses it. The database value can still be saved for sessions without the variable.',
    remoteFallbackSavedToast: 'Saved.',
    advancedTitle: 'Advanced area (settings.advanced)',
    advancedDesc:
      'Plugins that declare <code>settings.advanced</code> in the manifest render here.',
    advancedSlotAria: 'Settings advanced slot',
    securityLabel: 'Security',
    forceIframeTitle: 'Force iframe mode',
    forceIframeDesc:
      'When on, all plugin UIs load in iframes (safer, may reduce quality). Restart the app for full effect after saving.',
    pluginsPanelTitle: 'Directory plugins · settings slots',
    pluginsPanelHint1:
      'Declare <code>settings.panel</code> in the plugin manifest ui_slots to embed a settings page here.',
    pluginsPanelHint2:
      'Same loading rules as chat_toolbar: https://ocliveplugin.localhost/<id>/<entry>; order/hide in plugin manager.',
    iframeSavedInfo: 'Saved. Restart the app for forced iframe to apply fully.',
  },
  hotkeys: {
    title: 'Global shortcuts',
    lead:
      'All off by default. When enabled, the OS listens globally and may conflict with other apps; save errors show a toast.',
    fieldAccelerator: 'Shortcut',
    accelPlaceholder: 'e.g. Ctrl+Shift+L',
    enabled: 'On',
    action: 'Action',
    actionOpenLauncher: 'Open plugin directory list',
    actionOpenSlot: 'Open a plugin slot page',
    pluginId: 'Plugin id',
    slotName: 'Slot name',
    appearanceOptional: 'appearance (optional)',
    remove: 'Remove',
    addRow: 'Add row',
    save: 'Save',
    savedToast: 'Hotkeys saved (only enabled rows register globally).',
  }
}
