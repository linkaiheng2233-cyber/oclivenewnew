/**
 * Playwright / vite preview E2E fixtures — in-memory app state mutated by mock invoke handlers.
 */
export type MockRole = { id: string; name: string; version: string; author: string };

export const mockRoles: MockRole[] = [
  { id: "role-a", name: "Role Alpha", version: "1.0.0", author: "E2E" },
  { id: "role-b", name: "Role Beta", version: "1.0.0", author: "E2E" },
];

export let currentRoleId = "role-a";

export const installedPluginIds = new Set<string>(["demo-plugin"]);

const defaultPluginBackends = {
  memory: "builtin",
  emotion: "builtin",
  event: "builtin",
  prompt: "builtin",
  llm: "ollama",
  agent: "builtin",
  directory_plugins: {},
};

const defaultPackUi = {
  theme: "default",
  layout: {},
  slots: {},
};

function roleInfo(roleId: string) {
  const row = mockRoles.find((r) => r.id === roleId) ?? mockRoles[0]!;
  return {
    role_id: row.id,
    role_name: row.name,
    version: row.version,
    author: row.author,
    description: "E2E mock role",
    current_favorability: 50,
    current_emotion: "neutral",
    personality_vector: [],
    scenes: ["default"],
    scene_labels: [{ id: "default", label: "Default" }],
    current_scene: "default",
    user_presence_scene: null,
    virtual_time_ms: 0,
    user_relations: [
      {
        id: "friend",
        name: "Friend",
        prompt_hint: "",
        favor_multiplier: 1,
        initial_favorability: 50,
      },
    ],
    default_relation: "friend",
    current_user_relation: "friend",
    use_manifest_default: false,
    relation_state: "Stranger",
    event_impact_factor: 1,
    effective_ollama_model: "mock",
    identity_binding: "global",
    remote_life_enabled: false,
    remote_life_pack_default: null,
    interaction_mode: "immersive",
    interaction_mode_pack_default: null,
    current_life: null,
    plugin_backends: defaultPluginBackends,
    plugin_backends_session_override: null,
    plugin_backends_effective: defaultPluginBackends,
    plugin_backends_effective_sources: {
      memory: "pack_default",
      emotion: "pack_default",
      event: "pack_default",
      prompt: "pack_default",
      llm: "pack_default",
      agent: "pack_default",
    },
    knowledge_enabled: false,
    knowledge_chunk_count: 0,
    pack_ui_config: defaultPackUi,
    pack_ui_baseline: defaultPackUi,
    author_pack: null,
    slot_registry_pack: null,
    slot_registry_effective: null,
    slot_session_overridden_keys: [],
    blueprint_groups_pack: null,
    dual_core_enabled: false,
    pipeline_experimental_actions: [],
  };
}

function roleData(roleId: string) {
  const info = roleInfo(roleId);
  return {
    role_id: info.role_id,
    name: info.role_name,
    version: info.version,
    author: info.author,
    description: info.description,
    personality_vector: [],
    current_favorability: info.current_favorability,
    current_emotion: info.current_emotion,
    memory_count: 0,
    event_count: 0,
    user_relations: info.user_relations,
    default_relation: info.default_relation,
    relation_state: info.relation_state,
    current_user_relation: info.current_user_relation,
    use_manifest_default: info.use_manifest_default,
    remote_life_enabled: false,
    remote_life_pack_default: null,
    event_impact_factor: 1,
    effective_ollama_model: info.effective_ollama_model,
    identity_binding: info.identity_binding,
    interaction_mode: info.interaction_mode,
    interaction_mode_pack_default: null,
    current_life: null,
    plugin_backends: defaultPluginBackends,
    pack_ui_config: defaultPackUi,
  };
}

const emptyPluginState = {
  shellPluginId: "",
  disabled_plugins: [] as string[],
  slot_order: {},
  disabled_slot_contributions: {},
};

export function mockInvoke(command: string, payload: Record<string, unknown> = {}): unknown {
  switch (command) {
    case "list_roles":
      return mockRoles.map((r) => ({ ...r }));
    case "load_role": {
      const roleId = String(payload.roleId ?? payload.role_id ?? currentRoleId);
      return roleData(roleId);
    }
    case "get_role_info":
    case "switch_role": {
      const req = (payload.req ?? payload) as Record<string, unknown>;
      const roleId = String(
        payload.roleId ?? req?.role_id ?? currentRoleId,
      );
      if (command === "switch_role") currentRoleId = roleId;
      return roleInfo(roleId);
    }
    case "send_message": {
      const req = payload.req as Record<string, unknown>;
      const userMessage = String(req?.user_message ?? "");
      return {
        api_version: 1,
        schema: 1,
        presence_mode: "co_present",
        relation_state: "Stranger",
        reply: `Echo: ${userMessage}`,
        emotion: {
          joy: 0,
          sadness: 0,
          anger: 0,
          fear: 0,
          surprise: 0,
          disgust: 0,
        },
        bot_emotion: "neutral",
        portrait_emotion: "neutral",
        favorability_delta: 0,
        favorability_current: 50,
        events: [],
        scene_id: String(req?.scene_id ?? "default"),
        offer_destination_picker: false,
        offer_together_travel: false,
        reply_is_fallback: false,
        knowledge_chunks_in_prompt: 0,
        timestamp: Date.now(),
      };
    }
    case "get_directory_plugin_catalog":
      return [...installedPluginIds].map((id) => ({
        id,
        version: "0.0.1",
        hasRpcProcess: false,
        isShell: false,
        uiSlotNames: [],
        provides: [],
        dependencyStatus: "ok",
        dependencyIssues: [],
      }));
    case "get_plugin_state":
      return {
        role: { ...emptyPluginState },
        globalDefaults: { ...emptyPluginState },
      };
    case "get_directory_plugin_bootstrap":
      return {
        shellPluginId: null,
        shellUrl: null,
        forceIframeMode: false,
      };
    case "install_plugin_from_zip":
      installedPluginIds.add("e2e-local-plugin");
      return "e2e-local-plugin";
    case "query_events":
    case "query_memories":
      return [];
    case "reload_policy_plugins":
      return "ok";
    case "resolve_role_asset_path":
      return null;
    case "get_hotkey_bindings":
      return { bindings: {} };
    case "list_mcp_servers":
      return [];
    case "get_agent_debug_traces":
      return [];
    case "list_high_risk_grants":
      return { grants: [] };
    case "run_environment_diagnostics":
      return {
        ollama_reachable: true,
        ollama_base_url: "http://127.0.0.1:11434",
        ollama_detail: "mock",
        roles_root_exists: true,
        roles_root_readable: true,
        roles_root_detail: "mock",
        app_data_writable: true,
        app_data_detail: "mock",
      };
    case "get_remote_fallback_app_settings":
      return { allow_remote_fallback: false };
  }
  return null;
}

export function resetE2eMockState(): void {
  currentRoleId = "role-a";
  installedPluginIds.clear();
  installedPluginIds.add("demo-plugin");
}
