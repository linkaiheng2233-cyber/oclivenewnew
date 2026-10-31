import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useRoleStore } from '../stores/roleStore'
import { useRolePackImport } from './useRolePackImport'

const mocks = vi.hoisted(() => ({ open: vi.fn(), peek: vi.fn(), install: vi.fn(), listen: vi.fn(), unlisten: vi.fn() }))
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: mocks.open }))
vi.mock('@tauri-apps/api/event', () => ({ listen: mocks.listen }))
vi.mock('vue-i18n', async original => ({
  ...await original<typeof import('vue-i18n')>(),
  useI18n: () => ({ t: (key: string) => key }),
}))
vi.mock('@oclive/shared/api', async original => ({
  ...await original<typeof import('@oclive/shared/api')>(),
  peekRolePack: mocks.peek,
  importRolePack: mocks.install,
}))

describe('role package import integration', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
    mocks.listen.mockResolvedValue(mocks.unlisten)
    mocks.peek.mockResolvedValue({ id: 'new-role', name: 'New role', version: '1' })
    mocks.install.mockResolvedValue('new-role')
    vi.spyOn(useRoleStore(), 'loadRoles').mockResolvedValue(undefined)
  })

  it('cancels source selection without reading or installing a package', async () => {
    mocks.open.mockResolvedValue(null)
    const imported = vi.fn()
    const flow = useRolePackImport({ onImported: imported })
    await flow.runImportWithPicker('archive')
    expect(mocks.peek).not.toHaveBeenCalled()
    expect(mocks.install).not.toHaveBeenCalled()
    expect(imported).not.toHaveBeenCalled()
    expect(flow.importProgressOpen.value).toBe(false)
  })

  it('requires confirmation for an existing ID and leaves the old package untouched on cancel', async () => {
    useRoleStore().roles = [{ id: 'new-role', name: 'Existing role' }]
    const flow = useRolePackImport()
    await flow.runImportFlow('candidate.ocpak')
    expect(flow.conflictOpen.value).toBe(true)
    expect(mocks.install).not.toHaveBeenCalled()
    flow.closeConflict()
    expect(mocks.install).not.toHaveBeenCalled()
    expect(useRoleStore().roles[0]?.name).toBe('Existing role')
  })

  it('refreshes the real library and activates only the returned imported role', async () => {
    const imported = vi.fn()
    const flow = useRolePackImport({ onImported: imported })
    await flow.runImportFlow('candidate.ocpak')
    expect(mocks.install).toHaveBeenCalledWith('candidate.ocpak', false)
    expect(useRoleStore().loadRoles).toHaveBeenCalled()
    expect(imported).toHaveBeenCalledWith('new-role')
    expect(flow.importProgressOpen.value).toBe(false)
    expect(mocks.unlisten).toHaveBeenCalledTimes(1)
  })

  it('recovers its progress UI if event registration fails before installation', async () => {
    mocks.listen.mockRejectedValueOnce(new Error('event bridge unavailable'))
    const notify = vi.fn()
    mocks.open.mockResolvedValue('candidate.ocpak')
    const flow = useRolePackImport({ onNotify: notify })
    await flow.runImportWithPicker('archive')
    expect(flow.importProgressOpen.value).toBe(false)
    expect(mocks.install).not.toHaveBeenCalled()
    expect(notify).toHaveBeenCalledWith({ type: 'error', message: 'event bridge unavailable', details: '' })
  })

  it('does not claim that an imported package failed when only library refresh failed', async () => {
    vi.mocked(useRoleStore().loadRoles).mockRejectedValueOnce(new Error('library refresh offline'))
    const notify = vi.fn()
    const flow = useRolePackImport({ onNotify: notify })
    await flow.runImportFlow('candidate.ocpak')
    expect(mocks.install).toHaveBeenCalledOnce()
    expect(notify).toHaveBeenCalledWith({ type: 'warning', message: 'errorUi.importRefreshFailed', details: 'library refresh offline' })
    expect(flow.importProgressOpen.value).toBe(false)
  })
})
