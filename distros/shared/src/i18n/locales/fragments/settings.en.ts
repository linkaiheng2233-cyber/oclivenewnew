/** settings — en. */
export default {
  settings: {
    ariaDialog: 'Settings',
    ariaNav: 'Settings sections',
    title: 'Settings',
    closeAria: 'Close',
    tabGeneral: 'General',
    tabVoice: 'Voice',
    tabPlugins: 'Plugins',
    tabStorage: 'Storage',
    voicePanelLead:
      'Speech recognition (hold-to-talk), submit behavior, and optional emotional TTS expansion with model packs.',
    generalLeadHtml:
      'Use the Activity Bar or <strong>Ctrl+Shift+S / M</strong> for settings and models; '
      + 'appearance and locale live under Settings → General; plugins are Story-mode only.',
    appearanceSectionTitle: 'Appearance',
    appearanceSectionHelp: 'Locale, theme, and UI scale; stored on this device.',
    skinWin98Label: 'Windows 98 skin (easter egg)',
    skinWin98Help: 'Classic gray chrome and pixel scrollbars.',
    interactionModeSectionTitle: 'Interaction mode',
    interactionModeSectionLead:
      'Switch daily chat vs story mode here. Daily chat stays on the simple home scene; scenes, travel, and plugins unlock in story mode.',
    interactionModeFieldLabel: 'Mode',
    interactionModePersistNote: 'Your choice is remembered for the next launch.',
    interactionModeLockedNote: 'Mode switching is disabled for this distro profile.',
    immersiveOnlyNote:
      'Virtual time and narrative scenes appear under “More” only in Story mode.',
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
    forceIframeTitle: 'Plugin UI isolation (enforced in releases)',
    forceIframeDesc:
      'Release builds always use constrained HTML/iframes and never execute plugin Vue in the host page. Disabling this requires both a Vite dev build and VITE_OCLIVE_UNSAFE_INLINE_PLUGIN_VUE=1; that mode inherits host-window authority.',
    pluginsPanelTitle: 'Directory plugins · settings slots',
    pluginsPanelHint1:
      'Declare <code>settings.panel</code> in the plugin manifest ui_slots to embed a settings page here.',
    pluginsPanelHint2:
      'Same loading rules as chat_toolbar: https://ocliveplugin.localhost/<id>/<entry>; order/hide in plugin manager.',
    iframeSavedInfo: 'Saved. Restart the app for forced iframe to apply fully.',
    userIdentitySectionTitle: 'User identity',
    userIdentitySectionLead:
      'Choose who you are in the story; the character will treat you accordingly on the next turn.',
    userIdentitySectionLeadSecondary:
      'Different from the top-bar Relation: relation affects favor and distance; if an identity maps to a relation, switching identity syncs relation automatically.',
    postProcessorSectionTitle: 'Reply post-processor',
    chatStreamSectionTitle: 'Chat',
    chatStreamSectionHelp:
      'Streaming shows tokens as they arrive; when off, each turn waits for the full blocking reply.',
    chatStreamEnabledLabel: 'Streaming replies',
    chatStreamEnabledHint: 'Enable SSE token-by-token display (default on)',
    noIdentityCatalogHint:
      'This role pack has no selectable identities; reply post-processor status is still shown below.',
    layoutSectionTitle: 'Layout',
    layoutSectionHelp:
      'Drag the divider on the role rail or settings/plugins/models side panel to resize; preferences are stored locally.',
    layoutSectionLead: 'Left rail and side panel widths are separate from the role pack ui.json layout keys.',
    layoutCurrentWidths: 'Current: left rail {left}px · side panel {side}px',
    layoutResetWidths: 'Reset panel widths',
    layoutResetWidthsDone: 'Default panel widths restored.',
    layoutResizeLeftRail: 'Resize role sidebar',
    layoutResizeSidePanel: 'Resize side panel',
    advancedFoldTitle: 'Advanced & diagnostics',
  },
  keybindings: {
    title: 'Keybindings',
    lead:
      'Bind actions like a game: in-app keybindings require window focus; global shortcuts are OS-level listeners (off by default) and may conflict with other apps.',
    actions: {
      openSettings: 'Open settings',
      openPluginManager: 'Open plugin manager (immersive)',
      openModelManager: 'Open model manager',
      toggleDebug: 'Toggle debug panel (immersive)',
      openShortcutHelp: 'Open shortcut help',
      holdToTalk: 'Hold to talk',
      openPluginLauncher: 'Open plugin launcher list (global)',
      openPluginSlot: 'Open a plugin slot (global)',
    },
    immersiveOnly: 'Immersive only',
    current: 'Current',
    capture: 'Capture',
    captureBtn: 'Capture key',
    capturing: 'Press keys…',
    captureHint: 'Press a new key combo; press Esc to clear.',
    clear: 'Clear',
    enabled: 'On',
    resetDefaults: 'Reset defaults',
    save: 'Save',
    savedToast: 'Keybindings saved.',
    conflictToast: 'Conflicting keybindings detected. Fix conflicts before saving.',
    conflictInline: 'Conflicts detected: multiple actions share the same keybinding within a scope.',
    holdInvalidToast: 'Hold bindings only support a single key (e.g. V).',
    useCtrlLongPress: 'Use Ctrl long-press',
    globalTitle: 'Global shortcuts (plugins)',
    globalLead:
      'OS-level global listeners. Off by default; may conflict with other apps.',
    globalAction: 'Action',
    globalAccelerator: 'Shortcut',
    accelPlaceholder: 'e.g. Ctrl+Shift+L',
    addGlobalLauncher: 'Add: open plugin launcher list',
    addGlobalSlot: 'Add: open plugin slot',
    pluginId: 'Plugin id',
    slotName: 'Slot',
    appearanceOptional: 'appearance (optional)',
    movedNotice: 'Plugin global shortcuts moved to: Settings → General → Advanced → Keybindings.',
    goToGeneralAdvanced: 'Go to General → Advanced',
  },
}
