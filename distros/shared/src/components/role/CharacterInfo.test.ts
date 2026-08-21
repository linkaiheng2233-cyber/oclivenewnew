// @vitest-environment jsdom

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
})
