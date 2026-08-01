import type { ComposerTranslation } from 'vue-i18n'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { installPendingProtocolPlugins } from '../composables/useAppBootstrap'

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
    vi.clearAllMocks()
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
