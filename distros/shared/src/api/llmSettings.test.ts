import { beforeEach, describe, expect, it, vi } from 'vitest'

import { invokeWithFriendlyError } from './helpers'
import {
  activateLocalLoraAdapter,
  deleteLocalLoraAdapter,
  importLocalLoraAdapter,
  saveLlmUserSettings,
} from './llmSettings'

vi.mock('./helpers', () => ({
  invokeWithFriendlyError: vi.fn(),
}))

const invokeMock = vi.mocked(invokeWithFriendlyError)

describe('api/llmSettings LoRA IPC contracts', () => {
  beforeEach(() => {
    invokeMock.mockReset()
    invokeMock.mockResolvedValue(null)
  })

  it('keeps structured import data under req', async () => {
    await importLocalLoraAdapter({
      sourcePath: 'D:\\models\\adapter.gguf',
      name: '  Character  ',
      baseModel: '  Qwen2.5-7B-Instruct  ',
      contentRating: 'adult',
      replaceExisting: true,
    })

    expect(invokeMock).toHaveBeenCalledWith('import_local_lora_adapter', {
      req: {
        sourcePath: 'D:\\models\\adapter.gguf',
        name: 'Character',
        baseModel: 'Qwen2.5-7B-Instruct',
        contentRating: 'adult',
        replaceExisting: true,
      },
    })
  })

  it('passes activation scalars as top-level Tauri arguments', async () => {
    await activateLocalLoraAdapter('  local.lora.example  ', true)

    expect(invokeMock).toHaveBeenCalledWith('activate_local_lora_adapter', {
      adapterId: 'local.lora.example',
      adultContentAcknowledged: true,
    })
  })

  it('passes null when deactivating the active adapter', async () => {
    await activateLocalLoraAdapter(null)

    expect(invokeMock).toHaveBeenCalledWith('activate_local_lora_adapter', {
      adapterId: null,
      adultContentAcknowledged: false,
    })
  })

  it('keeps delete data under req', async () => {
    await deleteLocalLoraAdapter('  local.lora.example  ')

    expect(invokeMock).toHaveBeenCalledWith('delete_local_lora_adapter', {
      req: { adapterId: 'local.lora.example' },
    })
  })

  it('carries adult base acknowledgement inside the save request', async () => {
    await saveLlmUserSettings({
      roleId: 'mumu',
      provider: 'local',
      localModelPath: 'D:\\models\\adult-base.gguf',
      adultContentAcknowledged: true,
    })

    expect(invokeMock).toHaveBeenCalledWith('save_llm_user_settings', {
      req: {
        roleId: 'mumu',
        sessionId: null,
        provider: 'local',
        cloudVendor: undefined,
        cloudApiStyle: undefined,
        ollamaBaseUrl: undefined,
        localModelsDir: undefined,
        localModelPath: 'D:\\models\\adult-base.gguf',
        adultContentAcknowledged: true,
        ollamaModel: undefined,
        remoteUrl: undefined,
        remoteToken: undefined,
        remoteModel: undefined,
      },
    })
  })
})
