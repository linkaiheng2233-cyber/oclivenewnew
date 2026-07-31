// @vitest-environment jsdom
import { afterEach, describe, expect, it } from 'vitest'

const SKIN_KEY = 'oclive-runtime-skin'
const UNLOCK_KEY = 'oclive-easteregg-unlocked'

describe('win98 skin storage contract', () => {
  afterEach(() => {
    localStorage.removeItem(SKIN_KEY)
    localStorage.removeItem(UNLOCK_KEY)
    document.documentElement.removeAttribute('data-skin')
  })

  it('persists win98 skin flag in localStorage', () => {
    localStorage.setItem(SKIN_KEY, 'win98')
    expect(localStorage.getItem(SKIN_KEY)).toBe('win98')
    document.documentElement.setAttribute('data-skin', 'win98')
    expect(document.documentElement.getAttribute('data-skin')).toBe('win98')
  })

  it('konami unlock flag is separate from skin enabled', () => {
    localStorage.setItem(UNLOCK_KEY, '1')
    expect(localStorage.getItem(UNLOCK_KEY)).toBe('1')
    expect(localStorage.getItem(SKIN_KEY)).toBeNull()
  })
})
