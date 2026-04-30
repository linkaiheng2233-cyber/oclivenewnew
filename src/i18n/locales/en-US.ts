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
  shortcutHelp: {
    dialogLabel: "Shortcuts",
    title: "Shortcuts",
    rows: {
      ctrlShiftS: "Open Settings (extensions, security, shortcuts, and plugin configuration).",
      ctrlHoldKey: "Ctrl (hold ~1s)",
      ctrlHoldDesc: "Open this shortcuts panel",
    },
    footer: "More shortcuts will be added as features evolve.",
    launcherSlot: {
      aria: "Launcher slot",
      title: "Plugin slot (launcher.palette)",
      embedAria: "Launcher slot embed",
    },
  },
  debugPanel: {
    title: "🎛️ Developer panel",
    hint: [
      "For development and troubleshooting: view favorability, personality dimensions, recent events and memory summaries; reload policies, generate monologue, import/manage role packs, etc.",
      "Shortcut Ctrl+Shift+D toggles this panel; Esc also closes it. You can also click “Open debug panel” under the top bar More menu.",
    ],
    debugDockSlotAria: "Debug panel extension slot",
    monologue: {
      prefix: "[Monologue]",
      inserted: "Monologue inserted.",
      generating: "Generating…",
      insert: "Insert monologue",
    },
    knowledge: {
      title: "World knowledge",
      packIndex: "Pack index:",
      loaded: "Loaded",
      notLoaded: "Not loaded",
      totalChunks: "Total {n} chunks",
      lastInjected: "Injected into last prompt:",
      hint:
        "“Last prompt” updates after you send a message. Click “Refresh debug data” to sync pack chunk count (after changing files on disk, run load_role first).",
      presence: {
        coPresent: "Co-present",
        remoteStub: "Remote stub",
        remoteLife: "Remote life",
      },
    },
    favorability: {
      title: "Favorability",
      status: {
        superClose: "💖 Super close!",
        veryGood: "💕 Very good relationship",
        ok: "👍 Not bad",
        gettingToKnow: "🤝 Still getting to know",
        strangers: "😶 A bit distant",
      },
    },
    personalityVector: {
      title: "Personality vector",
      profileHint:
        "This role uses Profile-based personality source. The 7D vector here is a runtime summarized view for understanding and is not the only source of truth.",
    },
  },
  pluginMarketV1: {
    localKinds: {
      rolePack: "Role pack",
      pluginArchive: "Plugin bundle",
      pluginDir: "Plugin directory",
      moduleJson: "Module entry",
      profileJson: "Profile",
    },
    localJson: {
      toastCopied: "Copied JSON content to clipboard.",
      errors: {
        mustBeObject: "JSON must be an object.",
        typeMustBeModuleOrProfile: 'Entry type must be "module" or "profile".',
        missingRequiredFields: "Entry must include id/name/version.",
        onlyModuleProfile: "Only module/profile local entries are supported.",
      },
    },
    rolePack: {
      confirmOverwriteImport:
        "Overwrite import role pack: {name} (id={id} v{version})\n\nThis will replace the existing pack with the same id on this machine. Continue?",
      confirmImport:
        "Import role pack: {name} (id={id} v{version})\n\nImport into local roles/ ? (By default it won't overwrite the same id.)",
      toastImported: "Imported: {id}",
      toastImportedOverwrite: "Overwritten import: {id}",
    },
    perms: {
      confirmGrantAll:
        "{title}\n\nDeclared permissions:\n{list}\n\nContinuing will grant all permissions by default (you can adjust later in Professional Mode).",
    },
    install: {
      offlineBundleTitle: "Install plugin (offline bundle): {id}",
      zipTitle: "Install plugin (ZIP): {id}",
      dirTitle: "Install plugin (directory): {id}",
      confirmOverwritePlugin:
        "Allow overwriting an existing plugin with the same id?\n\nPlugin: {id}\n\nConfirm = overwrite install; Cancel = error if already exists.",
      toastInstalled: "Installed: {id}",
    },
  },
  pluginManagerV1: {
    llama: {
      toastNotFound: "Directory plugin not found: {id}",
      permConsentTitle: "What permissions are needed to enable Local Llama (directory plugin)?",
      permConsentTrustSummary:
        "Source: local directory plugin (bundled with the release or placed by you under plugins/)\nExplanation: enabling an LLM backend needs at least process:spawn to start local sidecar/llama-server.\nIf you want the plugin to download model files via URL, it also needs network:*; otherwise you can leave it unchecked and manually put .gguf files into the model directory.",
      plan: {
        writeSessionOverride: "Will write session-level backend override (current session only)",
        writePermGrants: "Will write permission grants (revocable anytime)",
      },
      preflightTitle: "Enable Local Llama (directory plugin) in one click",
      toastEnabled: "Directory LLM enabled: {id}",
    },
    sessionOverride: {
      confirmRollback:
        "Rollback session backend override (current session only)\n\nSource: {source}\nEntry: {label}\nSaved at: {savedAt}\n\nRollback now?",
      toastRolledBack: "Session backend override rolled back.",
    },
    permissions: {
      risk: {
        high: "High risk",
        medium: "Medium risk",
        low: "Low risk",
        unknown: "Unknown",
      },
      confirmHighRisk:
        "You selected high-risk permissions:\n\n{list}\n\nContinue?",
    },
    reviews: {
      none: "No reviews yet",
      summary: "{avg} ({count})",
      toastCopiedTemplate: "Copied review template (JSON).",
    },
    profile: {
      toastLoaded: "Profile loaded: {name}",
      toastNoPlugins: "This Profile declares no plugins; plugin installation is skipped.",
      toastMarketMissingPlugin: "Plugin not found in index: {id} (source={source})",
      toastApplied: "Profile applied: plugin install/permission consent done; backends override written to current session.",
    },
    marketSync: {
      toastFailed:
        "Index sync failed (source={source}): {msg}\n\nTips: check your network and retry later. For third-party sources, make sure Developer mode is enabled.",
      toastOk: "Index synced.",
    },
    modules: {
      toastMissingBody: "This entry does not provide a module body.",
      toastNoDeps: "This module declares no dependency plugins.",
      toastApplied: "Module applied: {id} (slot placement can be adjusted in the Slots tab).",
    },
    profiles: {
      toastMissingBody: "This entry does not provide a profile body.",
      toastPredeclaredPerms: "This Profile predeclares permissions: {list}",
      toastApplied: "Profile applied: {id} (slot placement can be adjusted in the Slots tab).",
    },
    localImports: {
      toastJsonCopied: "JSON copied to clipboard.",
      toastOnlyModuleOrProfile: "Only local module/profile entries are supported.",
      toastInstalled: "Installed: {id}",
      toastRolePackImported: "Imported: {id}",
      toastRolePackOverwritten: "Overwrite imported: {id}",
      confirmImportRolePack:
        "Import role pack: {name} (id={id} v{version})\n\nImport into local roles/ now? (does not overwrite same id by default)",
      confirmOverwriteRolePack:
        "Overwrite import role pack: {name} (id={id} v{version})\n\nThis will replace the existing role pack with the same id. Continue?",
      confirmHighRiskPerms:
        "This plugin contains high-risk permissions:\n{list}\n\nDo you still want to continue installation?",
      confirmOverwritePlugin:
        "Allow overwriting an existing plugin with the same id?\n\nPlugin: {id}\n\nOK = overwrite install; Cancel = error if already exists.",
      jsonParseFailed: "JSON parse failed: {msg}",
      jsonMustBeObject: "JSON must be an object.",
      entryTypeMustBeModuleOrProfile: "Local entry type must be \"module\" or \"profile\".",
      entryMissingIdNameVersion: "Local entry must include id/name/version.",
      moduleMustHaveModuleObject: "type=module must include a module object.",
      modulePluginsMustBeArray: "module.plugins must be an array.",
      profileMustHaveProfileObject: "type=profile must include a profile object.",
      profilePluginsMustBeArray: "profile.plugins must be an array.",
    },
    batch: {
      toastEnabled:
        "Enabled {n} plugins. Changes take effect after saving; restart is recommended.",
      toastDisabled:
        "Disabled {n} plugins. Changes take effect after saving; restart is recommended.",
      toastGitUpdated: "Pulled updates from Git index (ff-only). If failed, check the error.",
    },
    marketInstall: {
      toastMissingDeps: "Missing dependencies, cannot install: {list}",
      permTitleInstall: "Install {id}",
      permTitleInstallVersion: "Install {id} v{version}",
      confirmHighRisk:
        "You selected high-risk permissions.\n\nOnly install sources you trust.\n\nConfirm: continue installation?",
      confirmHighRiskVersion:
        "You selected high-risk permissions.\n\nOnly install sources you trust.\n\nConfirm: continue installing v{version}?",
      toastRolledBackOrSwitched: "Rolled back / switched {id} → v{version}",
      toastInstalledVersion: "Installed {id} v{version}",
      toastUpdated: "Updated {id} (git pull --ff-only).",
    },
    save: {
      toastSaved:
        "Plugin configuration saved. After disabling plugins, a restart is recommended to fully take effect.",
    },
    author: {
      toastAppliedSuggestedBackends:
        "Applied suggested_plugin_backends from author.json (session-level; settings.json unchanged).",
    },
    installed: {
      toastGitPulled: "Pulled updates from remote Git.",
      toastCheckUpdatesDone: "Check completed (online registry API is reserved).",
      toastZipUpdated: "Update completed. Please restart the app to take effect.",
      toastZipIdMismatch: "manifest.id in zip ({zipId}) does not match target plugin ({targetId}).",
      permTitleSideloadUpdate: "Sideload update {id}",
      sideloadSourceLocalZip: "Source: local zip (sideload)",
      confirmSideloadHighRiskUpdate:
        "You selected high-risk permissions.\n\nSideload sources cannot automatically verify publisher identity.\n\nConfirm: continue updating from local zip?",
      packStatusPickFirst: "Please select a plugin from the catalog first.",
      packStatusDone: "Pack completed: {path}",
    },
    ui: {
      dialogLabel: "Plugin workbench (Professional mode)",
      title: "Plugins & Features",
      proModeBadge: "Professional mode",
      proModeBadgeTitle: "For creators and troubleshooting: directory plugins, backends and session overrides",
      subtitle:
        "Ctrl+Shift+F toggles this window · Ctrl+Shift+A opens Plugin Market · after saving, restart is recommended for slot/enabled state.",
      loading: "Loading…",
      tabsAria: "Plugins and features sections",
      tabs: {
        plugins: "Overview",
        backends: "Chat backends",
        slots: "UI placement",
      },
      preflight: {
        dialogLabel: "Pre-apply confirmation",
        hint: "After confirming, we will sync the index and enter per-plugin permission consent flow.",
        confirmAndContinue: "Confirm and continue",
      },
      permConsent: {
        dialogLabel: "Plugin install permissions",
        trustSummaryTitle: "Trust summary",
        hint:
          "Choose which permissions you want to grant (you can adjust later in “Installed plugins → Permissions”).",
        loadingTokenInfo: "Loading permission descriptions…",
        selectAll: "Select all",
        selectNone: "Select none",
        continueInstall: "Continue installation",
      },
      market: {
        title: "Plugin Market",
        openMarket: "Open Plugin Market (Ctrl+Shift+A)",
        hint:
          "Market (community index / modules / profiles / local imports) is separated into its own window to avoid mixing with management functions.",
      },
      persistScope: {
        title: "Where are these changes saved?",
        hint:
          "“Current role” only affects this role; “Global default” becomes the default for all roles (merged with each role's own settings).",
        aria: "Plugin config persistence scope",
        roleOnly: "Current role only",
        globalDefault: "Global default",
      },
      actions: {
        install: "Install",
        update: "Update",
        enable: "Enable",
        disable: "Disable",
        updateFromGit: "Update from Git",
      },
      marketVersions: {
        rollbackOrSwitch: "Rollback / switch",
        installThisVersion: "Install this version",
        updatable: "Update available",
        installed: "Installed",
      },
      installed: {
        title: "Installed plugins (most used)",
        helpLabel: "Installed plugins help",
        batchSelect: "Batch select",
        newPlugin: "New plugin",
        packCurrent: "Pack current plugin",
        checkUpdates: "Check updates",
        enableSelected: "Enable selected",
        disableSelected: "Disable selected",
        updateSelectedFromGit: "Update selected from Git",
        selectedCount: "{n} selected",
        noDirectoryPluginsFound:
          "No directory plugins found (place plugins under a plugins/ directory next to roles/).",
        sidebarTitle: "Catalog",
        chip: {
          shell: "Shell",
          directory: "Directory",
        },
        mainSub: "Config & debug · switching plugins on the left keeps this layout",
        gitPull: "Pull updates from Git",
        updateFromZip: "Update from local zip",
      },
      localLlama: {
        title: "Enable local Llama (Directory LLM) with one click",
        hint:
          "Switch current session LLM to “directory plugin”, and write into the directory_plugins.llm slot.",
        pluginIdLabel: "Plugin ID",
        statusLabel: "Status",
        status: {
          scanned: "Found",
          notScanned: "Not found",
        },
        enableOneClick: "Enable",
        rollbackLastOverride: "Rollback last override",
      },
      slots: {
        previewTitle: "Plugin Manager preview (read-only)",
        previewHint:
          "Uses the same slot as “settings.plugins” below. Preview is not interactive; reorder by dragging in the list.",
        settingsPluginsTitle: "settings.plugins order",
        settingsPluginsHint: "Embedded area on this page; drag to reorder; choose appearance.",
        settingsPluginsAria: "Plugin manager page slot order",
        chatToolbarTitle: "chat_toolbar order",
        chatToolbarHint:
          "Drag to reorder; includes only non-shell plugins that declare this slot.",
        chatToolbarAria: "Toolbar plugin order",
        settingsPanelTitle: "settings.panel order",
        settingsPanelHint: "Embed order in Settings → Plugins; drag to reorder.",
        settingsPanelAria: "Settings page plugin order",
        roleDetailTitle: "role.detail order",
        roleDetailHint: "Embed order in role detail (below the portrait) on the left.",
        roleDetailAria: "Role detail plugin order",
        sidebarTitle: "sidebar order",
        sidebarHint: "Left sidebar extension area (below role list); drag to reorder.",
        sidebarAria: "Sidebar plugin order",
        chatHeaderTitle: "chat.header order",
        chatHeaderHint: "Top of chat column (above message list); drag to reorder.",
        chatHeaderAria: "Chat header plugin order",
        settingsAdvancedTitle: "settings.advanced order",
        settingsAdvancedHint: "Extension area in Settings dialog → General; drag to reorder.",
        settingsAdvancedAria: "settings.advanced order",
        overlayFloatingTitle: "overlay.floating order",
        overlayFloatingHint: "Floating overlay area at bottom-right of main UI; drag to reorder.",
        overlayFloatingAria: "overlay.floating order",
        launcherPaletteTitle: "launcher.palette order",
        launcherPaletteHint: "Aggregated area inside shortcut help overlay; drag to reorder.",
        launcherPaletteAria: "launcher.palette order",
        debugDockTitle: "debug.dock order",
        debugDockHint: "Extension area inside Debug panel; drag to reorder.",
        debugDockAria: "debug.dock order",
        empty: "No plugins in {slot} slot.",
      },
      footer: {
        resetToPackDefault: "Reset to role pack recommendations",
      },
    },
    ipwd: {
      toastPermUpdated: "Permissions updated.",
      privateSettingsTitle: "Plugin private settings",
      permissionsTitle: "Permissions",
      loadingTokenInfo: "Loading permission descriptions…",
      declaredFromIndexTitle: "Declared (from market index)",
      declaredFromIndexHint:
        "This is the permission scope declared by the author in the index; actual availability depends on granted permissions.",
      loading: "Loading…",
      noPermInfo:
        "No permission info (may be installed by an older version, or the plugin declares no permissions).",
      extraTag: "Extra",
      risk: {
        high: "High risk",
        medium: "Medium risk",
        low: "Low risk",
        unknown: "Unknown",
      },
      permsHint:
        "After disabling a permission, the host will deny the capability (and write audit metadata). Some changes may require restarting the plugin process.",
      debugTitle: "Debug console",
      auditTitle: "Audit (recent)",
      noAuditLogs:
        "No audit logs (metadata is written only when a call is allowed/denied).",
      auditHint: "Metadata only (no content).",
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

