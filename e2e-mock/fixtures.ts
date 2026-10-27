/**
 * Playwright / vite preview E2E fixtures — in-memory app state mutated by mock invoke handlers.
 */
import type { SessionMeta, StoredMessage } from '../distros/shared/src/api/chatStorage'

const conversations = new Map<string, { meta: SessionMeta, messages: StoredMessage[] }>()
export interface MockRole { id: string, name: string, version: string, author: string }

export const mockRoles: MockRole[] = [
  { id: 'role-a', name: 'Role Alpha', version: '1.0.0', author: 'E2E' },
  { id: 'role-b', name: 'Role Beta', version: '1.0.0', author: 'E2E' },
]

export let currentRoleId = 'role-a'

export const installedPluginIds = new Set<string>(['demo-plugin'])

const defaultPluginBackends = {
  memory: 'builtin',
  emotion: 'builtin',
  event: 'builtin',
  prompt: 'builtin',
  llm: 'ollama',
  agent: 'builtin',
  directory_plugins: {},
}

const defaultPackUi = {
  theme: 'default',
  layout: {},
  slots: {},
}

function roleInfo(roleId: string) {
  const row = mockRoles.find(r => r.id === roleId) ?? mockRoles[0]!
  return {
    role_id: row.id,
    role_name: row.name,
    version: row.version,
    author: row.author,
    description: 'E2E mock role',
    current_favorability: 50,
    current_emotion: 'neutral',
    personality_vector: [],
    scenes: ['default'],
    scene_labels: [{ id: 'default', label: 'Default' }],
    current_scene: 'default',
    user_presence_scene: null,
    virtual_time_ms: 0,
    user_relations: [
      {
        id: 'friend',
        name: 'Friend',
        prompt_hint: '',
        favor_multiplier: 1,
        initial_favorability: 50,
      },
    ],
    default_relation: 'friend',
    current_user_relation: 'friend',
    use_manifest_default: false,
    relation_state: 'Stranger',
    event_impact_factor: 1,
    effective_ollama_model: 'mock',
    identity_binding: 'global',
    remote_life_enabled: false,
    remote_life_pack_default: null,
    interaction_mode: 'immersive',
    interaction_mode_pack_default: null,
    current_life: null,
    plugin_backends: defaultPluginBackends,
    plugin_backends_session_override: null,
    plugin_backends_effective: defaultPluginBackends,
    plugin_backends_effective_sources: {
      memory: 'pack_default',
      emotion: 'pack_default',
      event: 'pack_default',
      prompt: 'pack_default',
      llm: 'pack_default',
      agent: 'pack_default',
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
  }
}

function roleData(roleId: string) {
  const info = roleInfo(roleId)
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
  }
}

function llmUserSettings() {
  const baseModelPath = 'C:\\Models\\mumu-base.gguf'
  return {
    provider: 'local',
    cloudVendor: 'openai_compatible',
    cloudApiStyle: 'openai',
    ollamaBaseUrl: 'http://127.0.0.1:11434',
    ollamaReachable: true,
    ollamaDetail: 'E2E mock Ollama',
    localModelsDir: 'C:\\Models',
    localModelFiles: [
      {
        name: 'mumu-base.gguf',
        path: baseModelPath,
        sizeBytes: 4_294_967_296,
        contentRating: 'general',
        description: 'E2E base-model fixture',
        license: 'Apache-2.0',
        source: null,
        sha256: null,
      },
      {
        name: 'Qwen2.5-7B-Instruct Abliterated v3 Q4_K_M（成人基座）',
        path: 'C:\\Models\\qwen2.5-7b-instruct-abliterated-v3.Q4_K_M.gguf',
        sizeBytes: 4_683_074_560,
        contentRating: 'adult',
        description:
          '独立完整消融基座，可与任意经兼容性验证的 GGUF LoRA 自由组合；“第二套”仅是当前测试称呼，不是固定配置或绑定关系。',
        license: 'Apache-2.0',
        source: 'https://huggingface.co/mradermacher/Qwen2.5-7B-Instruct-abliterated-v3-GGUF',
        sha256: 'fb4821c8707f89b03bd6738c07a382744184a3f15bd6668e6500cb313fbcaa75',
      },
    ],
    localModelPath: baseModelPath,
    localLoraAdapters: [
      {
        id: 'mumu-dialogue',
        name: 'Mumu dialogue',
        version: '1.0.0',
        format: 'gguf',
        contentRating: 'general',
        fileName: 'mumu-dialogue.gguf',
        sizeBytes: 134_217_728,
        sha256: 'e2e-mock-sha256',
        baseModel: 'mumu-base',
        architecture: 'llama',
        description: 'E2E adapter fixture',
        license: 'Apache-2.0',
        source: null,
        installedAt: '2026-07-25T00:00:00Z',
        active: true,
      },
    ],
    activeLocalLoraAdapterId: 'mumu-dialogue',
    localRuntimeMode: 'performance',
    performanceEndpoint: 'http://127.0.0.1:18080',
    performanceRuntimeAvailable: true,
    performanceModelConfigured: true,
    performanceReady: true,
    performanceActiveBackend: 'performance',
    performanceDetail: 'E2E mock llama-server',
    packOllamaModel: null,
    sessionOllamaModel: 'mock',
    effectiveModel: baseModelPath,
    remoteUrl: '',
    remoteTokenConfigured: false,
    remoteModel: '',
    remoteUrlEnvActive: false,
    remoteTokenEnvActive: false,
  }
}

