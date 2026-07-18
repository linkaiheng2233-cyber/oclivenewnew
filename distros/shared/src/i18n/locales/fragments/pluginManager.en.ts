/** pluginManager — en. */
export default {
  pluginManager: {
    legend: {
      enabled: 'Enabled: this path can take effect now',
      pending: 'Needs setup: usually missing a directory plugin ID',
      disabled: 'Disabled: this path is not active',
    },
    source: {
      session_override: 'Session override',
      env_override: 'Environment override',
      pack_default: 'Role pack default',
    },
    risk: {
      needsConfig: 'Needs setup',
      envFirst: 'Env wins',
    },
    nav: {
      explorerAria: 'Filters (workspace style)',
      title: 'Explorer',
      subtitle: 'Filter view',
      rootTooltip: 'UI hierarchy only; not a disk path',
      treeAria: 'Filter tree',
      byModule: 'By module',
      byBackend: 'By backend',
      byStatus: 'By status',
    },
    search: {
      placeholder: 'Search: e.g. remote, emotion, directory',
      empty: 'No matches—try a shorter keyword.',
    },
    detail: {
      readonlyNotice:
        'Read-only: nothing is saved here; change environment variables or the role pack, then reload the app.',
      previewNotice:
        'Preview: “Apply changes” writes to the current session only (does not edit pack settings.json; env wins on conflicts).',
      expand: 'Expand',
      collapse: 'Collapse',
      placeholder: 'Pick a card from the list in the middle.',
    },
    env: { label: 'Environment variables' },
    cards: {
      optionPackDefault: 'Follow pack default ({backend})',
      llmMain: {
        title: 'Reply engine',
        description: 'Choose whether replies come from local models, a remote service, or a directory plugin.',
      },
      llmEndpoint: {
        title: 'LLM remote endpoints',
        description: 'When remote is selected, the LLM-specific URL is preferred.',
        summary: 'Configure URLs in environment variables rather than hard-coding them in the pack.',
        fieldLlmUrl: 'Dedicated LLM remote URL (preferred)',
        fieldPluginUrl: 'Generic remote URL (fallback)',
      },
      emotionMain: {
        title: 'Emotion engine',
        description: 'Choose built-in logic, remote service, or a directory plugin for user emotion analysis.',
      },
      emotionEndpoint: {
        title: 'Emotion remote endpoint',
        description: 'Remote emotion calls use the generic remote plugin URL by default.',
        summary: 'Put endpoints in environment variables instead of baking them into the pack.',
        fieldPluginUrl: 'Common remote entry for emotion',
      },
      complexSwitch: {
        title: 'Complex emotion toggle',
        description: 'Turns on the remote “inner voice” path for richer complex emotion.',
        sessionOn: 'On for this session',
        sessionOff: 'Off for this session',
        label: 'Enable complex emotion (remote inner voice)',
        hint: 'When enabled, configure URL and TOKEN environment variables.',
      },
      complexEndpoint: {
        title: 'Complex emotion endpoints',
        description: 'Services are often deployed separately and may require a token.',
        summary: 'If auth is required, set both URL and TOKEN.',
        fieldUrl: 'Complex emotion service URL',
        fieldToken: 'Complex emotion auth token',
      },
    },
    apply: {
      endpointNoSave: 'Read-only card—configure environment variables instead.',
      remoteLifeUpdated: 'Complex emotion toggle updated.',
      unsupported: 'Only LLM / Emotion session writes are supported right now.',
      sessionSaved: 'Saved to the current session.',
    },
    installed: {
      privateSettings: 'Plugin private settings',
      advanced: 'Advanced',
      debugWorkbench: 'Debug console',
    },
    legendAria: 'Status legend',
    v1ListItem: {
      aria: 'Plugin {id}',
      kindShell: 'Shell',
      kindSlot: 'Slots',
      uiSlots: 'UI slots: {list}',
      depsUnmet: 'Dependencies not satisfied ({status}): {issues}',
      disablePlugin: 'Disable plugin',
      hideToolbarEmbed: 'Hide toolbar embed',
      hideSettingsEmbed: 'Hide settings embed',
      hideRoleDetailEmbed: 'Hide role detail embed',
      hideSidebarEmbed: 'Hide sidebar embed',
      hideChatHeaderEmbed: 'Hide chat header embed',
    },
    v1Backend: {
      leadBefore: 'Below is the pack default and session override for ',
      leadAfter: '; changes here are not written to the role pack on disk.',
      leadPath: 'settings.json → plugin_backends',
      moduleLine:
        'Module backends: mem {mem} · emotion {emotion} · event {event} · prompt {prompt} · llm {llm} · agent {agent}',
      sessionEffectiveLine:
        'Session effective: mem {mem} · emotion {emotion} · event {event} · prompt {prompt} · llm {llm} · agent {agent}',
      sessionOverrideHint:
        'Session module overrides are enabled (this session only; not written to the pack).',
      sourcesLine:
        'Sources: mem {mem} · emotion {emotion} · event {event} · prompt {prompt} · llm {llm} · agent {agent}',
      titleModule: 'settings.json → plugin_backends',
      titleSession: 'Session effective',
      titleSources: 'Sources',
      remoteLife: 'Remote inner voice',
      packDefaultSuggestOn: 'Pack default suggests on',
      followPackDefault: 'Follow pack default ({value})',
      localMemPlaceholder: 'provider_id; empty clears this session override',
      applySession: 'Apply to this session',
      debugSnapshot: 'Debug snapshot',
      refresh: 'Refresh',
      copy: 'Copy',
      packPlugin: 'Pack plugin',
      oneClickPack: 'One-click pack (agent/llm)',
      copyOk: 'Copied',
      copyFail: 'Copy failed',
      packNeedTarget: 'Configure a directory plugin in the agent or llm slot first.',
      packDone: 'Pack complete: {path} (sha256={sha}…)',
      directoryPluginsPack: 'Pack · directory_plugins: {summary}',
      directoryPluginsEffective: 'Effective · directory_plugins: {summary}',
    },
    slotsAria: {
      settingsPanelTablist: 'Plugin settings tabs',
      settingsPanelEmpty: 'No plugins declare the {slot} slot yet.',
      sidebarList: 'Sidebar plugin slots',
      roleDetailList: 'Role detail plugin slots',
      chatHeaderStrip: 'Chat header plugin slots',
      chatToolbar: 'Plugin toolbar',
    },
    v2: {
      slotKey: 'Instance key',
      packBackend: 'Pack default backend',
    },
    pmSlot: {
      appearance: 'Appearance',
      defaultVariant: 'Default',
      hideSlot: 'Hide this slot',
    },
    template: {
      directoryIdPh: 'e.g. my-plugin-id',
      directoryManualPick: 'Type manually or pick a candidate',
    },
    slotEmbed: {
      defaultAria: 'Plugin embed area',
    },
    errorPlaceholder: {
      retry: 'Load failed — tap to retry',
      fallback: 'Use HTML version',
      viewDetails: 'View details',
    },
    privateSettings: {
      loading: 'Loading settings…',
      noFields: 'This plugin does not declare uiSchema.fields.',
      templatePrefix: 'Template:',
      saving: 'Saving…',
      save: 'Save private settings',
      toastSaved: 'Private plugin settings saved.',
    },
  },
  pluginTerms: {
    module: {
      llm: 'Dialogue brain (LLM)',
      emotion: 'Emotion engine',
      complex_emotion: 'Complex emotion',
    },
    category: {
      all: 'All features',
      module: 'By module',
      type: 'By implementation',
      status: 'By status',
    },
    type: {
      builtin: 'Built-in',
      remote: 'Remote',
      directory: 'Directory plugin',
    },
    status: {
      enabled: 'Enabled',
      disabled: 'Disabled',
      needs_config: 'Needs setup',
    },
    backend: {
      follow_default: 'Follow role pack default',
      ollama: 'Ollama (local model)',
      remote: 'Remote service',
      directory: 'Directory plugin',
      builtin: 'Built-in',
    },
    field: {
      backend: 'Backend mode',
      directory_plugin: 'Directory plugin ID',
      remote_life: 'Remote inner voice',
    },
    hint: {
      directory_id_empty: 'Clearing the field removes the session override and returns to the pack default.',
      endpoint_env: 'Prefer environment variables for endpoints to ease migration and troubleshooting.',
    },
    action: {
      apply: 'Apply changes',
      open_v1: 'Open advanced mode (V1)',
      close: 'Close',
    },
    title: { v2: 'Plugins & backends (simple mode)' },
    subtitle: {
      v2:
        'Lightweight overview from slot_registry instances and session overrides. Full add/remove, groups, and pack writes: V1 advanced mode → Architecture graph (Ctrl+Shift+F).',
    },
  },
}
