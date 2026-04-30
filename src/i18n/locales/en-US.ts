export const enUS = {
  apiErrors: {
    txn: {
      TXN_BEGIN_FAILED: "Transaction begin failed. Please retry.",
      TXN_RUNTIME_ENSURE_FAILED: "Failed to initialize role runtime state.",
      TXN_PERSONALITY_INSERT_FAILED: "Failed to write personality data.",
      TXN_FAVORABILITY_UPDATE_FAILED: "Failed to update favorability.",
      TXN_FAVORABILITY_HISTORY_INSERT_FAILED: "Failed to write favorability history.",
      TXN_MEMORY_INSERT_FAILED: "Failed to save memory data.",
      TXN_SHORT_TERM_INSERT_FAILED: "Failed to write chat history.",
      TXN_SHORT_TERM_TRIM_FAILED: "Failed to trim chat history.",
      TXN_EVENT_INSERT_FAILED: "Failed to write event.",
      TXN_FAVORABILITY_READ_FAILED: "Failed to read favorability.",
      TXN_COMMIT_FAILED: "Transaction commit failed. Please retry.",
      TXN_ROLLBACK_FAILED: "Transaction rollback failed. Please contact support.",
    },
    common: {
      DB_ERROR: "Database operation failed. Please retry.",
      IO_ERROR: "Local file I/O failed. Please check environment permissions.",
      API_PLUGIN_NOT_FOUND:
        "Directory plugin not found or not scanned. Check plugin id and install path.",
      API_PERMISSION_DENIED: "Permission denied. Declare required permissions in manifest.",
      API_INVALID_MANIFEST: "Invalid plugin manifest. Check manifest.json.",
      LLM_ERROR:
        "Model call failed (common: Ollama not running, model not downloaded, or name mismatch). Run ollama list and set OLLAMA_MODEL to an existing model name (default qwen2.5:7b).",
      ROLE_NOT_FOUND: "Role not found. Check role_id.",
      ROLE_PACK_EXISTS:
        "This role ID already exists. Choose overwrite if you want to replace the local version.",
      INVALID_PARAMETER: "Invalid parameter. Check your input.",
      OLLAMA_TIMEOUT: "No response. Please ask again.",
      TXN_ROLLBACK: "Operation failed. Please retry.",
      SERDE_ERROR: "Failed to parse data. Please retry.",
      UNKNOWN_ERROR: "Unknown error. Please retry.",
      PLUGIN_PINNED_VERSION:
        "This plugin is pinned to a version tag and cannot be updated via git pull. Use Plugin Market to install/switch to a target version.",
      PLUGIN_PUBKEY_REVOKED:
        "Plugin signature public key has been revoked; installation is blocked. Contact the author or change version/source.",
      PLUGIN_PUBKEY_NOT_FOUND:
        "Signature public key for this version is not registered in the index; installation is blocked. Check that index entry and author key registration match.",
      PLUGIN_SIGNATURE_VERIFY_FAILED:
        "Plugin signature verification failed; installation is blocked. The download may be tampered with or signature mismatched.",
      PLUGIN_SIGNATURE_BASE64_INVALID: "Invalid signature file format (base64).",
      PLUGIN_SIGNATURE_SIZE_INVALID: "Invalid signature file format (byte size).",
      PLUGIN_SIGNATURE_ALGO_UNSUPPORTED: "Unsupported signature algorithm.",
      PLUGIN_SIGNATURE_ID_MISMATCH: "Signature file does not match plugin id.",
      PLUGIN_ARCHIVE_TOO_MANY_FILES: "Too many files in plugin archive; installation blocked.",
      PLUGIN_ARCHIVE_SINGLE_FILE_TOO_LARGE:
        "A file in plugin archive is too large; installation blocked.",
      PLUGIN_ARCHIVE_TOTAL_TOO_LARGE:
        "Total size of plugin archive is too large; installation blocked.",
      PLUGIN_ARCHIVE_ILLEGAL_PATH: "Illegal path found in plugin archive; installation blocked.",
      ZIP_TOO_MANY_FILES: "Too many files in zip; extraction blocked.",
      ZIP_SINGLE_FILE_TOO_LARGE: "A file in zip is too large; extraction blocked.",
      ZIP_TOTAL_TOO_LARGE: "Zip total size is too large; extraction blocked.",
      PLUGIN_PERMISSION_NOT_GRANTED:
        "Required permissions are not granted; call is blocked. Grant permissions in plugin permissions manager and retry.",
    },
    special: {
      roleNotFoundWithDetail: "Role not found or manifest missing. {detail}",
      hostJsonSerdeFailed:
        "Plugin bridge returned data that cannot be serialized to JSON. Host and plugin interface may be incompatible; check console logs.",
    },
  },
  common: {
    save: "Save",
    close: "Close",
    copy: "Copy",
    cancel: "Cancel",
    confirm: "Confirm",
    continue: "Continue",
    refresh: "Refresh",
    clear: "Clear",
    export: "Export",
    start: "Start",
    stop: "Stop",
    restart: "Restart",
    kill: "Kill",
    expand: "Expand",
    collapse: "Collapse",
    loading: "Loading…",
    enable: "Enable",
    remove: "Remove",
    security: "Security",
    advanced: "Advanced",
  },
  chat: {
    list: {
      roleSwitching: "Switching…",
      historyCtaTitle: "Expand previous conversations here",
      historyCtaSub: "{step} per click · Total {total}",
      historyExpandedLabel: "Expanded",
      itemsUnit: "items",
      showEarlierStep: "Show {step} earlier",
      collapse: "Collapse",
      historyInlineAria: "Previous conversations",
      historyInlineTitle: "Previous conversations",
      historyInlineHint: "Shares the same scroll area as the current session below",
      currentSessionDivider: "Current session below",
      empty: "No messages yet. Start chatting.",
      thinking: "Thinking…",
    },
    message: {
      fallbackBadge: "Fallback reply",
    },
    input: {
      label: "Message input",
      placeholder: "Say something…",
      send: "Send",
    },
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
        narrativeScene: {
          title: "Narrative scene",
          help:
            "Your current narrative scene; matches role pack scene config. Switching may create a history split boundary.",
          characterAt: "Character at",
        },
      },
    },
    defaults: {
      roleName: "Mumu",
    },
    interactionMode: {
      immersive: "Immersive",
      pureChat: "Chat",
    },
    toasts: {
      remoteLifeEnabled: "Remote presence enabled.",
      remoteLifeDisabled: "Remote presence disabled.",
      interactionModeSwitched: "Interaction mode switched to {mode}.",
      layoutResetOk: "Layout reset to role pack recommendations.",
      layoutResetFailed: "Reset failed: {message}",
      fallbackReply: "This was a fallback reply (auto-generated when the model returned no content).",
      roleSwitched: "Role switched: {id}",
      identityScope: {
        scene: "Scene identity",
        global: "Identity",
      },
      identitySet: "{scope} set: {name}",
      pluginInstalledFromUrl: "Plugin installed from web URL: {id}",
      pluginsAutoRefreshed: "Plugin changes detected; auto-refreshed",
    },
    sceneTravel: {
      travelingTo: "Traveling to “{label}”…",
    },
    status: {
      favorabilityAria: "Favorability",
      favorabilityLabel: "Favorability",
      lifeAria: "Life schedule inference",
      lifeNow: "Now",
    },
    floatingSlot: {
      aria: "Floating plugin slot",
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
    advancedSlot: {
      title: "Extension slot (settings.advanced)",
      hint: "Plugins that declare slot settings.advanced in manifest are embedded here.",
      aria: "Settings extension slot",
    },
    security: {
      label: "Security",
      forceIframe: {
        title: "Force iframe mode",
        hint:
          "When enabled, all plugin UIs will be loaded via iframe. It is safer but may reduce UX. Restart the app for it to fully take effect.",
      },
      forceIframeSavedToast:
        "Saved. Restart the app for Force iframe mode to fully take effect.",
    },
    plugins: {
      devMode: {
        sectionLabel: "Advanced (Developer mode)",
        help: {
          p1: "Developer mode enables third-party plugin index sources and sideload install entry points.",
          p2: "Only add sources you trust; unsigned sources will show strong warnings on install.",
        },
        title: "Enable Developer mode",
        hint:
          "After enabling, you can use third-party index sources and sideload install. Recommended for advanced users only.",
        enabledToast: "Developer mode enabled.",
        disabledToast: "Developer mode disabled.",
      },
      sources: {
        hint:
          "Third-party index sources (one URL per line). After saving, you can sync index in Plugin Manager with this URL (source picker UI will be added later).",
        saveButton: "Save third-party sources",
        savedToast: "Third-party index sources saved.",
      },
      directorySlot: {
        title: "Directory plugins · Settings slot",
        help: {
          p1:
            'Declare slot "settings.panel" under ui_slots in the plugin manifest to embed a settings page here.',
          p2:
            "Same as chat_toolbar: load via https://ocliveplugin.localhost/<id>/<entry>. You can reorder or hide them in Plugin Manager.",
        },
      },
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
  rolePackBar: {
    barTitle:
      "Install .ocpak/.zip archives or an extracted folder (same structure as roles/{id}/)",
    actions: {
      export: "Export role pack",
      importArchive: "Import archive",
      importFolder: "Import folder",
      openMarket: "Optional: roles.json index",
    },
    export: {
      filterName: "OCPak role pack",
    },
    progress: {
      preparing: "Preparing…",
    },
    toasts: {
      exported: "Role pack exported.",
      imported: "Role imported: {id}",
      importedNoSwitch: "Role imported: {id} (not switched)",
    },
    creatorEcho: {
      titleWithMessage: "Creator note",
      titleSuccess: "Import successful",
      promptSwitchNow: "Role pack imported. Switch to this role now to start using it?",
      later: "Later",
      switchNow: "Switch now",
    },
    conflict: {
      title: "Role already exists",
      bodyHtml:
        "Local role ID “<strong>{id}</strong>” already exists ({name} v{version}). Import will overwrite the role folder. Continue?",
      overwrite: "Overwrite import",
    },
    market: {
      title: "Role pack market (roles.json)",
      hint:
        "Direct downloads will be imported after downloading and verifying SHA-256. Page/cloud links require manual download, then import via “Import archive”.",
      searchPlaceholder: "Search id / name / author / tags…",
      syncing: "Syncing…",
      sync: "Sync",
      sourcePlaceholder: "Optional: custom roles.json source (empty = official default)",
      loading: "Loading…",
      empty: "Index is empty (or not loaded).",
      installPicked: "Install selected mirror",
      notDirectHint:
        "This mirror is not a direct download (page/pan). We tried to open the link; please download manually and install via “Import archive”.",
      confirmOverwrite:
        "Role “{id}” already exists locally. Overwrite install?",
    },
  },
  roleRuntimePanel: {
    meta: {
      versionAuthor: "Version {version} · Author {author}",
    },
    personality: {
      sourceLabel: {
        profile: "Archive (mutable text maintained by chat)",
        vector: "7D vector",
      },
      sourceLabelTitle: "Personality source",
      hints: {
        profileP1:
          'Source=profile: runtime uses core personality + the database "mutable personality archive". The 7D view is summarized from text.',
        profileP2:
          "Different from vector mode (7D directly participates in evolution). See docs/personality-archive-notes.md.",
        vectorP1:
          "Source=vector: events and emotions are adjusted with fine-grained 7D; matches settings evolution.personality_source.",
      },
    },
    backendHint: {
      prefix:
        "Module backends, Remote Life, session overrides and debug snapshots have moved to",
      linkText: "Plugins & backends → Backends",
      suffix: "(Ctrl+Shift+F)",
    },
    feedback: {
      leadHtml:
        'Feel something is off? You can send a <strong>semi-private feedback</strong> to the author (saved locally, not publicly displayed).',
      openButton: "Feedback this role pack",
    },
    fields: {
      relation: "Relation",
      eventImpact: "Event impact",
    },
    feedbackModal: {
      title: "Feedback this role pack",
      sub:
        "By default, this feedback is only visible to the author (semi-private) to iterate the role pack. Please avoid personal privacy information.",
      moodLabel: "Mood tag (optional)",
      moodPlaceholder: "e.g. happy / sad / angry / confused / none",
      messageLabel: "Message",
      messagePlaceholder: "Describe the issue/suggestion you encountered (required)",
      submitting: "Submitting…",
      submit: "Submit feedback",
    },
    toasts: {
      submitted: "Feedback submitted (only visible to the author).",
      submitFailed: "Submit failed",
    },
  },
  pluginBackendSessionPanel: {
    leadHtml:
      "Below are the pack default and session-level overrides of <strong>settings.json → plugin_backends</strong>; it does not write to role pack files on disk.",
    sources: {
      packDefault: "Pack default",
      sessionOverride: "Session override",
      envOverride: "Env override",
    },
    modules: {
      memory: "Memory",
      emotion: "Emotion",
      event: "Event",
      prompt: "Prompt",
      llm: "LLM",
      agent: "Agent",
    },
    directoryPlugins: {
      packLine: "Pack · directory_plugins: {v}",
      effectiveLine: "Effective · directory_plugins: {v}",
    },
    meta: {
      packTitle: "settings.json → plugin_backends",
      packLabel: "Module backends",
      sessionEffectiveTitle: "Session effective",
      sessionEffectiveLabel: "Session effective",
      hasSessionOverrideHint:
        "Session override is enabled (current session only; not written to role pack).",
      sourcesTitle: "Sources",
      sourcesLabel: "Sources",
    },
    remoteLife: {
      label: "Remote Life",
      packDefaultHint: "Pack suggests enabled",
    },
    followPackDefault: "Follow pack default ({v})",
    localMemory: {
      placeholder: "provider_id; empty clears current session override",
      applyToSession: "Apply to session",
    },
    debugSnapshot: {
      label: "Debug snapshot",
      copied: "Copied",
      copyFailed: "Copy failed",
    },
    pack: {
      label: "Pack plugin",
      oneClick: "One-click pack (agent/llm)",
      noTargetHint:
        "Configure a target plugin in directory plugin slots first (agent or llm).",
      done: "Packed: {path} (sha256={sha})",
    },
  },
  pluginManagerV2: {
    sources: {
      sessionOverride: "Session override",
      envOverride: "Env override",
      packDefault: "Role pack default",
      envVar: "Environment variables",
      sessionEnabled: "Enabled in current session",
      sessionDisabled: "Disabled in current session",
    },
    modules: {
      llm: "Chat brain (LLM)",
      emotion: "Emotion engine (Emotion)",
      complexEmotion: "Complex emotion (Complex Emotion)",
    },
    options: {
      followPackDefault: "Follow role pack default ({v})",
      ollama: "Ollama (local model)",
      remote: "Remote service",
      directory: "Directory plugin",
      builtin: "Built-in",
      builtinV2: "Built-in V2",
    },
    cards: {
      llmMain: {
        title: "Reply engine",
        description: "Choose reply model source: local model, remote service, or directory plugin.",
      },
      llmEndpoint: {
        title: "LLM remote endpoint notes",
        description: "When using remote service, prefer the LLM-specific endpoint.",
        summary: "Tip: set endpoints via system env vars for easier migration and debugging.",
        fields: {
          remoteLlmUrl: "LLM-specific remote URL (preferred)",
          remotePluginUrl: "Generic remote URL (fallback)",
        },
      },
      emotionMain: {
        title: "Emotion inference engine",
        description: "Choose emotion handling: built-in logic, remote service, or directory plugin.",
      },
      emotionEndpoint: {
        title: "Emotion remote endpoint notes",
        description: "Emotion remote reads the generic remote URL by default.",
        summary: "Tip: set endpoints via system env vars to avoid hardcoding into role packs.",
        fields: {
          remotePluginUrl: "Common Emotion remote entry",
        },
      },
      complexSwitch: {
        title: "Complex emotion toggle",
        description: "Enables remote presence chain; complex emotion becomes more evident.",
        label: "Enable complex emotion (remote presence)",
        hint: "After enabling, configure URL and TOKEN env vars.",
      },
      complexEndpoint: {
        title: "Complex emotion endpoint notes",
        description: "Complex emotion service is usually deployed separately and supports auth token.",
        summary: "If auth is required, set both URL and TOKEN.",
        fields: {
          url: "Complex emotion service URL",
          token: "Complex emotion auth token",
        },
      },
    },
    categories: {
      all: "All features",
      builtin: "Built-in",
      remote: "Remote",
      directory: "Local directory plugins",
      statusEnabled: "Enabled",
      statusDisabled: "Disabled",
      statusNeedsConfig: "Needs setup",
    },
    toasts: {
      endpointNoSave: "Endpoint notes are not saved. Configure via environment variables.",
      complexSwitchUpdated: "Complex emotion toggle updated.",
      writtenToSession: "Config written to current session.",
    },
    errors: {
      onlyLlmEmotionSupported: "Only LLM / Emotion config writing is supported currently.",
    },
    list: {
      searchPlaceholder: "Search: e.g. remote, emotion, directory plugin",
      empty: "No matches. Try a shorter keyword.",
    },
    card: {
      status: {
        enabled: "Enabled",
        needsConfig: "Needs setup",
        disabled: "Disabled",
      },
      type: {
        builtin: "Built-in",
        remote: "Remote",
        directory: "Directory plugin",
      },
      risk: {
        needsConfig: "Missing config",
        envFirst: "Env first",
      },
    },
    rightPanel: {
      placeholder: "Select a card from the middle list first.",
      changeNotice: {
        readonly:
          "Read-only note: nothing is written here. Modify via environment variables or role pack, then reload the app.",
        preview:
          "Change preview: click “Apply changes” below to write into current session (does not modify role pack settings.json; if env vars conflict, env takes precedence).",
      },
    },
    filters: {
      explorerAria: "Filters (workspace style)",
      explorerTitle: "Explorer",
      explorerSub: "Filter view",
      rootTitle: "UI hierarchy only; not a disk path",
      treeAria: "Filter tree",
    },
    legend: {
      aria: "Status legend",
      enabled: "Enabled: effective immediately",
      pending: "Needs setup: usually missing directory plugin id",
      disabled: "Disabled: this chain is not enabled",
    },
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
      dialogAria: "Plugin permissions",
      title: "Plugin permissions: {id}",
      loadingTokenInfo: "Loading permission descriptions…",
      grantAllDeclared: "Grant all declared permissions",
      grantMissing: "Grant missing",
      grantMissingTitle: "Grant missing: {missing}",
      declaredTitle: "Declared (from index / install metadata)",
      loading: "Loading…",
      noPermInfo:
        "No permission info (may be installed by an older version, or the plugin declares no permissions).",
      extraTag: "Extra",
      hint:
        "After disabling a permission, the host will deny the capability. Some changes may require restarting the plugin process.",
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
      aria: "Quick slot configuration",
      title: "Put plugins into the UI",
      helpLabel: "What is this for?",
      helpLine1: "Two steps: pick a slot (where the plugin shows), then check plugins to show.",
      helpLine2:
        "If a plugin doesn't declare the slot in its manifest, it won't appear here.",
      toastSaved: "Saved: slot order and enabled state written to config.",
      slotLabel: "Slot",
      enabledCount: "Enabled {enabled} / {total}",
      noSlots: "No available slots detected.",
      pickPluginsTitle: "Select plugins to show",
      noPluginsForSlot: "No plugins available for this slot (no plugin declares it).",
      missingPerms: "Missing perms ({n})",
      permsBtn: "Permissions",
      fixPermsBtn: "Fix",
      toggleEnableTitle: {
        enable: "This plugin is disabled; click to enable",
        disable: "This plugin is enabled; click to disable",
      },
      enabled: "Enabled",
      disabled: "Disabled",
      orderTitle: "Display order (top to bottom)",
      noPickedPlugins: "No plugins selected yet.",
      moveUp: "Up",
      moveDown: "Down",
    },
    gitSection: {
      aria: "Install plugin from Git",
      title: "Install from Git repository",
      helpLabel: "What is “install from Git”?",
      helpLine1: "Good for pulling plugins directly from GitHub or self-hosted Git repos.",
      helpLine2:
        "For cloud drive / zip packages, put them into the drop directory and install via “Plugin Market → Local imports”.",
      installing: "Installing…",
      install: "Install",
      hint:
        "Tip: only install sources you trust. If you later see a permission error, grant it in the permissions panel.",
    },
    localLlamaSection: {
      aria: "Local Llama (basic)",
      title: "Local Llama (basic)",
      helpLabel: "Why set it here?",
      helpLine1:
        "This is the minimal path: grant required permissions and switch current session LLM backend to directory.",
      helpLine2: "Advanced model/log/tuning workbench stays in V1.",
      clearOverride: "Clear session override",
      pluginIdLabel: "Plugin ID",
      statusLabel: "Status",
      status: {
        scanned: "scanned",
        notScanned: "not scanned",
      },
      enableForSession: "Enable (current session)",
      effectiveLabel: "Effective LLM",
    },
    localLlama: {
      effective: {
        notSet: "not set",
        directoryWithId: "directory · {id}",
        directoryNoId: "directory · (plugin id not set)",
      },
      toastNotScanned: "Directory plugin not found: {id}",
      confirmEnable:
        "Enabling local Llama (current session) will grant these permissions:\n{list}\n\nAnd switch LLM backend to directory: {id}\n\nContinue?",
      toastEnabled: "Local Llama enabled: {id} (current session)",
      confirmClearSessionOverride:
        "Clear current session LLM backend override and restore role pack/default settings. Continue?",
      toastClearedOverride: "Current session LLM override cleared.",
    },
    gitInstall: {
      confirm:
        "Install plugin from Git repository:\n{url}\n\nTip: only install sources you trust. If you later see a permission error, grant it in Plugin Manager.\nContinue?",
      toastInstalled: "Installed: {id}",
    },
  },
  pluginStore: {
    errors: {
      depsNotMet: "Plugin “{id}” dependencies are not satisfied and cannot be enabled. {issues}",
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
      trustSummaryTitle: "Trust summary",
      hint:
        "Choose which permissions you want to grant (you can adjust them later in Professional Mode → Installed plugins → Permissions).",
      loadingTokenInfo: "Loading permission descriptions…",
      selectAll: "Select all",
      selectNone: "Select none",
      continueInstall: "Continue installation",
    },
    market: {
      actions: {
        rollbackOrSwitch: "Rollback / switch",
        installThisVersion: "Install this version",
        install: "Install",
        updatable: "Updatable",
        installed: "Installed",
        update: "Update",
      },
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
      chunksUnit: "chunks",
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
      traits: {
        stubbornness: "Stubbornness",
        clinginess: "Clinginess",
        sensitivity: "Sensitivity",
        assertiveness: "Assertiveness",
        forgiveness: "Forgiveness",
        talkativeness: "Talkativeness",
        warmth: "Warmth",
      },
    },
    meta: {
      eventsMemories: "Events: {events} · Memories: {memories}",
      recentEvents: "Recent events",
      recentMemories: "Recent memories",
    },
    actions: {
      refreshDebugData: "Refresh debug data",
      reloadPolicy: "Reload policy",
    },
    footer: "💡 Ctrl+Shift+D toggles panel · role pack & monologue live here",
  },
  timeDial: {
    title: "Adjust virtual time",
    hint:
      "Scroll to pick date and time (like a system wheel), accurate to minutes.",
    aria: {
      dateTimeGroup: "Date and time",
    },
    fields: {
      year: "Year",
      month: "Month",
      day: "Day",
      hour: "Hour",
      minute: "Minute",
    },
    units: {
      month: "",
      day: "",
    },
    presets: {
      nextMorning: "Next morning",
      skipIdleTime: "Skip idle time",
    },
    applying: "…",
    confirm: "Confirm selected time",
    toasts: {
      updated: "Virtual time updated.",
      favorabilityDelta: "Favorability {sign}{delta} (current {current})",
    },
  },
  chatExport: {
    exportAllRoles: "Export all roles",
    includePluginDebugSingleRole: "Include plugin diagnostics (single role)",
    exportJson: "Export JSON",
    exportTxt: "Export TXT",
    toasts: {
      downloaded: "Downloaded {name}",
      exported: "Exported.",
      cancelled: "Save cancelled.",
    },
  },
  agentDebugPanel: {
    title: "Agent debug traces",
    lead:
      "Inspect MCP servers, manually call tools, and view recent agent task breakdowns and tool-call traces.",
    common: {
      pick: "Please choose",
    },
    actions: {
      refreshServers: "Refresh MCP servers",
      refreshTraces: "Refresh agent traces",
      clearTraces: "Clear traces",
      callTool: "Call tool",
    },
    templates: {
      title: "Template library",
      pickPlaceholder: "Choose a template",
      saveAsPlaceholder: "Save current request as a custom template",
      save: "Save template",
      weather: "Weather query",
      fileRead: "Read file",
      webFetch: "Fetch web page",
    },
    tool: {
      placeholder: "e.g. get_weather",
      pickFromList: "Pick from server tool list",
    },
    diff: {
      title: "Response diff",
      left: "Left response",
      right: "Right response",
      compare: "Compare",
      noDiff: "No differences",
    },
    traces: {
      title: "Recent tasks",
      empty: "No agent traces yet.",
    },
  },
  envVarManager: {
    title: "Environment variables (session draft)",
    keyPlaceholder: "OCLIVE_*",
    valuePlaceholder: "value",
    upsert: "Add / update",
    remove: "Remove",
    copyAsCommand: "Copy as terminal command",
    copied: "Copied",
  },
  pluginScaffoldWizard: {
    title: "Create plugin scaffold",
    fields: {
      id: "Plugin ID",
      name: "Plugin name",
      language: "Language",
      type: "Type",
      outputDirOptional: "Output directory (optional)",
      outputDirPlaceholder: "Leave empty to use default plugins/",
    },
    types: {
      moduleExt: "6-module extension",
    },
    validation: {
      title: "Manifest validation (live)",
      ok: "Manifest validation passed",
    },
    actions: {
      create: "Generate scaffold",
    },
    status: {
      created: "Generated: {dir}",
    },
    errors: {
      missingField: "Missing required field: {field}",
      missingProcessOrRemoteUrl: "Missing process or remote_url",
      invalidPermission: "permissions contains invalid value: {p}",
    },
  },
  pluginPrivateSettingsForm: {
    loading: "Loading settings…",
    empty: "This plugin did not declare uiSchema.fields.",
    templateLabel: "Template",
    saving: "Saving…",
    save: "Save private settings",
    toasts: {
      saved: "Private plugin settings saved.",
    },
  },
  hotkeySettings: {
    title: "Global shortcuts",
    lead:
      "All disabled by default. When enabled, they are registered as system-wide shortcuts and may conflict with other apps. Save errors will show the reason.",
    fields: {
      accelerator: "Shortcut",
      acceleratorPlaceholder: "e.g. Ctrl+Shift+L",
      action: "Action",
      pluginId: "Plugin id",
      slot: "Slot name",
      appearanceOptional: "appearance (optional)",
    },
    actions: {
      openLauncherList: "Open plugin catalog list",
      openPluginSlot: "Open a plugin slot page",
    },
    addOne: "Add one",
    toasts: {
      saved:
        "Shortcut config saved (only enabled bindings will be registered as global shortcuts).",
    },
  },
  hotkeyHost: {
    dialogs: {
      pluginWindowAria: "Plugin quick window",
      pluginNotFoundAria: "Plugin not found",
      launcherAria: "Launchable plugins",
    },
    notFound: {
      title: "Cannot open plugin page",
      body:
        "This role bootstrap does not include UI for plugin {pluginId} in slot {slot}. Please ensure the plugin is enabled, its slot contribution is not hidden, and plugin config has been saved.",
    },
    launcher: {
      title: "Plugin catalog",
      empty: "No plugins were scanned.",
    },
  },
  helpHint: {
    ariaLabel: "View help",
  },
  asyncPluginVue: {
    securityWarning: {
      title: "Plugin security warning",
      body:
        "This plugin contains potentially dangerous code:\n{list}\n\nContinue loading?",
    },
  },
  pluginUiTemplates: {
    slotSelector: {
      backendLabel: "Backend mode",
      directoryIdLabel: "Directory plugin ID",
      directoryIdPlaceholder: "e.g. my-plugin-id",
      directoryPickPlaceholder: "Type manually or pick a candidate",
      hint:
        "Leaving it empty will clear the session override and return to role pack defaults.",
      apply: "Apply changes",
    },
  },
  pluginSettingsPanelSlots: {
    tabsAria: "Plugin settings page",
    empty: "No plugins declare the settings.panel slot.",
  },
  pluginListItem: {
    aria: "Plugin {id}",
    kind: {
      shell: "Shell",
      slot: "Slot",
    },
    uiSlotsLabel: "UI slots",
    depsNotMet: "Dependencies not met ({status})",
    disablePlugin: "Disable plugin",
    hide: {
      chatToolbar: "Hide chat toolbar embed",
      settingsPanel: "Hide settings embed",
      roleDetail: "Hide role detail embed",
      sidebar: "Hide sidebar embed",
      chatHeader: "Hide chat header embed",
    },
  },
  pluginDebug: {
    target: "Target",
    status: {
      runningPid: "Running · PID {pid}",
      notRunning: "Not running",
    },
    tabs: {
      aria: "Debug sections",
      process: "Process",
      console: "Console",
    },
    console: {
      hint:
        'The "Output" below is a ring buffer (~1000 lines) polled by the host, similar to an IDE console.',
    },
    inspector: {
      aria: "Response and log output",
      response: "Response",
      output: "Output",
      noResponse: "(No response yet)",
    },
    process: {
      thisPlugin: "This plugin",
      runningPid: "Running · PID {pid}",
      notStarted: "Not started",
      spawnUnsupportedHint:
        "This plugin manifest does not declare `process`, so it cannot be started here. If the process is launched by the host or other means, you can still send requests in the RPC tab to a ready endpoint.",
      refreshList: "Refresh process list",
      managedByHost: "Host-managed plugin processes",
    },
    rpc: {
      method: "Method",
      discover: "Discover methods",
      paramsJson: "Params JSON",
      send: "Send",
      format: "Format",
      historyTitle: "Request history (click to fill)",
    },
    logs: {
      filterPlaceholder: "Filter logs…",
      empty:
        "No logs yet (stdout/stderr will appear here after starting the plugin process).",
    },
  },
  pluginMarketV1: {
    panel: {
      dialogLabel: "Plugin Market",
      title: "Plugin Market",
    },
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
        parseFailed: "JSON parse failed: {msg}",
        mustBeObject: "JSON must be an object.",
        typeMustBeModuleOrProfile: 'Entry type must be "module" or "profile".',
        missingRequiredFields: "Entry must include id/name/version.",
        onlyModuleProfile: "Only module/profile local entries are supported.",
      },
    },
    localImports: {
      title: "Local imports (drop folder)",
      hint:
        "After placing files into the drop folder, click “Scan drop folder” to discover them, then manually confirm install/import.",
      rootLabel: "Root dir",
      scanning: "Scanning…",
      scan: "Scan drop folder",
      empty: "No candidates yet.",
      actions: {
        import: "Import",
        overwriteImport: "Overwrite import",
        install: "Install",
        apply: "Apply",
        copyJson: "Copy JSON",
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
    shell: {
      title: "Shell capabilities (Module 8)",
      hint:
        "Slots are part of the frontend shell capability set. Plugins can render or degrade based on capabilities. Backend bootstrap returns supported slot names for this release.",
      noSupportedSlotsHint:
        "supportedUiSlots is not provided (may be an older core/backend). Treating as “all supported” for compatibility.",
      supportedLabel: "Supported:",
      unsupportedOfficialLabel: "Unsupported (official slots):",
    },
    profileSection: {
      title: "Profile (fingerprint / one-click setup)",
      loadingPreview: "Loading…",
      pickFile: "Pick Profile file",
      applying: "Applying…",
      apply: "Apply Profile",
      hint:
        "Profile syncs index by declared sources and installs plugins one by one (permission consent per plugin), then writes backends into current session override.",
      marketSourcesLabel: "Market sources",
      developerModeLabel: "Developer mode",
      devModeOn: "On",
      devModeOff: "Off",
      pluginsLabel: "Plugins",
      backendsLabel: "Backends override",
      empty: "No Profile selected.",
    },
    authorSuggestedBackends: {
      title: "Author suggestion · Backends",
      hint:
        "Writes suggested_plugin_backends from author.json into current session backends override (same as the Backends tab).",
      apply: "Apply author-suggested backends",
    },
    authorPack: {
      title: "Author & recommendations",
      optional: "(optional)",
      empty: "No recommended_plugins listed.",
    },
    communityIndex: {
      title: "Community index",
      tabs: {
        aria: "Market entry type",
        plugin: "Plugin",
        module: "Module",
      },
      source: {
        official: "Official default index",
        thirdParty: "Third-party · {s}",
      },
      syncing: "Syncing…",
      sync: "Sync online index",
      offlineHint: "Offline mode (using local cached index).",
      thirdPartyWarning:
        "You are using a third-party index source. Only install sources you trust and grant permissions carefully (Developer mode feature).",
      emptyHint: "No index data yet. Click “Sync online index”.",
      pager: {
        aria: "Market pagination",
        summary: "{total} items · Page {page} / {pages}",
        perPage: "Per page",
        perPageAria: "Items per page",
        prev: "Prev",
        next: "Next",
      },
      sourceBadge: {
        officialTitle: "Official default index",
        thirdTitle: "Third-party index source",
        official: "Official",
        third: "Third-party",
      },
      entryTypeBadge: {
        moduleTitle: "No-code module entry",
        profileTitle: "No-code Profile entry",
        module: "Module",
        profile: "Profile",
      },
      trust: {
        source: "Source",
        publisher: "Publisher",
        pubkeysTitle: "Public key status registered in index",
        pubkeys: "Pubkeys",
      },
      trustLine: {
        source: "Source: {v}",
        publisher: "Publisher: {v}",
        pubkeys: "Pubkeys: {v}",
      },
      reviews: {
        overallTitle:
          "Public reviews (overall): {rating}\n\nTip: reviews should be bound to pluginId+pubkeyId (signing public key) by default.",
        pubkeyDimension: "Pubkey dimension:",
        copyPubkeyTemplateTitle: "Copy JSON review template bound to this pubkeyId",
        goContribute: "Contribute review",
        copyOverallTemplateTitle:
          "Copy a JSON template that can be directly submitted to reviews.json (recommended: use pubkeyId dimension)",
        copyTemplate: "Copy template",
        refresh: "Refresh reviews",
        recent: "Recent short reviews:",
      },
      details: {
        viewModule: "View module declaration",
        viewProfile: "View Profile declaration",
        deps: "Dependencies",
        predeclaredPerms: "Predeclared permissions",
        backends: "Backends override (session-level):",
        noBackends: "No backends override declared.",
      },
      missingDeps: "Missing deps",
      applyModule: "Apply module",
      applyProfile: "Apply Profile",
    },
    applyPlan: {
      type: {
        module: "Type: Module (no code)",
        profile: "Type: Profile (no code)",
      },
      entry: "Entry: {id}",
      willSyncSources: "Will sync index sources: {list}",
      willInstallDeps: "Will install dependency plugins: {list}",
      willWriteSessionOverride: "Will write backend override (session-level):",
      titleModule: "Apply module: {id}",
      titleProfile: "Apply profile: {id}",
      depNotFound:
        "Dependency plugin not found in index: {id} (source={source})\n\nTip: verify it exists in that source, or switch to the correct source and sync again.",
    },
    localImports: {
      kindLabels: {
        rolePack: "Role pack",
        pluginArchive: "Plugin archive",
        pluginDir: "Plugin directory",
        moduleEntry: "Module entry",
        profileEntry: "Profile",
      },
      sectionTitle: "Local imports (drop folder)",
      sectionHintHtml:
        "This is an <strong>add-only entry</strong>: after you drop files into the directory, Oclive Manager only <strong>discovers</strong> them. You still need to <strong>confirm permissions / enable</strong> here; nothing runs automatically.",
      rootLabel: "Root",
      paths: {
        roles: "Role packs (.ocpak/.zip)",
        pluginsPlugin:
          "Plugin archives (.zip/.oclive-plugin) or plugin directories (with manifest.json)",
        pluginsModule: "Plugin modules (module entry JSON, no code)",
        profiles: "Profile JSON (.oclive.profile.json etc.)",
      },
      scanning: "Scanning…",
      scan: "Scan drop folder",
      showAll: "Show all",
      empty: "No candidates.",
      cols: {
        rolePacks: "Role packs",
        plugins: "Plugins",
        moduleProfile: "Module / Profile",
      },
      actions: {
        import: "Import",
        overwriteImport: "Overwrite import",
        overwriteImportTitle:
          "Overwrite import: replace local role pack when the same role_id already exists",
        install: "Install",
        hide: "Hide",
        apply: "Apply",
        copyJson: "Copy JSON",
      },
      permTitleOfflinePackage: "Install plugin (offline package): {id}",
      permTitleZip: "Install plugin (ZIP): {id}",
      permTitleDir: "Install plugin (directory): {id}",
      sideloadTrustSummary: "Source: local drop folder (Developer mode)",
      offlineTrustSummary: "Source: local drop folder (Developer mode)\n{signature}",
      signature: {
        verified: "Signature: verified",
        unknown: "Signature: unknown",
        message: "Signature: {msg}",
      },
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
    noCodeModules: {
      title: "Module management (no-code entries)",
      hint:
        "Modules are “recipes”: declare dependency plugins + optional backends override. Dropping into the import folder will not auto-enable; you must confirm manually here.",
      scanning: "Scanning…",
      scanLocal: "Scan local modules",
      rollbackTitle: "Rollback snapshot: {label} @ {savedAt}",
      rollbackLast: "Rollback last override",
      localTitle: "Local modules (imports/plugins/module)",
      localEmptyHtml:
        "No local modules. Put the module JSON into <code>{dir}</code>.",
      applyModule: "Apply module",
      copyJson: "Copy JSON",
      marketTitle: "Market modules (type=module)",
      marketEmpty: "No module entries in current index. Sync in “Plugin Market” first.",
    },
    noCodeProfiles: {
      title: "Profile management (no-code entries)",
      hint:
        "Profiles are higher-level “environment recipes”: dependency plugins + optional backends override + optional predeclared permission hints. Dropping into the import folder will not auto-enable.",
      scanning: "Scanning…",
      scanLocal: "Scan local Profiles",
      rollbackTitle: "Rollback snapshot: {label} @ {savedAt}",
      rollbackLast: "Rollback last override",
      localTitle: "Local Profiles (imports/profiles)",
      localEmptyHtml:
        "No local Profiles. Put the profile JSON into <code>{dir}</code>.",
      applyProfile: "Apply Profile",
      copyJson: "Copy JSON",
      marketTitle: "Market Profiles (type=profile)",
      marketEmpty: "No Profile entries in current index. Sync in “Plugin Market” first.",
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
      toastInstalledRecommendedRestart:
        "Installed {id}. Tip: save config and restart the app if needed.",
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
        helpLine1:
          "Manage enable/disable/update here first, then adjust where plugins appear in the Slots tab.",
        helpLine2: "This is the most frequently used area for daily operations.",
        batchSelect: "Batch select",
        newPlugin: "New plugin",
        packCurrent: "Pack current plugin",
        checkUpdates: "Check updates",
        enableSelected: "Enable selected",
        disableSelected: "Disable selected",
        updateSelectedFromGit: "Update selected from Git",
        primaryHelpLabel: "Enable/disable & update notes",
        primaryHelpLine1: "Enable/disable controls whether a plugin participates in runtime and rendering.",
        primaryHelpLine2:
          "Update from Git only works for plugins installed via git; plugins pinned to a tag cannot pull.",
        primaryHelpLine3: "After updating, restarting the app is recommended for more stable slot rendering.",
        primaryActionsAria: "Installed plugins primary actions",
        batchActionsAria: "Batch actions",
        workspaceAria: "Plugin workspace",
        catalogAria: "Installed directory plugins",
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
        hasUpdateBadge: "New version",
      },
      toasts: {
        resetToPackDefaultOk:
          "Layout reset to current role pack recommendation (author.suggested_ui preferred, otherwise ui.json).",
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
        rollbackTitle: "Rollback snapshot: {label} @ {savedAt}",
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
    common: {
      notSet: "(not set)",
      empty: "(empty)",
      yes: "Yes",
      no: "No",
    },
    strengthWarning: {
      mustBeNumber: "Strength must be a number.",
      ltZero: "Strength < 0 is usually unreasonable.",
      gtTwo: "Strength > 2 may degrade or destabilize outputs.",
      highSuggestion: "Strength is high; try 1.0–1.4 first.",
    },
    import: {
      baseDialogTitle: "Pick a Base GGUF (will copy into models/gguf)",
      loraDialogTitle: "Pick a LoRA GGUF (will copy into models/loras)",
    },
    toasts: {
      appliedToSession:
        "Applied to current session (will restart local llama).\nmodelPath={modelPath}\nllamaArgs={llamaArgs}",
      rolledBackAndApplied:
        "Rolled back and re-applied.\nmodelPath={modelPath}\nllamaArgs={llamaArgs}",
      retriedAndApplied: "Retried and applied.\nmodelPath={modelPath}\nllamaArgs={llamaArgs}",
      importedBase: "Imported Base model into models/gguf.",
      importedLora: "Imported LoRA into models/loras.",
      setAsRoleDefault: "Set as role default.",
      clearedSessionOverrideAndApplied: "Cleared session override and re-applied.",
      clearedRoleDefault: "Cleared role default.",
    },
    confirm: {
      rollbackLastRun:
        "Rollback to last applied config (Module 9 Ctrl+Z) and re-apply to current session.\nTip: you can rollback to any run in Run History.\nContinue?",
      retryRunApply:
        "Retry this target config and re-apply to current session:\nBase={base} / LoRA={loras} / PromptStyle={promptStyle}\nContinue?",
      exportWorkflowFile:
        "Export workflow file (shareable for others to import and reproduce):\nBase={base} / LoRA={loras} / PromptStyle={promptStyle}\nFilename: {filename}\nContinue?",
      rollbackSummaryLine: "\nRollback to: Base={base} / LoRA={loras} / PromptStyle={promptStyle}",
      rollbackToSelectedRun:
        "Rollback to the selected history config and re-apply to current session.{summary}\nContinue?",
      clearRunsAll: "Clear current session Run History (all). Continue?",
      clearRunsWithMode: "{modeLabel}.{keepPinned}\nContinue?",
      clearSessionOverrideAndApply:
        "Clear current session Expert Models override and rollback to role default / role pack default.\nContinue?",
      clearRoleDefault:
        "Clear this role's Expert Models default (will not change role pack files).\nContinue?",
    },
    runHistory: {
      errors: {
        noTargetGraphForRetry:
          "This run has no saved targetGraph (may be from an older version); cannot retry.",
        noTargetGraphForSaveWorkflow:
          "This run has no saved targetGraph (may be from an older version); cannot save as workflow.",
        noTargetGraphForExportWorkflow:
          "This run has no saved targetGraph (may be from an older version); cannot export workflow file.",
      },
      prompts: {
        saveAsWorkflowName: "Save as workflow: enter a name",
      },
      toastCopiedDiagnostics: "Run diagnostics copied.",
      toastSavedToLibrary: "Saved to workflow library: {name}",
      toastExportedShareable: "Workflow file exported. You can share it for others to import.",
      toastNoPinnedRuns: "No pinned runs (★). Pin a run first.",
      toastCleared: "Run History cleared.",
      toastClearedWithMode: "Clear operation executed.",
      ui: {
        title: "Run History ({n})",
        applyingTitle: "Applying…",
        applyingHint: "Local llama will restart; please wait.",
        exportPinned: "Export latest ★",
        clearExecute: "Clear",
        filterStatus: {
          all: "All",
          unknown: "Unknown",
        },
        searchBasePlaceholder: "Search Base filename…",
        emptyHint:
          "No Run History yet. A snapshot is recorded before each “Apply to current session”.",
        pinTitle: {
          pin: "Pin (keep preferred)",
          unpin: "Unpin (allow trimming/clearing)",
        },
        basePill: "Base: {name}",
        durationPill: "Duration: {ms}ms",
        expandDetail: "Details",
        collapseDetail: "Hide details",
        rollbackToHere: "Rollback to here",
        retry: "Retry",
        copyDiagnostics: "Copy diagnostics",
        saveAsWorkflow: "Save as workflow",
        exportWorkflow: "Export workflow",
        loadingDetail: "Loading details…",
        targetTitle: "Target (apply)",
        snapshotTitle: "Rollback snapshot (before apply)",
        errorTitle: "Error",
        resultTitle: "Result",
        llamaArgsExpand: "llamaArgs (expand)",
        notReturned: "(not returned)",
      },
    },
    actions: {
      refresh: "Refresh",
      backfillFromEffective: "Backfill editor from effective config",
    },
    footer: {
      applying: "Applying…",
      applyToSession: "Apply to current session (restart local llama)",
      rollbackLastTitle: "Rollback to last applied config (current session only)",
      rollbackLast: "Rollback last Run",
      setRoleDefault: "Set as role default",
      clearSessionOverride: "Clear session override",
      clearRoleDefault: "Clear role default",
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
    meta: {
      graphSource: "Graph source",
      promptStyleSource: "PromptStyle source",
    },
    permsMissing: {
      title: "Local Llama is missing required permissions",
      hint:
        "Missing: {list}. Without permissions, it may fallback to other LLMs or calls will be blocked.",
      goGrant: "Grant permissions",
    },
    workflows: {
      title: "Workflows (Module 9 packages)",
      nameLabel: "Name",
      namePlaceholder: "Name this workflow…",
      libraryLabel: "Library",
      notSelected: "(not selected)",
      load: "Load",
      saveAsNew: "Save as new workflow",
      overwriteSave: "Overwrite save",
      delete: "Delete",
      exportFile: "Export file",
      importFile: "Import file",
      hint:
        "Tip: workflows save node layout, edges, and parameters; export to share with other creators.",
      toastPickFirst: "Please select a workflow first.",
      toastPickFirstForOverwrite: "Please select a workflow to overwrite.",
      confirmOverwrite: "Overwrite save the selected workflow. Continue?",
      toastSaved: "Workflow saved: {name}",
      toastOverwritten: "Workflow overwritten: {name}",
      toastLoaded: "Workflow loaded: {name}",
      confirmDelete: "Delete workflow: {name}\n\nContinue?",
      toastDeleted: "Workflow deleted.",
      toastExported: "Workflow file exported.",
      dialogImportTitle: "Import workflow (JSON)",
      importDefaultName: "Imported workflow",
      unnamedDefault: "Untitled workflow",
      defaultName: "Workflow",
      toastImportedAndSaved: "Imported and saved to workflow library: {name}",
    },
    editor: {
      label: "Editor",
      canvas: "Canvas (edges)",
      form: "Form",
      canvasHint:
        "Tip: canvas writes node positions and edges into ExpertGraph (used for M2 compilation).",
    },
    canvas: {
      actions: {
        addBase: "+ BaseModel",
        addLora: "+ LoRA",
        addPromptStyle: "+ PromptStyle",
        tidyLayout: "Tidy layout",
        fitView: "Fit view",
        deleteSelectedNode: "Delete selected node",
        deleteSelectedEdge: "Delete selected edge",
        delete: "Delete",
        clearSelection: "Clear selection",
      },
      warnings: {
        missingBase: "Missing BaseModel node (cannot pick base GGUF).",
        multipleBase: "Multiple BaseModels found: compiler will pick one “main Base”.",
        multiplePromptStyle: "Multiple PromptStyles found: compiler will pick one.",
      },
    },
    inspector: {
      title: "Node attributes",
      baseHint: "Base can only select GGUF under `models/gguf/`.",
      pickLora: "(pick a LoRA…)",
      strengthLabel: "Strength (ComfyUI style, default 1.0)",
      enableLora: "Enable this LoRA",
      promptStyleHint:
        "Tip: edits here sync to the PromptStyle draft and take effect as an override layer when applying.",
    },
    promptStyle: {
      replyQualityAnchor: "Reply quality anchor",
      corePersonality: "Core personality",
      description: "Description",
    },
    form: {
      baseTitle: "Base model (GGUF)",
      importBase: "Import GGUF…",
      keepCurrent: "(not set / keep current)",
      baseDirHint: "Folder: `{app_data}/models/gguf/*.gguf`",
      loraTitle: "LoRA (multi-select)",
      importLora: "Import LoRA…",
      addLoraPlaceholder: "Add a LoRA…",
      noLora: "No LoRA added yet.",
      strengthShort: "Strength",
      moveUp: "Up",
      moveDown: "Down",
      remove: "Remove",
      loraDirHint:
        "Folder: `{app_data}/models/loras/*.gguf` (also compatible with placing in the gguf folder)",
      promptStyleTitle: "PromptStyle (optional override)",
      replyQualityAnchorHint: "Reply quality anchor (override role pack/default)",
      corePersonalityHint: "Core personality (override role.core_personality)",
      descriptionHint: "Description (override role.description)",
      emptyMeansNoOverride: "Leave empty to keep unchanged",
      promptStyleFooterHint:
        "Tip: when not set, prompt behavior remains exactly the same as the current version.",
    },
    effective: {
      title: "Effective config (for debugging)",
      hint:
        "This section shows the effective config (session override / role default / role pack default). It is not the same as the draft you're editing.",
      noLoras: "(none / disabled)",
      promptStyleOverridden: "(overridden)",
      promptStyleNotOverridden: "(not overridden)",
    },
    advancedForm: {
      title: "Advanced / compatibility editor (form)",
    },
  },
} as const;

