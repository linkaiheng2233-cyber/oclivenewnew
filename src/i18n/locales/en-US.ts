export const enUS = {
  common: {
    save: "Save",
    close: "Close",
    cancel: "Cancel",
    confirm: "Confirm",
    continue: "Continue",
    refresh: "Refresh",
    security: "Security",
    advanced: "Advanced",
  },
  app: {
    startup: {
      loadingRoleAndPlugins: "Loading roles and plugins…",
      scanningRolePacks: "Scanning role packs…",
      loadingRoleData: "Loading role data…",
      initializingPlugins: "Initializing plugins…",
      failed: "Startup failed. Please check role and plugin configuration.",
      noRolesFound:
        "No usable roles found (roles directory is empty or all validations failed). Please check the host roles path. In dev you can set OCLIVE_ROLES_DIR to the repo roles folder.",
    },
    topBar: {
      more: {
        open: "More",
        collapse: "Collapse",
        regionLabel: "More actions",
      },
      tiles: {
        interactionMode: {
          title: "Interaction mode",
          hint: [
            "Immersive: enables virtual time, narrative scenes, schedule inference and travel capabilities.",
            "Chat-only: keeps conversation only; hides scene/time bars for casual chatting.",
          ],
          immersive: "Immersive",
          pureChat: "Chat-only",
        },
        identity: {
          title: "Identity",
          hint:
            "Relationship identity with the role (friend, lover, etc.). Affects dialogue and relationship values; different from the role pack’s core personality profile.",
        },
        appearance: {
          title: "Appearance",
          hint: [
            "A− / A+ uses the same scale steps as the editor/launcher and is stored locally.",
            "Theme can be Light / Dark / System and will be remembered.",
          ],
          toolbarLabel: "Appearance and scale",
          scaleLabel: "UI scale",
          shrink: "Shrink",
          shrinkAria: "Shrink UI",
          enlarge: "Enlarge",
          enlargeAria: "Enlarge UI",
          relativeScaleTitle: "Relative to default: {label}",
          themeTitle: "Theme: {label} (click to cycle)",
          themeSystem: "System",
          themeDark: "Dark",
          themeLight: "Light",
        },
        settingsEntry: {
          title: "Entries",
          groupLabel: "Entry actions",
          shortcutHelp: "Shortcuts",
          settings: "Settings",
          pluginMarket: "Plugin Market (Ctrl+Shift+A)",
        },
        rolePackShare: {
          title: "Role packs (share with friends)",
          hint: [
            "If you receive a .ocpak/.zip from a friend, click “Import archive” to use it directly.",
            "roles.json index is optional: it can use self-hosted/community sources and does not depend on official services.",
          ],
        },
        debug: {
          title: "Debug",
          hint:
            "For developers and troubleshooting. Ctrl+Shift+D toggles the debug window. When “More” is open, pressing Esc will close this panel first.",
          openPanel: "Open debug panel",
        },
        virtualTime: {
          title: "Virtual time",
          hint: [
            "In-story time, independent from real clock. Click time to open the wheel editor.",
            "Use quick buttons to advance time; some role packs can trigger scenes/monologues after jumps.",
          ],
        },
      },
    },
  },
  settings: {
    title: "Settings",
    sectionsNavLabel: "Settings sections",
    tabs: {
      general: "General",
      plugins: "Plugins",
    },
    language: {
      label: "Language",
      options: {
        system: "System",
        zhCN: "中文 (Simplified)",
        enUS: "English",
      },
      hint: "Takes effect immediately; “System” follows your OS language.",
    },
    shortcuts: {
      label: "Shortcuts",
      immersiveHint:
        'Virtual time, narrative scenes, etc. are only shown under "More" in immersive mode.',
    },
    experimental: {
      label: "Experimental",
    },
  },
  pluginManager: {
    entry: {
      settingsGeneralLeadHtml:
        "Plugin Manager is being iterated. Some entries are hidden by default to reduce distractions.",
      settingsShortcutsHelpHint:
        "You can open Plugin Manager quickly via the top bar “More” menu or Ctrl+Shift+F.",
      settingsExperimentalSectionHelpHint:
        "Experimental features may change frequently and can break compatibility.",
      settingsExperimentalToggleTitle: "Enable new Plugin Manager UI (V2 preview)",
      settingsExperimentalToggleDescriptionHtml:
        "This is a preview. Some features may be incomplete; feedback is welcome.",
      settingsOpenV2PreviewButtonLabel: "Open V2 preview window",
    },
    moreMenu: {
      pluginButtonLabel: {
        v1: "Oclive Manager (V1)",
        v2: "Oclive Manager (V2)",
      },
      tileHelpText: {
        v1:
          "Centralizes shortcuts, Settings, and Oclive Manager (plugins & backends). Shortcuts: Ctrl+Shift+S opens Settings; Ctrl+Shift+F opens Oclive Manager (V1, with dev debug); Ctrl+Shift+A opens Plugin Market. Ctrl+Shift+D toggles Debug panel.",
        v2:
          "Centralizes shortcuts, Settings, and Oclive Manager. Shortcuts: Ctrl+Shift+S opens Settings; Ctrl+Shift+F and the button below open Oclive Manager (V2 preview); Ctrl+Shift+A opens Plugin Market; disable “V2 preview” in Settings to return to V1. Ctrl+Shift+D toggles Debug panel.",
      },
    },
    shortcuts: {
      ctrlShiftFDescription: {
        v1: "Open Oclive Manager (V1, includes UI plugins · dev debugging).",
        v2: "Open Oclive Manager (V2 preview). Disable “V2 preview” in Settings to return to V1.",
      },
      ctrlShiftADescription:
        "Open Plugin Market (V1/V2): online index, install, modules/profiles, local imports, etc.",
    },
  },
  pluginManagerV2: {
    slots: {
      settingsPanel: "Settings page (plugin settings)",
      settingsPlugins: "Plugin Manager embedded area",
      settingsAdvanced: "Settings page (advanced area)",
      sidebar: "Left sidebar",
      roleDetail: "Role detail",
      chatHeader: "Chat header",
      chatToolbar: "Chat toolbar",
      overlayFloating: "Floating overlay",
      launcherPalette: "Launcher (quick entry)",
      debugDock: "Debug panel",
    },
    permissions: {
      risk: {
        high: "High risk",
        medium: "Medium risk",
        low: "Low risk",
        unknown: "Unknown",
      },
      toastUpdated: "Permission updated.",
      toastNoDeclared: "This plugin declares no permissions.",
      confirmGrantAll:
        "Grant all declared permissions to this plugin (total {n}).\n\nTip: only grant permissions to plugins you trust.\n\nContinue?",
      toastGrantedAll: "All declared permissions granted.",
      toastNoMissing: "No missing permissions for this plugin.",
      confirmGrantMissing:
        "Grant missing declared permissions (total {n}):\n\n{list}\n\nContinue?",
      toastGrantedMissing: "Missing permissions granted.",
    },
    slotDashboard: {
      toastSaved: "Saved: slot order and enabled state written to config.",
    },
    gitInstall: {
      confirm:
        "Install plugin from Git repository:\n{url}\n\nTip: only install sources you trust. If you later see a permission error, grant it in Plugin Manager.\nContinue?",
      toastInstalled: "Installed: {id}",
    },
  },
  pluginMarketV2: {
    lead:
      "Same online index and install flow as Professional Mode (V1). Folder drop and “scan drop directory” are still located in the V1 community index section.",
    preflight: {
      dialogLabel: "Pre-apply confirmation",
      hint: "After confirming, we will sync the index and enter per-plugin permission consent flow.",
      confirmAndContinue: "Confirm and continue",
    },
    permConsent: {
      dialogLabel: "Plugin install permissions",
      hint:
        "Choose which permissions you want to grant (you can adjust them later in Professional Mode → Installed plugins → Permissions).",
      loadingTokenInfo: "Loading permission descriptions…",
      selectAll: "Select all",
      selectNone: "Select none",
      continueInstall: "Continue installation",
    },
    communityIndex: {
      title: "Community Index (Plugin Market)",
      entryTypeLabel: "Market entry type",
    },
    tabs: {
      plugin: "Plugin",
      module: "Module",
      profile: "Profile",
    },
    sources: {
      official: "Official index",
      thirdParty: "Third-party · {s}",
    },
    sync: {
      syncing: "Syncing…",
      syncOnlineIndex: "Sync online index",
    },
    offlineMode: "Offline mode (using cached local index).",
    thirdPartyWarning:
      "You are using a third-party index. Only install sources you trust and grant permissions carefully (developer mode feature).",
    emptyHint: "No index data yet. Click “Sync online index”.",
    pager: {
      toolbarLabel: "Market pagination",
      summary: "{total} items · Page {page} / {pages}",
      pageSize: "Per page",
      pageSizeAria: "Items per page",
      prev: "Previous",
      next: "Next",
    },
  },
  expertModels: {
    title: "Expert Models (Module 9)",
    subtitle:
      "Pick a Base GGUF + LoRA strengths, and optionally override PromptStyle. Session override takes priority over role default; leaving empty keeps current behavior.",
    actions: {
      refresh: "Refresh",
      backfillFromEffective: "Backfill editor from effective config",
    },
    source: {
      sessionOverride: "Session override",
      roleDefault: "Role default",
      rolePackDefault: "Role pack default",
    },
    relative: {
      justNow: "just now",
      secondsAgo: "{n}s ago",
      minutesAgo: "{n}m ago",
      hoursAgo: "{n}h ago",
      daysAgo: "{n}d ago",
    },
  },
} as const;

