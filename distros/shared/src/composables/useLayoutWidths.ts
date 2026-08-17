const LEFT_RAIL_KEY = 'oclive-layout-left-rail-w'
const SIDEPANEL_KEY = 'oclive-layout-sidepanel-w'

export const LAYOUT_WIDTH_DEFAULTS = {
  leftRail: 260,
  sidePanel: 400,
} as const

export const LAYOUT_WIDTH_LIMITS = {
  leftRail: { min: 160, max: 720 },
  sidePanel: { min: 280, max: 560 },
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
    /* ignore quota / private mode */
  }
}

function applyCssVars(leftRail: number, sidePanel: number): void {
  if (typeof document === 'undefined')
    return
  const root = document.documentElement
  root.style.setProperty('--tool-left-rail-w', `${leftRail}px`)
  root.style.setProperty('--tool-sidepanel-w', `${sidePanel}px`)
}

export function getLayoutWidths(): { leftRail: number, sidePanel: number } {
  return {
    leftRail: readStored(
      LEFT_RAIL_KEY,
      LAYOUT_WIDTH_DEFAULTS.leftRail,
      LAYOUT_WIDTH_LIMITS.leftRail.min,
      LAYOUT_WIDTH_LIMITS.leftRail.max,
    ),
    sidePanel: readStored(
      SIDEPANEL_KEY,
      LAYOUT_WIDTH_DEFAULTS.sidePanel,
      LAYOUT_WIDTH_LIMITS.sidePanel.min,
      LAYOUT_WIDTH_LIMITS.sidePanel.max,
    ),
  }
}

export function hydrateLayoutWidths(): { leftRail: number, sidePanel: number } {
  const widths = getLayoutWidths()
  applyCssVars(widths.leftRail, widths.sidePanel)
  return widths
}

export function setLeftRailWidth(px: number): number {
  const value = clamp(
    Math.round(px),
    LAYOUT_WIDTH_LIMITS.leftRail.min,
    LAYOUT_WIDTH_LIMITS.leftRail.max,
  )
  writeStored(LEFT_RAIL_KEY, value)
  document.documentElement.style.setProperty('--tool-left-rail-w', `${value}px`)
  return value
}

export function setSidePanelWidth(px: number): number {
  const value = clamp(
    Math.round(px),
    LAYOUT_WIDTH_LIMITS.sidePanel.min,
    LAYOUT_WIDTH_LIMITS.sidePanel.max,
  )
  writeStored(SIDEPANEL_KEY, value)
  document.documentElement.style.setProperty('--tool-sidepanel-w', `${value}px`)
  return value
}

export function resetLayoutWidths(): { leftRail: number, sidePanel: number } {
  try {
    localStorage.removeItem(LEFT_RAIL_KEY)
    localStorage.removeItem(SIDEPANEL_KEY)
  }
  catch {
    /* ignore */
  }
  return hydrateLayoutWidths()
}
