// @vitest-environment jsdom

import { readFileSync } from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { flushPromises, mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createI18n } from 'vue-i18n'
import CharacterInfo from './CharacterInfo.vue'

const mocks = vi.hoisted(() => ({
  readRoleAssetBytes: vi.fn(),
  resolveRoleAssetPath: vi.fn(),
  convertFileSrc: vi.fn(),
  createObjectURL: vi.fn(),
  revokeObjectURL: vi.fn(),
}))

vi.mock('@oclive/shared/api', () => ({
  readRoleAssetBytes: mocks.readRoleAssetBytes,
  resolveRoleAssetPath: mocks.resolveRoleAssetPath,
}))

vi.mock('@tauri-apps/api/core', () => ({
  convertFileSrc: mocks.convertFileSrc,
}))

function mountCharacter() {
  const i18n = createI18n({
    legacy: false,
    locale: 'zh-CN',
    messages: {
      'zh-CN': {
        emotionUi: { neutral: '平静' },
      },
    },
  })
  return mount(CharacterInfo, {
    props: {
      roleId: 'mumu',
      name: '沐沐',
      emotion: 'neutral',
      portraitAssetRelPath: 'assets/images/normal-v2.png',
    },
    global: { plugins: [i18n] },
  })
}

describe('character info role portrait', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    Object.defineProperty(window, '__TAURI_INTERNALS__', {
      configurable: true,
      value: {},
    })
    mocks.resolveRoleAssetPath.mockResolvedValue('D:\\roles\\mumu\\assets\\images\\normal-v2.png')
    mocks.readRoleAssetBytes.mockResolvedValue([137, 80, 78, 71])
    mocks.createObjectURL.mockReturnValue('blob:role-portrait')
    Object.defineProperties(URL, {
      createObjectURL: { configurable: true, value: mocks.createObjectURL },
      revokeObjectURL: { configurable: true, value: mocks.revokeObjectURL },
    })
  })

  afterEach(() => {
    Reflect.deleteProperty(window, '__TAURI_INTERNALS__')
    Reflect.deleteProperty(URL, 'createObjectURL')
    Reflect.deleteProperty(URL, 'revokeObjectURL')
    vi.restoreAllMocks()
  })

  it('renders catalog bytes through a blob URL in Tauri', async () => {
    const wrapper = mountCharacter()
    await flushPromises()

    expect(mocks.resolveRoleAssetPath).toHaveBeenCalledWith(
      'mumu',
      'assets/images/normal-v2.png',
    )
    expect(mocks.readRoleAssetBytes).toHaveBeenCalledWith(
      'mumu',
      'assets/images/normal-v2.png',
    )
    expect(wrapper.get('img.avatar').attributes('src')).toBe('blob:role-portrait')
    expect(wrapper.find('.avatar-fallback').exists()).toBe(false)

    wrapper.unmount()
    expect(URL.revokeObjectURL).toHaveBeenCalledWith('blob:role-portrait')
  })

  it('loads all seven emotion families and three intensities from the actual Mumu catalog', async () => {
    const wrapper = mountCharacter()
    await flushPromises()
    mocks.resolveRoleAssetPath.mockImplementation(async (_role: string, rel: string) => rel)
    mocks.readRoleAssetBytes.mockImplementation(async (_role: string, rel: string) => {
      return readFileSync(path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../../../chat-pro/roles/mumu', rel))
    })
    let index = 0
    mocks.createObjectURL.mockImplementation(() => `blob:portrait-${++index}`)
    for (const emotion of ['happy', 'sad', 'angry', 'neutral', 'excited', 'confused', 'shy']) {
      for (const intensity of ['mild', 'moderate', 'severe']) {
        const rel = `assets/images/${emotion}_${intensity}.png`
        await wrapper.setProps({ emotion, portraitAssetRelPath: rel })
        await flushPromises()
        expect(mocks.readRoleAssetBytes).toHaveBeenLastCalledWith('mumu', rel)
        expect(wrapper.get('img.avatar').attributes('src')).toBe(`blob:portrait-${index}`)
        expect(wrapper.find('.avatar-fallback').exists()).toBe(false)
        expect(mocks.createObjectURL.mock.lastCall?.[0].size).toBeGreaterThan(1000)
      }
    }
    expect(index).toBe(21)
    wrapper.unmount()
  })

  it('keeps the current portrait until a same-role replacement is ready, and ignores an older completion', async () => {
    const wrapper = mountCharacter()
    await flushPromises()
    let resolveOld!: (bytes: number[]) => void
    mocks.readRoleAssetBytes.mockImplementationOnce(() => new Promise<number[]>((resolve) => {
      resolveOld = resolve
    }))
    await wrapper.setProps({ emotion: 'sad', portraitAssetRelPath: 'assets/images/sad_severe.png' })
    await flushPromises()
    expect(wrapper.get('img.avatar').attributes('src')).toBe('blob:role-portrait')
    expect(mocks.revokeObjectURL).not.toHaveBeenCalled()
    mocks.createObjectURL.mockReturnValue('blob:new-happy')
    await wrapper.setProps({ emotion: 'happy', portraitAssetRelPath: 'assets/images/happy_mild.png' })
    await flushPromises()
    resolveOld([1, 2, 3])
    await flushPromises()
    expect(wrapper.get('img.avatar').attributes('src')).toBe('blob:new-happy')
    expect(mocks.createObjectURL).toHaveBeenCalledTimes(2)
    wrapper.unmount()
  })

  it('invalidates pending requests when unmounted instead of leaking a new blob', async () => {
    let finish!: (bytes: number[]) => void
    mocks.readRoleAssetBytes.mockImplementationOnce(() => new Promise<number[]>((resolve) => {
      finish = resolve
    }))
    const wrapper = mountCharacter()
    await flushPromises()
    wrapper.unmount()
    finish([1, 2, 3])
    await flushPromises()
    expect(mocks.createObjectURL).not.toHaveBeenCalled()
  })

  it('falls back to emotion filenames if the requested catalog asset is missing', async () => {
    mocks.resolveRoleAssetPath.mockImplementation(async (_role: string, rel: string) => rel.includes('missing') ? null : rel)
    const wrapper = mountCharacter()
    await flushPromises()
    await wrapper.setProps({ emotion: 'shy', portraitAssetRelPath: 'assets/images/missing.png' })
    await flushPromises()
    expect(mocks.readRoleAssetBytes).toHaveBeenLastCalledWith('mumu', 'assets/images/shy_mild.png')
    expect(wrapper.find('.avatar-fallback').exists()).toBe(false)
    await wrapper.setProps({ emotion: 'sad' })
    await flushPromises()
    expect(mocks.readRoleAssetBytes).toHaveBeenLastCalledWith('mumu', 'assets/images/sad_mild.png')
    wrapper.unmount()
  })

  it('shows a safe emoji when no assets exist or an image fails to decode', async () => {
    mocks.resolveRoleAssetPath.mockResolvedValue(null)
    const wrapper = mountCharacter()
    await flushPromises()
    expect(wrapper.find('img.avatar').exists()).toBe(false)
    expect(wrapper.get('.avatar-fallback').text()).toBe('😐')
    mocks.resolveRoleAssetPath.mockResolvedValue('normal.png')
    await wrapper.setProps({ emotion: 'shy', portraitAssetRelPath: null })
    await flushPromises()
    await wrapper.get('img.avatar').trigger('error')
    expect(wrapper.find('img.avatar').exists()).toBe(false)
    expect(wrapper.find('.avatar-fallback').exists()).toBe(true)
    wrapper.unmount()
  })

  it('clears the old role immediately and does not reload an unchanged catalog path', async () => {
    const wrapper = mountCharacter()
    await flushPromises()
    await wrapper.setProps({ emotion: 'happy' })
    await flushPromises()
    expect(mocks.readRoleAssetBytes).toHaveBeenCalledTimes(1)
    mocks.resolveRoleAssetPath.mockResolvedValue(null)
    await wrapper.setProps({ roleId: 'other' })
    await flushPromises()
    expect(wrapper.find('img.avatar').exists()).toBe(false)
    expect(mocks.revokeObjectURL).toHaveBeenCalledWith('blob:role-portrait')
    wrapper.unmount()
  })

  it('opts into a cropped library preview without reloading or changing the runtime portrait source', async () => {
    const wrapper = mountCharacter()
    await flushPromises()
    expect(wrapper.find('.hero--preview').exists()).toBe(false)
    await wrapper.setProps({ layout: 'preview' })
    expect(wrapper.get('.hero').attributes('aria-hidden')).toBe('true')
    expect(wrapper.find('.title').exists()).toBe(false)
    expect(wrapper.get('img.avatar').attributes('src')).toBe('blob:role-portrait')
    expect(mocks.readRoleAssetBytes).toHaveBeenCalledTimes(1)
    await wrapper.setProps({ layout: 'stack' })
    expect(wrapper.get('.title').text()).toBe('沐沐')
    expect(wrapper.find('.hero--preview').exists()).toBe(false)
    wrapper.unmount()
  })

  it('uses the role initial when a library preview is missing', async () => {
    mocks.resolveRoleAssetPath.mockResolvedValue(null)
    const wrapper = mountCharacter()
    await wrapper.setProps({ layout: 'preview' })
    await flushPromises()
    expect(wrapper.get('.avatar-fallback').text()).toBe('沐')
    wrapper.unmount()
  })
})
