export default {
  app: {
    locale: {
      label: "Language",
      system: "Match system",
      zhCN: "中文",
      enUS: "English",
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
  },
};
