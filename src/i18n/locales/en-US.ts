import apiErrors from "./fragments/apiErrors.en";
import chat from "./fragments/chat.en";
import devTools from "./fragments/devTools.en";
import emotionUi from "./fragments/emotionUi.en";
import pluginWorkbench from "./fragments/pluginWorkbench.en";
import virtualTime from "./fragments/virtualTime.en";

export default {
  apiErrors,
  chat,
  devTools,
  emotionUi,
  pluginWorkbench,
  virtualTime,
  app: {
    locale: {
      label: "Language",
      system: "Match system",
      zhCN: "中文",
      enUS: "English",
    },
    connectivity: {
      pluginIndexOffline: "The community plugin index could not be refreshed online; using local cache.",
      dismiss: "Dismiss",
    },
    theme: {
      system: "Match system",
      light: "Light",
      dark: "Dark",
    },
    defaultRoleName: "Mumu",
    /** Browser tab title (keep in sync with index.html inline bootstrap) */
    documentTitle: "OCLIVE — Desktop AI companion",
    more: {
      collapse: "Collapse",
      more: "More",
      ariaMoreFeatures: "More options",
      interactionMode: "Interaction mode",
      interactionImmersiveHint:
        "Immersive: virtual time, narrative scenes, schedule hints, and travel-related features.",
      interactionPureChatHint:
        "Chat-only: conversation only; hides scene/time bars for casual chat.",
      interactionImmersive: "Immersive",
      interactionPureChat: "Chat-only",
      identity: "Identity",
      identityHelp:
        "How you relate to the character (friend, partner, etc.); affects dialogue and stats. Different from the pack’s core personality file (core_personality.txt).",
      ui: "Appearance",
      uiHint1:
        "A− / A+ font steps are shared with the pack editor and launcher and are saved locally.",
      uiHint2: "Light / dark / system theme is also remembered.",
      appearanceToolbar: "Appearance & text size",
      scaleGroup: "UI scale",
      shrinkTitle: "Smaller",
      shrinkAria: "Decrease UI scale",
      scaleRelativeTitle: "Relative to default: {label}",
      enlargeTitle: "Larger",
      enlargeAria: "Increase UI scale",
      themeTitle: "Theme: {label} (click to cycle)",
      settingsEntry: "Settings",
      shortcutHelp: "Shortcuts",
      openSettings: "⚙ Settings",
      debug: "Debug",
      debugHelp:
        "Developer tools: favorability, memory, policy reload, etc. Ctrl+Shift+D toggles this panel; Esc closes “More” first when it is open.",
      openDebugPanel: "Open debug panel",
      virtualTime: "Virtual time",
      virtualTimeHint1:
        "In-story time, independent from the real clock. Click the time to open the wheel.",
      virtualTimeHint2:
        "Use quick buttons to advance; some packs trigger scenes or monologues after jumps.",
      narrativeScene: "Narrative scene",
      narrativeSceneHelp:
        "The scene you are narrating; matches pack scene config. Switching may fold chat history.",
      characterAt: "Character at: {label}",
      pluginBtnV1: "Plugins & backends (V1)",
      pluginBtnV2: "Plugin manager (V2)",
      settingsTileHelpV2:
        "Shortcuts, settings, and plugin manager in one place. Ctrl+Shift+S opens settings; " +
        "Ctrl+Shift+F and the button below open V2 preview; turn off “V2 preview” in settings to restore V1. Ctrl+Shift+D toggles the debug panel.",
      settingsTileHelpV1:
        "Shortcuts, settings, and plugin/back-end management together. Ctrl+Shift+S opens settings; " +
        "Ctrl+Shift+F opens advanced mode (V1) including developer debugging. Ctrl+Shift+D toggles the debug panel.",
    },
    toast: {
      remoteLifeOn: "Remote inner voice enabled",
      remoteLifeOff: "Remote inner voice disabled",
      interactionImmersive: "Switched to immersive mode",
      interactionPureChat: "Switched to chat-only mode",
      layoutResetOk: "Restored the pack’s recommended layout.",
      layoutResetFailPrefix: "Restore failed: ",
      noRolesScanned:
        "No usable role packs found (roles folder empty or validation failed). Check the roles path; for dev, set OCLIVE_ROLES_DIR to the repo roles folder.",
      fallbackReply: "Fallback reply (no body from the model)",
      roleSwitched: "Switched role: {id}",
      relationSetPerScene: "Scene identity set: {name}",
      relationSetGlobal: "Identity set: {name}",
      pluginInstalledFromWeb: "Installed plugin from web link: {id}",
      pluginFilesChanged: "Plugin files changed; refreshed",
    },
    sceneTransition: {
      going: "Going to “{label}”…",
    },
    sidebar: {
    favorability: "Favorability",
    lifeNow: "Now: {label}",
    scheduleInference: "Schedule hint",
    },
    floatingSlot: "Floating plugin slot",
    narrativeAside: {
      aria: "Narrative & inner monologue",
      title: "Narrative & inner monologue",
    },
    scene: {
      selectDestinationFirst: "Pick a destination first",
      toastTogether: "Arrived (together)",
      toastNarrativeOnly: "Narrative scene updated (character did not move)",
      systemLine:
        "Narrative scene is now “{narrative}”; character remains at “{character}”.",
    },
    hotkeyHost: {
      pluginDialogAria: "Plugin quick window",
      notFoundDialogAria: "Plugin not found",
      cannotOpenTitle: "Cannot open plugin page",
      notFoundBody:
        "The current role bootstrap has no UI for {plugin} in slot {slot}. Enable the plugin, unhide that slot, and save plugin settings.",
      launcherDialogAria: "Launchable plugins",
      launcherTitle: "Plugin catalog",
      noPlugins: "No directory plugins scanned.",
    },
    helpHintAria: "View help",
    roleSelector: {
      role: "🎭 Role",
      identity: "👤 Identity",
    },
  },
  pluginTerms: {
    module: {
      llm: "Dialogue brain (LLM)",
      emotion: "Emotion engine",
      complex_emotion: "Complex emotion",
    },
    category: {
      all: "All features",
      module: "By module",
      type: "By implementation",
      status: "By status",
    },
    type: {
      builtin: "Built-in",
      remote: "Remote",
      directory: "Directory plugin",
    },
    status: {
      enabled: "Enabled",
      disabled: "Disabled",
      needs_config: "Needs setup",
    },
    backend: {
      follow_default: "Follow role pack default",
      ollama: "Ollama (local model)",
      remote: "Remote service",
      directory: "Directory plugin",
      builtin: "Built-in",
      builtin_v2: "Built-in V2",
    },
    field: {
      backend: "Backend mode",
      directory_plugin: "Directory plugin ID",
      remote_life: "Remote inner voice",
    },
    hint: {
      directory_id_empty: "Clearing the field removes the session override and returns to the pack default.",
      endpoint_env: "Prefer environment variables for endpoints to ease migration and troubleshooting.",
    },
    action: {
      apply: "Apply changes",
      open_v1: "Open advanced mode (V1)",
      close: "Close",
    },
    title: { v2: "Plugins & backends (simple mode)" },
    subtitle: {
      v2:
        "For daily use: plain-language cards, filters, and templated configuration. Use advanced mode (V1) → UI plugins for directory-plugin developer debugging.",
    },
  },
  pluginManager: {
    legend: {
      enabled: "Enabled: this path can take effect now",
      pending: "Needs setup: usually missing a directory plugin ID",
      disabled: "Disabled: this path is not active",
    },
    source: {
      session_override: "Session override",
      env_override: "Environment override",
      pack_default: "Role pack default",
    },
    risk: {
      needsConfig: "Needs setup",
      envFirst: "Env wins",
    },
    nav: {
      explorerAria: "Filters (workspace style)",
      title: "Explorer",
      subtitle: "Filter view",
      rootTooltip: "UI hierarchy only; not a disk path",
      treeAria: "Filter tree",
      byModule: "By module",
      byBackend: "By backend",
      byStatus: "By status",
    },
    search: {
      placeholder: "Search: e.g. remote, emotion, directory",
      empty: "No matches—try a shorter keyword.",
    },
    detail: {
      readonlyNotice:
        "Read-only: nothing is saved here; change environment variables or the role pack, then reload the app.",
      previewNotice:
        "Preview: “Apply changes” writes to the current session only (does not edit pack settings.json; env wins on conflicts).",
      expand: "Expand",
      collapse: "Collapse",
      placeholder: "Pick a card from the list in the middle.",
    },
    env: { label: "Environment variables" },
    cards: {
      optionPackDefault: "Follow pack default ({backend})",
      llmMain: {
        title: "Reply engine",
        description: "Choose whether replies come from local models, a remote service, or a directory plugin.",
      },
      llmEndpoint: {
        title: "LLM remote endpoints",
        description: "When remote is selected, the LLM-specific URL is preferred.",
        summary: "Configure URLs in environment variables rather than hard-coding them in the pack.",
        fieldLlmUrl: "Dedicated LLM remote URL (preferred)",
        fieldPluginUrl: "Generic remote URL (fallback)",
      },
      emotionMain: {
        title: "Emotion engine",
        description: "Choose built-in logic, remote service, or a directory plugin for user emotion analysis.",
      },
      emotionEndpoint: {
        title: "Emotion remote endpoint",
        description: "Remote emotion calls use the generic remote plugin URL by default.",
        summary: "Put endpoints in environment variables instead of baking them into the pack.",
        fieldPluginUrl: "Common remote entry for emotion",
      },
      complexSwitch: {
        title: "Complex emotion toggle",
        description: "Turns on the remote “inner voice” path for richer complex emotion.",
        sessionOn: "On for this session",
        sessionOff: "Off for this session",
        label: "Enable complex emotion (remote inner voice)",
        hint: "When enabled, configure URL and TOKEN environment variables.",
      },
      complexEndpoint: {
        title: "Complex emotion endpoints",
        description: "Services are often deployed separately and may require a token.",
        summary: "If auth is required, set both URL and TOKEN.",
        fieldUrl: "Complex emotion service URL",
        fieldToken: "Complex emotion auth token",
      },
    },
    apply: {
      endpointNoSave: "Read-only card—configure environment variables instead.",
      remoteLifeUpdated: "Complex emotion toggle updated.",
      unsupported: "Only LLM / Emotion session writes are supported right now.",
      sessionSaved: "Saved to the current session.",
    },
    installed: {
      privateSettings: "Plugin private settings",
      debugWorkbench: "Debug console",
    },
    legendAria: "Status legend",
    v1ListItem: {
      aria: "Plugin {id}",
      kindShell: "Shell",
      kindSlot: "Slots",
      uiSlots: "UI slots: {list}",
      depsUnmet: "Dependencies not satisfied ({status}): {issues}",
      disablePlugin: "Disable plugin",
      hideToolbarEmbed: "Hide toolbar embed",
      hideSettingsEmbed: "Hide settings embed",
      hideRoleDetailEmbed: "Hide role detail embed",
      hideSidebarEmbed: "Hide sidebar embed",
      hideChatHeaderEmbed: "Hide chat header embed",
    },
    v1Backend: {
      leadBefore: "Below is the pack default and session override for ",
      leadAfter: "; changes here are not written to the role pack on disk.",
      leadPath: "settings.json → plugin_backends",
      moduleLine:
        "Module backends: mem {mem} · emotion {emotion} · event {event} · prompt {prompt} · llm {llm} · agent {agent}",
      sessionEffectiveLine:
        "Session effective: mem {mem} · emotion {emotion} · event {event} · prompt {prompt} · llm {llm} · agent {agent}",
      sessionOverrideHint:
        "Session module overrides are enabled (this session only; not written to the pack).",
      sourcesLine:
        "Sources: mem {mem} · emotion {emotion} · event {event} · prompt {prompt} · llm {llm} · agent {agent}",
      titleModule: "settings.json → plugin_backends",
      titleSession: "Session effective",
      titleSources: "Sources",
      remoteLife: "Remote inner voice",
      packDefaultSuggestOn: "Pack default suggests on",
      followPackDefault: "Follow pack default ({value})",
      localMemPlaceholder: "provider_id; empty clears this session override",
      applySession: "Apply to this session",
      debugSnapshot: "Debug snapshot",
      refresh: "Refresh",
      copy: "Copy",
      packPlugin: "Pack plugin",
      oneClickPack: "One-click pack (agent/llm)",
      copyOk: "Copied",
      copyFail: "Copy failed",
      packNeedTarget: "Configure a directory plugin in the agent or llm slot first.",
      packDone: "Pack complete: {path} (sha256={sha}…)",
      directoryPluginsPack: "Pack · directory_plugins: {summary}",
      directoryPluginsEffective: "Effective · directory_plugins: {summary}",
    },
    slotsAria: {
      settingsPanelTablist: "Plugin settings tabs",
      settingsPanelEmpty: "No plugins declare the {slot} slot yet.",
      sidebarList: "Sidebar plugin slots",
      roleDetailList: "Role detail plugin slots",
      chatHeaderStrip: "Chat header plugin slots",
      chatToolbar: "Plugin toolbar",
    },
    v2PanelAria: "Plugins & backends (V2)",
    pmSlot: {
      appearance: "Appearance",
      defaultVariant: "Default",
      hideSlot: "Hide this slot",
    },
    template: {
      directoryIdPh: "e.g. my-plugin-id",
      directoryManualPick: "Type manually or pick a candidate",
    },
    slotEmbed: {
      defaultAria: "Plugin embed area",
    },
    errorPlaceholder: {
      retry: "Load failed — tap to retry",
      fallback: "Use HTML version",
      viewDetails: "View details",
    },
    privateSettings: {
      loading: "Loading settings…",
      noFields: "This plugin does not declare uiSchema.fields.",
      templatePrefix: "Template:",
      saving: "Saving…",
      save: "Save private settings",
      toastSaved: "Private plugin settings saved.",
    },
  },
  relation: {
    defaultOptionName: "Default identity ({label})",
  },
  settings: {
    ariaDialog: "Settings",
    ariaNav: "Settings sections",
    title: "Settings",
    closeAria: "Close",
    tabGeneral: "General",
    tabPlugins: "Plugins",
    generalLeadHtml:
      "The top bar <strong>“More”</strong> groups entry points; <strong>Ctrl+Shift+S</strong> opens settings; " +
      "<strong>Ctrl+Shift+F</strong> opens the plugin manager (without “V2 preview” below it is <strong>advanced mode (V1)</strong>; " +
      "with it, the same shortcut opens <strong>V2 preview</strong>, and V1 is reachable inside V2).",
    shortcutsLabel: "Shortcuts",
    shortcutsHelp:
      "Ctrl+Shift+S opens settings; Ctrl+Shift+F opens the plugin manager (V1/V2 per the experimental toggle); Ctrl+Shift+D toggles the debug panel.",
    immersiveOnlyNote:
      "Virtual time and narrative scenes appear under “More” only in immersive mode.",
    envCheckTitle: "Environment check",
    envCheckHelp:
      "Quick probe: Ollama reachability, roles root readability, app data dir writable; not a full startup health pass.",
    envCheckLead:
      "If chat or the model fails, run this first; see ERROR_CODES.md §1.5 for detailed codes.",
    envCheckRun: "Run check",
    envCheckRunning: "Checking…",
    envCheckDoneToast: "Environment check finished.",
    envCheckOllama: "Ollama ({url})",
    envCheckOllamaOk: "reachable",
    envCheckOllamaFail: "unreachable or error",
    envCheckRoles: "Roles root",
    envCheckRolesMissing: "missing",
    envCheckRolesUnreadable: "exists but not readable",
    envCheckRolesOk: "readable",
    envCheckRolesHint:
      "Path from OCLIVE_ROLES_DIR or default; must be the parent of per-role folders, each with manifest.json.",
    envCheckAppData: "App data directory",
    envCheckAppDataOk: "writable",
    envCheckAppDataFail: "not writable",
    envCheckDetail: "Detail:",
    sentrySectionTitle: "Crash diagnostics (Sentry)",
    sentrySectionLead:
      "Shown only when this build ships with a DSN. Reports uncaught Vue errors (not chat text); Rust still relies mainly on local logs.",
    sentryOptOutLabel: "Disable crash reporting",
    sentryOptOutHelp:
      "When checked, the Sentry client is closed immediately; preference is stored in localStorage (key oclive.telemetry.sentryOptOut). Uncheck and restart the app to re-enable reporting.",
    sentryDisabledToast: "Crash reporting disabled.",
    sentryReenableRestartToast: "Opt-out cleared; restart the app to resume reporting.",
    experimentalLabel: "Experimental",
    experimentalSectionHelp:
      "Preview entry for the new plugin manager (V2). If this build has no V2, the existing advanced mode is used.",
    experimentalToggleTitle: "Enable new plugin manager (V2 preview)",
    experimentalToggleHtml:
      "When on, <strong>Ctrl+Shift+F</strong> and the plugin entry in <strong>“More”</strong> open <strong>V2 preview</strong> (remembered in settings). " +
      "For full developer debugging, open <strong>advanced mode (V1)</strong> inside V2; you can turn this off to restore defaults.",
    openV2Preview: "Open plugin manager V2 preview",
    remoteFallbackSectionTitle: "Remote plugin failure policy",
    remoteFallbackLabel: "Fall back to built-in when remote HTTP fails",
    remoteFallbackHelp:
      "When off, slots configured as remote (memory / emotion / event / prompt / LLM sidecars) return REMOTE_SERVICE_UNAVAILABLE if the sidecar is unreachable, instead of silently using built-in implementations. This complements high-risk network grants: grants gate whether outbound calls are allowed; this switch gates whether failures may degrade to built-in. The environment variable <code>OCLIVE_REMOTE_FALLBACK_TO_BUILTIN</code> overrides the effective in-process value (when set, this toggle is locked for the running process).",
    remoteFallbackEnvLocked:
      "An environment variable is set; the running process uses it. The database value can still be saved for sessions without the variable.",
    remoteFallbackSavedToast: "Saved.",
    advancedTitle: "Advanced area (settings.advanced)",
    advancedDesc:
      "Plugins that declare <code>settings.advanced</code> in the manifest render here.",
    advancedSlotAria: "Settings advanced slot",
    securityLabel: "Security",
    forceIframeTitle: "Force iframe mode",
    forceIframeDesc:
      "When on, all plugin UIs load in iframes (safer, may reduce quality). Restart the app for full effect after saving.",
    pluginsPanelTitle: "Directory plugins · settings slots",
    pluginsPanelHint1:
      "Declare <code>settings.panel</code> in the plugin manifest ui_slots to embed a settings page here.",
    pluginsPanelHint2:
      "Same loading rules as chat_toolbar: https://ocliveplugin.localhost/<id>/<entry>; order/hide in plugin manager.",
    iframeSavedInfo: "Saved. Restart the app for forced iframe to apply fully.",
  },
  common: {
    cancel: "Cancel",
    close: "Close",
    loading: "Loading…",
    preparing: "Preparing…",
    importPackTitle: "Importing role pack",
    chatInputLabel: "Message",
    chatPlaceholder: "Say something to {name}…",
    send: "Send",
    sceneTravel: {
      togetherAria: "Together travel — pick destination",
      togetherLabel: "Together travel detected — pick a destination",
      postAria: "Pick scene to switch",
      postLabel: "Travel intent detected — pick a destination",
      pickPlaceholder: "Choose destination",
      solo: "Go alone",
      together: "Go together",
      dismiss: "Not now",
    },
    sceneMode: {
      title: "Go to “{label}”",
      desc: "Switch only your narrative view, or go together?",
      solo: "I go alone (character stays)",
      together: "Go together",
    },
    autonomousNotice:
      "System: after virtual time changed, the character’s scene moved from “{from}” to “{to}” (your narrative view did not auto-follow).",
    shortcutHelp: {
      aria: "Shortcuts",
      title: "Shortcuts",
      rowOpenSettings: "Open settings (advanced area, security, hotkeys, plugin config)",
      rowCtrlLong: "Open this shortcuts dialog",
      rowCtrlLongKeys: "Ctrl (hold ~1s)",
      foot: "More shortcuts will be added over time.",
      slotSectionAria: "Launcher slot",
      slotHeading: "Plugin slot (launcher.palette)",
      slotEmbedAria: "Launcher slot",
      ctrlShiftFV2:
        "Open plugin manager (V2 preview); turn off “V2 preview” in settings to use V1",
      ctrlShiftFV1: "Open advanced mode (V1) plugins & backends (incl. dev debug)",
    },
    rolePack: {
      exportFilterName: "OCPak role pack",
      importFilterName: "OCPak / ZIP",
      exported: "Role pack exported",
      importedOverwrite: "Imported (overwrite): {id}",
      imported: "Imported role: {name}",
      barTitle:
        "Install .ocpak / .zip or an extracted folder (same layout as roles/{id}/)",
      export: "Export pack",
      importArchive: "Import archive",
      importFolder: "Import folder",
      conflictTitle: "Role already exists",
      conflictBody:
        "Local role ID “{id}” already exists ({name} v{version}). Import will overwrite that folder. Continue?",
      overwrite: "Overwrite import",
    },
  },
  roleRuntime: {
    personalityProfile: "Profile (mutable text maintained by chat)",
    personalityVector: "7D vector",
    profileHint1:
      "Profile source: runtime uses core + mutable personality archives; the seven dimensions here are mostly a summarized view.",
    profileHint2:
      "Unlike vector mode (dimensions drive events); see docs/personality-archive-notes.md.",
    vectorHint1:
      "Vector source: events and mood adjust per dimension; matches evolution.personality_source in settings.",
    versionAuthor: "Version {version} · Author {author}",
    personalitySource: "Personality source:",
    backendHintBefore: "Module backends, remote inner voice, session overrides, and debug snapshots moved to",
    backendLink: "Plugins & backends → Backends",
    backendHintAfter: "(Ctrl+Shift+F)",
    relation: "Relation",
    eventImpact: "Event impact",
  },
  editor: {
    personalityTrait: {
      stubbornness: "Stubbornness",
      clinginess: "Clinginess",
      sensitivity: "Sensitivity",
      assertiveness: "Assertiveness",
      forgiveness: "Forgiveness",
      talkativeness: "Talkativeness",
      warmth: "Warmth",
    },
    chatExport: {
      allRoles: "Export all roles",
      pluginDebug: "Include plugin resolution debug (single role)",
      exportJson: "Export JSON",
      exportTxt: "Export TXT",
      downloaded: "Downloaded {name}",
      success: "Export saved",
      saveCancelled: "Save cancelled",
    },
    debug: {
      monologueInserted: "Monologue inserted",
      monologuePrefix: "[Monologue] ",
      title: "🎛️ Debug panel",
      hint1:
        "Inspect favorability, traits, recent events and memory; reload policy, generate monologue, import/manage packs.",
      hint2:
        "Ctrl+Shift+D toggles this panel; Esc also closes it. Under top bar “More”, use “Open debug panel”.",
      dockSlotAria: "Debug dock slot",
      insertMonoGenerating: "Generating…",
      insertMono: "Insert monologue",
      knowledgeTitle: "World knowledge",
      knowledgeIndexed: "Pack index:",
      knowledgeLoaded: "loaded",
      knowledgeNotLoaded: "not loaded",
      knowledgeChunks: " · {n} chunks",
      knowledgeLastPrompt: "Last prompt injection:",
      knowledgeChunksUnit: "chunks",
      knowledgeLastPromptLine: "Last prompt injection: {n} chunks",
      knowledgeHint:
        "Updates after you send; click “Refresh debug data” to sync chunk counts (call load_role after disk changes).",
      favorability: "Favorability",
      personalityVector: "Personality vector",
      personalityProfileHelp:
        "Profile-sourced pack: these seven values are mostly derived from archives for readability, not the sole source.",
      metaCounts: "Events: {events} · Memories: {memories}",
      recentEvents: "Recent events",
      recentMemories: "Recent memories",
      refresh: "Refresh debug data",
      reloadPolicy: "Reload policy",
      footer: "💡 Ctrl+Shift+D toggles panel · packs & monologue live here",
      fav80: "💖 Very close!",
      fav60: "💕 Going strong",
      fav40: "👍 Okay",
      fav20: "🤝 Getting to know",
      fav0: "😶 Still distant",
      presenceCoPresent: "Co-present",
      presenceRemoteStub: "Remote placeholder",
      presenceRemoteLife: "Remote inner voice",
    },
  },
  hotkeys: {
    title: "Global shortcuts",
    lead:
      "All off by default. When enabled, the OS listens globally and may conflict with other apps; save errors show a toast.",
    fieldAccelerator: "Shortcut",
    accelPlaceholder: "e.g. Ctrl+Shift+L",
    enabled: "On",
    action: "Action",
    actionOpenLauncher: "Open plugin directory list",
    actionOpenSlot: "Open a plugin slot page",
    pluginId: "Plugin id",
    slotName: "Slot name",
    appearanceOptional: "appearance (optional)",
    remove: "Remove",
    addRow: "Add row",
    save: "Save",
    savedToast: "Hotkeys saved (only enabled rows register globally).",
  },
};
