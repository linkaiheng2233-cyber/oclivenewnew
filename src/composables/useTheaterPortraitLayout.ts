const PORTRAIT_W_KEY = 'oclive-theater-portrait-w'
const PORTRAIT_MAX_H_KEY = 'oclive-theater-portrait-max-h'

export const THEATER_PORTRAIT_DEFAULTS = {
  width: 128,
  maxHeight: 280,
} as const

export const THEATER_PORTRAIT_LIMITS = {
  width: { min: 72, max: 320 },
  maxHeight: { min: 120, max: 520 },
} as const

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value))
}

function readStored(key: string, fallback: number, min: number, max: number): number {
  try {
    const raw = localStorage.getItem(key)
    if (raw == null || raw === '')
      return fallback
    const n = Number(raw)
    if (!Number.isFinite(n))
      return fallback
    return clamp(Math.round(n), min, max)
  }
  catch {
    return fallback
  }
}

function writeStored(key: string, value: number): void {
  try {
    localStorage.setItem(key, String(value))
  }
  catch {
    /* ignore */
  }
}

function applyCssVars(width: number, maxHeight: number): void {
  if (typeof document === 'undefined')
    return
  const root = document.documentElement
  root.style.setProperty('--theater-portrait-w', `${width}px`)
  root.style.setProperty('--theater-portrait-max-h', `${maxHeight}px`)
}

export function getTheaterPortraitLayout(): { width: number, maxHeight: number } {
  return {
    width: readStored(
      PORTRAIT_W_KEY,
      THEATER_PORTRAIT_DEFAULTS.width,
      THEATER_PORTRAIT_LIMITS.width.min,
      THEATER_PORTRAIT_LIMITS.width.max,
    ),
    maxHeight: readStored(
      PORTRAIT_MAX_H_KEY,
      THEATER_PORTRAIT_DEFAULTS.maxHeight,
      THEATER_PORTRAIT_LIMITS.maxHeight.min,
      THEATER_PORTRAIT_LIMITS.maxHeight.max,
    ),
  }
}

export function hydrateTheaterPortraitLayout(): { width: number, maxHeight: number } {
  const layout = getTheaterPortraitLayout()
  applyCssVars(layout.width, layout.maxHeight)
  return layout
}

export function setTheaterPortraitWidth(px: number): number {
  const value = clamp(
    Math.round(px),
    THEATER_PORTRAIT_LIMITS.width.min,
    THEATER_PORTRAIT_LIMITS.width.max,
  )
  writeStored(PORTRAIT_W_KEY, value)
  const layout = getTheaterPortraitLayout()
  applyCssVars(value, layout.maxHeight)
  return value
}

export function setTheaterPortraitMaxHeight(px: number): number {
  const value = clamp(
    Math.round(px),
    THEATER_PORTRAIT_LIMITS.maxHeight.min,
    THEATER_PORTRAIT_LIMITS.maxHeight.max,
  )
  writeStored(PORTRAIT_MAX_H_KEY, value)
  const layout = getTheaterPortraitLayout()
  applyCssVars(layout.width, value)
  return value
}

export function resetTheaterPortraitLayout(): { width: number, maxHeight: number } {
  writeStored(PORTRAIT_W_KEY, THEATER_PORTRAIT_DEFAULTS.width)
  writeStored(PORTRAIT_MAX_H_KEY, THEATER_PORTRAIT_DEFAULTS.maxHeight)
  applyCssVars(THEATER_PORTRAIT_DEFAULTS.width, THEATER_PORTRAIT_DEFAULTS.maxHeight)
  return { ...THEATER_PORTRAIT_DEFAULTS }
}
