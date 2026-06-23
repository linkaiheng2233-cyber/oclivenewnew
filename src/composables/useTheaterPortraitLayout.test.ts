// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import {
  getTheaterPortraitLayout,
  resetTheaterPortraitLayout,
  setTheaterPortraitMaxHeight,
  setTheaterPortraitWidth,
  THEATER_PORTRAIT_DEFAULTS,
  THEATER_PORTRAIT_LIMITS,
} from './useTheaterPortraitLayout'

describe('useTheaterPortraitLayout', () => {
  beforeEach(() => {
    localStorage.clear()
    resetTheaterPortraitLayout()
  })

  afterEach(() => {
    localStorage.clear()
  })

  it('clamps width within limits', () => {
    expect(setTheaterPortraitWidth(10)).toBe(THEATER_PORTRAIT_LIMITS.width.min)
    expect(setTheaterPortraitWidth(999)).toBe(THEATER_PORTRAIT_LIMITS.width.max)
    expect(localStorage.getItem('oclive-theater-portrait-w')).toBe('320')
    expect(getTheaterPortraitLayout().width).toBe(THEATER_PORTRAIT_LIMITS.width.max)
  })

  it('clamps max height within limits', () => {
    expect(setTheaterPortraitMaxHeight(50)).toBe(THEATER_PORTRAIT_LIMITS.maxHeight.min)
    expect(setTheaterPortraitMaxHeight(600)).toBe(THEATER_PORTRAIT_LIMITS.maxHeight.max)
    expect(localStorage.getItem('oclive-theater-portrait-max-h')).toBe('520')
  })

  it('reset restores defaults', () => {
    setTheaterPortraitWidth(160)
    setTheaterPortraitMaxHeight(280)
    const layout = resetTheaterPortraitLayout()
    expect(layout).toEqual({ ...THEATER_PORTRAIT_DEFAULTS })
    expect(localStorage.getItem('oclive-theater-portrait-w')).toBe('128')
    expect(localStorage.getItem('oclive-theater-portrait-max-h')).toBe('280')
  })
})