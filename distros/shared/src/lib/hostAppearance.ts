/**
 * Read host effective light/dark theme and UI scale (matches `html[data-theme]`, `--oclive-ui-scale`).
 * Used by plugin `oclive.getAppearance()` and built-in event payloads.
 */
export function readHostAppearance(): {
  effectiveTheme: 'light' | 'dark'
  scale: number
} {
  const dt = document.documentElement.getAttribute('data-theme')
  const effectiveTheme: 'light' | 'dark' = dt === 'dark' ? 'dark' : 'light'
  const raw = getComputedStyle(document.documentElement)
    .getPropertyValue('--oclive-ui-scale')
    .trim()
  const scale = Number.parseFloat(raw) || 1
  return { effectiveTheme, scale }
}