const emptyPluginState = {
  shellPluginId: '',
  disabled_plugins: [] as string[],
  slot_order: {},
  disabled_slot_contributions: {},
}

export function mockInvoke(command: string, payload: Record<string, unknown> = {}): unknown {
  // This module is aliased only by --mode e2e, never by a production build.
  if (typeof localStorage !== 'undefined') {
    const faultKey = `oclive.e2e.fail.${command}`
    const encoded = localStorage.getItem(faultKey)
    if (encoded) {
      const fault = JSON.parse(encoded) as { code: string, message: string, once?: boolean }
      if (fault.once)
        localStorage.removeItem(faultKey)
      throw JSON.stringify({ code: fault.code, message: fault.message })
    }
  }
  switch (command) {
    case 'get_chat_storage_capabilities':
      return { backend_kind: 'hybrid', mirror_enabled: true, default_max_messages_per_session: 500, supports_search: true, supports_replay: false, supports_cleanup: true, supports_chat_threads: true }
    case 'get_chat_storage_stats':
      return []
    case 'list_chat_sessions':
      return [...conversations.values()].map(row => row.meta)
        .filter(row => row.role_id === payload.roleId && row.scene_id === payload.sceneId)
        .sort((a, b) => b.updated_at.localeCompare(a.updated_at))
    case 'fetch_chat_messages':
      return conversations.get(String(payload.sessionId))?.messages ?? []
    case 'list_roles':
      if (typeof localStorage !== 'undefined' && localStorage.getItem('oclive.e2e.emptyRoles') === '1')
        return []
      return mockRoles.map(r => ({ ...r }))
    case 'load_role': {
      const roleId = String(payload.roleId ?? payload.role_id ?? currentRoleId)
      return roleData(roleId)
    }
    case 'get_role_info':
    case 'switch_role': {
      const req = (payload.req ?? payload) as Record<string, unknown>
      const roleId = String(
        payload.roleId ?? req?.role_id ?? currentRoleId,
      )
      if (command === 'switch_role')
        currentRoleId = roleId
      return roleInfo(roleId)
    }
    case 'send_message': {
      const req = payload.req as Record<string, unknown>
      const userMessage = String(req?.user_message ?? '')
      const roleId = String(req.role_id)
      const sessionId = req.chat_thread_id ? `${req.chat_thread_id}:chat:${roleId}` : roleId
      const now = new Date().toISOString()
      const conversation = conversations.get(sessionId) ?? {
        meta: { session_id: sessionId, role_id: roleId, scene_id: String(req.scene_id ?? 'default'), created_at: now, updated_at: now, message_count: 0, last_message_snippet: '' },
        messages: [],
      }
      const userId = crypto.randomUUID()
      const replyId = crypto.randomUUID()
      conversation.messages.push(
        { id: userId, session_id: sessionId, turn_index: conversation.messages.length, sender: 'user', content: userMessage, created_at: now },
        { id: replyId, session_id: sessionId, turn_index: conversation.messages.length, sender: 'assistant', content: `Echo: ${userMessage}`, created_at: now },
      )
      Object.assign(conversation.meta, { updated_at: now, message_count: conversation.messages.length, last_message_snippet: `Echo: ${userMessage}` })
      conversations.set(sessionId, conversation)
      return {
        api_version: 1,
        schema: 1,
        presence_mode: 'co_present',
        relation_state: 'Stranger',
        reply: `Echo: ${userMessage}`,
        emotion: {
          joy: 0,
          sadness: 0,
          anger: 0,
          fear: 0,
          surprise: 0,
          disgust: 0,
        },
        bot_emotion: 'neutral',
        portrait_emotion: 'neutral',
        favorability_delta: 0,
        favorability_current: 50,
        events: [],
        scene_id: String(req?.scene_id ?? 'default'),
        offer_destination_picker: false,
        offer_together_travel: false,
        reply_is_fallback: false,
        knowledge_chunks_in_prompt: 0,
        timestamp: Date.now(),
        user_message_id: userId,
        assistant_message_id: replyId,
        user_message_timestamp: now,
        assistant_message_timestamp: now,
      }
    }
    case 'get_directory_plugin_catalog':
      return [...installedPluginIds].map(id => ({
        id,
        version: '0.0.1',
        hasRpcProcess: false,
        isShell: false,
        uiSlotNames: [],
        provides: [],
        dependencyStatus: 'ok',
        dependencyIssues: [],
      }))
    case 'get_plugin_state':
      return {
        role: { ...emptyPluginState },
        globalDefaults: { ...emptyPluginState },
      }
    case 'get_directory_plugin_bootstrap':
      return {
        shellPluginId: null,
        shellUrl: null,
        forceIframeMode: false,
      }
    case 'install_plugin_from_zip':
      installedPluginIds.add('e2e-local-plugin')
      return {
        installedPluginId: 'e2e-local-plugin',
        installPath: '/mock/plugins/e2e-local-plugin',
      }
    case 'query_events':
    case 'query_memories':
      return []
    case 'reload_policy_plugins':
      return 'ok'
    case 'find_role_asset_path':
      return null
    case 'get_hotkey_bindings':
      return { schemaVersion: 1, bindings: [] }
    case 'list_mcp_servers':
      return []
    case 'get_agent_debug_traces':
      return []
    case 'list_high_risk_grants':
      return { grants: [] }
    case 'get_kernel_connection_status':
    case 'reconnect_kernel':
      return { mode: 'spawned', baseUrl: 'http://127.0.0.1:50000', port: 50000, binaryPath: '/isolated/kernel', kernelTier: 'shared', healthy: true }
    case 'get_kernel_diagnostics':
      return { status: { mode: 'spawned', baseUrl: 'http://127.0.0.1:50000', port: 50000, binaryPath: '/isolated/kernel', kernelTier: 'shared', healthy: true }, sharedRuntimePath: '/isolated/kernel', sharedRuntimeExists: true, sharedRuntimeModifiedMs: 1, healthJson: { healthy: true } }
    case 'read_support_logs':
      return {
        directory: '/isolated/test-data/logs',
        files: [{ name: 'oclive.2026-10-26.log', sizeBytes: 600, modifiedAtEpochMs: 1 }],
        selectedFile: 'oclive.2026-10-26.log',
        truncated: true,
        text: '2026-10-26T12:00:00Z INFO kernel ready\n2026-10-26T12:01:00Z WARN retry scheduled\n2026-10-26T12:02:00Z ERROR model unavailable api_key=fixture-secret',
      }
    case 'run_environment_diagnostics':
      return {
        ollamaReachable: true,
        ollamaBaseUrl: 'http://127.0.0.1:11434',
        ollamaDetail: 'mock',
        rolesDir: '/isolated/roles',
        rolesDirExists: true,
        rolesDirReadable: true,
        appDataDir: '/isolated/test-data',
        appDataWritable: true,
        appDataDetail: 'mock',
      }
    case 'get_time_state':
      return {
        virtual_time_ms: Date.UTC(2026, 6, 25, 20, 0),
        iso_datetime: '2026-07-25T20:00:00.000Z',
      }
    case 'get_global_ollama_model':
      return { model: 'mock' }
    case 'get_llm_user_settings':
      return llmUserSettings()
    case 'list_ollama_models':
      return ['mock']
    case 'get_remote_fallback_app_settings':
      return { remoteFallbackToBuiltin: localStorage.getItem('oclive.e2e.remoteFallback') ?? '1', remoteFallbackEnvOverrideActive: false }
    case 'set_remote_fallback_to_builtin':
      localStorage.setItem('oclive.e2e.remoteFallback', payload.allow ? '1' : '0')
      return null
  }
  return null
}

export function resetE2eMockState(): void {
  conversations.clear()
  currentRoleId = 'role-a'
  installedPluginIds.clear()
  installedPluginIds.add('demo-plugin')
}
