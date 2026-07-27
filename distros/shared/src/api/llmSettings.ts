import type { RoleInfo } from './role'
import { invokeWithFriendlyError } from './helpers'

export interface LocalModelFile {
  name: string
  path: string
  sizeBytes: number
  contentRating: LocalContentRating
  description: string | null
  license: string | null
  source: string | null
  sha256: string | null
}

export type LocalContentRating = 'general' | 'adult'
export type LoraContentRating = LocalContentRating

export interface LocalLoraAdapter {
  id: string
  name: string
  version: string
  format: string
  contentRating: LoraContentRating
  fileName: string
  sizeBytes: number
  sha256: string
  baseModel: string | null
  architecture: string | null
  description: string | null
  license: string | null
  source: string | null
  installedAt: string
  active: boolean
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
  localLoraAdapters: LocalLoraAdapter[]
  activeLocalLoraAdapterId: string | null
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
  adultContentAcknowledged?: boolean
  ollamaModel?: string
  remoteUrl?: string
  remoteToken?: string
  remoteModel?: string
}

export async function importLocalLoraAdapter(opts: {
  sourcePath: string
  name?: string
  baseModel?: string
  contentRating: LoraContentRating
  replaceExisting?: boolean
}): Promise<LocalLoraAdapter> {
  return invokeWithFriendlyError<LocalLoraAdapter>('import_local_lora_adapter', {
    req: {
      sourcePath: opts.sourcePath,
      name: opts.name?.trim() || null,
      baseModel: opts.baseModel?.trim() || null,
      contentRating: opts.contentRating,
      replaceExisting: opts.replaceExisting ?? false,
    },
  })
}

export async function activateLocalLoraAdapter(
  adapterId: string | null,
  adultContentAcknowledged = false,
): Promise<LocalLoraAdapter | null> {
  return invokeWithFriendlyError<LocalLoraAdapter | null>('activate_local_lora_adapter', {
    adapterId: adapterId?.trim() || null,
    adultContentAcknowledged,
  })
}

export async function deleteLocalLoraAdapter(adapterId: string): Promise<void> {
  return invokeWithFriendlyError<void>('delete_local_lora_adapter', {
    req: { adapterId: adapterId.trim() },
  })
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
      adultContentAcknowledged: req.adultContentAcknowledged ?? false,
      ollamaModel: req.ollamaModel,
      remoteUrl: req.remoteUrl,
      remoteToken: req.remoteToken,
      remoteModel: req.remoteModel,
    },
  })
}
