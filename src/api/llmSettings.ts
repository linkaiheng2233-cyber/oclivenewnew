import { invokeWithFriendlyError } from './helpers'
import type { RoleInfo } from './role'

export interface LocalModelFile {
  name: string
  path: string
  sizeBytes: number
}

export interface LlmUserSettings {
  provider: 'local' | 'cloud' | string
  cloudVendor: string
  cloudApiStyle: string
  ollamaBaseUrl: string
  ollamaReachable: boolean
  ollamaDetail: string
  localModelsDir: string
  localModelFiles: LocalModelFile[]
  packOllamaModel: string | null
  sessionOllamaModel: string | null
  effectiveModel: string
  remoteUrl: string
  remoteTokenConfigured: boolean
  remoteModel: string
  remoteUrlEnvActive: boolean
  remoteTokenEnvActive: boolean
}

export interface SaveLlmUserSettingsRequest {
  roleId: string
  sessionId?: string | null
  provider: 'local' | 'cloud'
  cloudVendor?: string
  cloudApiStyle?: 'openai' | 'oclive_jsonrpc'
  ollamaBaseUrl?: string
  localModelsDir?: string
  ollamaModel?: string
  remoteUrl?: string
  remoteToken?: string
  remoteModel?: string
}

export async function getLlmUserSettings(
  roleId: string,
  sessionId?: string | null,
): Promise<LlmUserSettings> {
  return invokeWithFriendlyError<LlmUserSettings>('get_llm_user_settings', {
    roleId,
    sessionId: sessionId ?? null,
  })
}

export async function listOllamaModels(ollamaBaseUrl?: string): Promise<string[]> {
  return invokeWithFriendlyError<string[]>('list_ollama_models', {
    ollamaBaseUrl: ollamaBaseUrl?.trim() || null,
  })
}

export async function scanLocalModelFiles(directory?: string): Promise<LocalModelFile[]> {
  return invokeWithFriendlyError<LocalModelFile[]>('scan_local_model_files', {
    directory: directory?.trim() || null,
  })
}

export async function openPathInFileManager(path: string): Promise<void> {
  return invokeWithFriendlyError<void>('open_path_in_file_manager', { path })
}

export async function importGgufToOllama(opts: {
  filePath: string
  modelName?: string
  ollamaBaseUrl?: string
}): Promise<string> {
  return invokeWithFriendlyError<string>('import_gguf_to_ollama', {
    req: {
      file_path: opts.filePath,
      model_name: opts.modelName ?? null,
      ollama_base_url: opts.ollamaBaseUrl ?? null,
    },
  })
}

export async function saveLlmUserSettings(
  req: SaveLlmUserSettingsRequest,
): Promise<RoleInfo> {
  return invokeWithFriendlyError<RoleInfo>('save_llm_user_settings', {
    req: {
      role_id: req.roleId,
      session_id: req.sessionId ?? null,
      provider: req.provider,
      cloud_vendor: req.cloudVendor,
      cloud_api_style: req.cloudApiStyle,
      ollama_base_url: req.ollamaBaseUrl,
      local_models_dir: req.localModelsDir,
      ollama_model: req.ollamaModel,
      remote_url: req.remoteUrl,
      remote_token: req.remoteToken,
      remote_model: req.remoteModel,
    },
  })
}
