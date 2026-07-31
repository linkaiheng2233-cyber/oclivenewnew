import { beforeEach, describe, expect, it, vi } from 'vitest'
import {
  getTheaterCustomLeadCast,
  getTheaterPokeMode,
  getTheaterVariantSwipeEnabled,
  setTheaterCustomLeadCast,
  setTheaterPokeMode,
  setTheaterVariantSwipeEnabled,
} from './useTheaterPokeSettings'

describe('useTheaterPokeSettings', () => {
  beforeEach(() => {
    vi.stubGlobal('localStorage', {
      store: {} as Record<string, string>,
      getItem(key: string) {
        return this.store[key] ?? null
      },
      setItem(key: string, value: string) {
        this.store[key] = value
      },
      clear() {
        this.store = {}
      },
    })
  })

  it('defaults to patch mode and variant swipe on', () => {
    expect(getTheaterPokeMode()).toBe('patch')
    expect(getTheaterVariantSwipeEnabled()).toBe(true)
    expect(getTheaterCustomLeadCast()).toBe('a')
  })

  it('persists poke mode and custom lead cast', () => {
    setTheaterPokeMode('ripple')
    setTheaterVariantSwipeEnabled(false)
    setTheaterCustomLeadCast('b')
    expect(getTheaterPokeMode()).toBe('ripple')
    expect(getTheaterVariantSwipeEnabled()).toBe(false)
    expect(getTheaterCustomLeadCast()).toBe('b')
  })
})
