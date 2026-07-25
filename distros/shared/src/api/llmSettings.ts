import type { RoleInfo } from './role'
import { invokeWithFriendlyError } from './helpers'

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
  localModelPath: string
  localRuntimeMode: 'performance' | 'ollama' | string
  performanceEndpoint: string
  performanceRuntimeAvailable: boolean
  performanceModelConfigured: boolean
  performanceReady: boolean
  performanceActiveBackend: 'performance' | 'ollama' | 'pending' | string
  performanceDetail: string
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
  localModelPath?: string
  ollamaModel?: string
  remoteUrl?: string
  remoteToken?: string
  remoteModel?: string
}

export async function getGlobalOllamaModel(): Promise<{ model: string }> {
  return invokeWithFriendlyError<{ model: string }>('get_global_ollama_model')
}

export async function setGlobalOllamaModel(
  model: string,
  roleId?: string | null,
): Promise<{ model: string }> {
  return invokeWithFriendlyError<{ model: string }>('set_global_ollama_model', {
    req: { model: model.trim(), roleId: roleId?.trim() || null },
  })
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

export async function listCloudModels(opts?: {
  remoteUrl?: string
  remoteToken?: string
}): Promise<string[]> {
  return invokeWithFriendlyError<string[]>('list_cloud_models', {
    remoteUrl: opts?.remoteUrl?.trim() || null,
    remoteToken: opts?.remoteToken?.trim() || null,
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
      filePath: opts.filePath,
      modelName: opts.modelName ?? null,
      ollamaBaseUrl: opts.ollamaBaseUrl ?? null,
    },
  })
}

export async function probeCloudLlm(
  roleId: string,
  sessionId?: string | null,
): Promise<string> {
  return invokeWithFriendlyError<string>('probe_cloud_llm', {
    roleId,
    sessionId: sessionId ?? null,
  })
}

export async function saveLlmUserSettings(
  req: SaveLlmUserSettingsRequest,
): Promise<RoleInfo> {
  return invokeWithFriendlyError<RoleInfo>('save_llm_user_settings', {
    req: {
      roleId: req.roleId,
      sessionId: req.sessionId ?? null,
      provider: req.provider,
      cloudVendor: req.cloudVendor,
      cloudApiStyle: req.cloudApiStyle,
      ollamaBaseUrl: req.ollamaBaseUrl,
      localModelsDir: req.localModelsDir,
      localModelPath: req.localModelPath,
      ollamaModel: req.ollamaModel,
      remoteUrl: req.remoteUrl,
      remoteToken: req.remoteToken,
      remoteModel: req.remoteModel,
    },
  })
}
