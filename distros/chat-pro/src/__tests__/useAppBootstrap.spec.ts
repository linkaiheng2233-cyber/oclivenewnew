import type { ComposerTranslation } from 'vue-i18n'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createProtocolInstallQueue, installPendingProtocolPlugins } from '../composables/useAppBootstrap'

const {
  consumePendingProtocolInstalls,
  installPluginFromGit,
  showPluginInstallReviewHint,
} = vi.hoisted(() => ({
  consumePendingProtocolInstalls: vi.fn(),
  installPluginFromGit: vi.fn(),
  showPluginInstallReviewHint: vi.fn(),
}))

vi.mock('@oclive/shared/api', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@oclive/shared/api')>()
  return {
    ...actual,
    consumePendingProtocolInstalls,
    installPluginFromGit,
  }
})

vi.mock('@oclive/shared/composables/usePluginInstallReviewHint', () => ({
  showPluginInstallReviewHint,
}))

describe('pending protocol plugin installation', () => {
  beforeEach(() => {
    vi.resetAllMocks()
  })

  it.each(['consume', 'first-install'])('keeps remaining queued installs when advanced closes during %s', async (stage) => {
    let enabled = true
    let finishConsume!: (value: Array<{ gitUrl: string }>) => void
    let finishInstall!: (value: { installedPluginId: string }) => void
    const items = [{ gitUrl: 'https://example.test/one.git' }, { gitUrl: 'https://example.test/two.git' }]
    consumePendingProtocolInstalls.mockImplementationOnce(() => stage === 'consume'
      ? new Promise((resolve) => { finishConsume = resolve })
      : Promise.resolve(items))
    consumePendingProtocolInstalls.mockResolvedValue([])
    installPluginFromGit.mockImplementationOnce(() => stage === 'first-install'
      ? new Promise((resolve) => { finishInstall = resolve })
      : Promise.resolve({ installedPluginId: 'one' }))
    installPluginFromGit.mockResolvedValue({ installedPluginId: 'two' })
    const options = {
      queue: createProtocolInstallQueue(),
      advancedFeaturesEnabled: () => enabled,
      showToast: vi.fn(),
      t: vi.fn((key: string) => key) as unknown as ComposerTranslation,
      refreshPlugins: vi.fn(async () => {}),
      openPluginManagerPanel: vi.fn(),
    }
    const pending = installPendingProtocolPlugins(options)
    await vi.waitFor(() => expect(stage === 'consume' ? finishConsume : finishInstall).toBeDefined())
    enabled = false
    if (stage === 'consume')
      finishConsume(items)
    else
      finishInstall({ installedPluginId: 'one' })
    await pending
    expect(installPluginFromGit).toHaveBeenCalledTimes(stage === 'consume' ? 0 : 1)
    expect(options.openPluginManagerPanel).not.toHaveBeenCalled()
    enabled = true
    await installPendingProtocolPlugins(options)
    expect(installPluginFromGit.mock.calls.map(call => call[0])).toEqual(items.map(item => item.gitUrl))
  })

  it('leaves pending installs unconsumed while advanced features are off', async () => {
    await installPendingProtocolPlugins({
      advancedFeaturesEnabled: () => false,
      showToast: vi.fn(),
      t: vi.fn((key: string) => key) as unknown as ComposerTranslation,
      refreshPlugins: vi.fn(async () => {}),
      openPluginManagerPanel: vi.fn(),
    })
    expect(consumePendingProtocolInstalls).not.toHaveBeenCalled()
    expect(installPluginFromGit).not.toHaveBeenCalled()
  })

  it('drains the saved backlog and new protocol requests together after reopening', async () => {
    const queue = createProtocolInstallQueue()
    queue.pending.push({ gitUrl: 'https://example.test/backlog.git' } as never)
    consumePendingProtocolInstalls.mockResolvedValue([{ gitUrl: 'https://example.test/new.git' }])
    installPluginFromGit.mockResolvedValue({ installedPluginId: 'fixture' })
    await installPendingProtocolPlugins({
      queue,
      advancedFeaturesEnabled: () => true,
      showToast: vi.fn(),
      t: vi.fn((key: string) => key) as unknown as ComposerTranslation,
      refreshPlugins: vi.fn(async () => {}),
      openPluginManagerPanel: vi.fn(),
    })
    expect(installPluginFromGit.mock.calls.map(call => call[0])).toEqual([
      'https://example.test/backlog.git',
      'https://example.test/new.git',
    ])
  })

  it('installs, displays the review hint, refreshes, and opens the manager', async () => {
    consumePendingProtocolInstalls.mockResolvedValue([
      { gitUrl: ' https://github.com/example/plugin.git ' },
    ])
    installPluginFromGit.mockResolvedValue({
      installedPluginId: 'com.example.plugin',
      installPath: 'D:/plugins/com.example.plugin',
    })
    const showToast = vi.fn()
    const refreshPlugins = vi.fn(async () => {})
    const openPluginManagerPanel = vi.fn()
    const t = vi.fn((key: string) => key) as unknown as ComposerTranslation

    await installPendingProtocolPlugins({
      showToast,
      t,
      refreshPlugins,
      openPluginManagerPanel,
    })

    expect(installPluginFromGit).toHaveBeenCalledWith(
      'https://github.com/example/plugin.git',
    )
    expect(showPluginInstallReviewHint).toHaveBeenCalledOnce()
    expect(refreshPlugins).toHaveBeenCalledOnce()
    expect(openPluginManagerPanel).toHaveBeenCalledOnce()
  })
})
