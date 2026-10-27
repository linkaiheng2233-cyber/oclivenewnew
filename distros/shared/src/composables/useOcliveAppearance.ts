import type { EffectScope } from 'vue'
import { readHostAppearance } from '@oclive/shared/lib/hostAppearance'
import { hostEventBus } from '@oclive/shared/lib/hostEventBus'
import { computed, effectScope, onMounted, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

const THEME_STORAGE_KEY = 'oclive-runtime-theme'
const SCALE_STORAGE_KEY = 'oclive-runtime-ui-scale'

export type ThemePreference = 'light' | 'dark' | 'system'

/** Same font-size steps as pack-editor / launcher. */
const UI_SCALE_STEPS = [0.8, 0.88, 0.96, 1, 1.08, 1.16, 1.24] as const
const themePreference = ref<ThemePreference>('system')
const effectiveTheme = ref<'light' | 'dark'>('light')
const uiScaleIndex = ref(3)
let mountCount = 0
let appearanceScope: EffectScope | undefined
let removeSchemeListener: (() => void) | undefined

function resolveEffectiveTheme(pref: ThemePreference): 'light' | 'dark' {
  if (pref === 'light')
    return 'light'
  if (pref === 'dark')
    return 'dark'
  return window.matchMedia('(prefers-color-scheme: dark)').matches
    ? 'dark'
    : 'light'
}

function readStoredTheme(): ThemePreference {
  try {
    const raw = localStorage.getItem(THEME_STORAGE_KEY)
    if (raw === 'light' || raw === 'dark' || raw === 'system')
      return raw
  }
  catch {
    /* ignore */
  }
  return 'system'
}

function nearestScaleIndex(value: number): number {
  let best = 3
  let bestDiff = Infinity
  UI_SCALE_STEPS.forEach((s, i) => {
    const d = Math.abs(s - value)
    if (d < bestDiff) {
      bestDiff = d
      best = i
    }
  })
  return best
}

function readStoredScaleIndex(): number {
  try {
    const raw = localStorage.getItem(SCALE_STORAGE_KEY)
    if (raw == null || raw === '')
      return 3
    const n = Number(raw)
    if (!Number.isFinite(n))
      return 3
    return nearestScaleIndex(n)
  }
  catch {
    return 3
  }
}

function applyThemeToDocument(): void {
  effectiveTheme.value = resolveEffectiveTheme(themePreference.value)
  document.documentElement.setAttribute('data-theme', effectiveTheme.value)
}

function applyScaleToDocument(): void {
  const scale = UI_SCALE_STEPS[uiScaleIndex.value] ?? 1
  document.documentElement.style.setProperty('--oclive-ui-scale', String(scale))
}

function emitAppearanceChanged(): void {
  hostEventBus.emitBuiltin('appearance:changed', {
    ...readHostAppearance(),
    themePreference: themePreference.value,
  })
}

function installAppearance(): void {
  themePreference.value = readStoredTheme()
  uiScaleIndex.value = readStoredScaleIndex()
  applyThemeToDocument()
  applyScaleToDocument()
  emitAppearanceChanged()

  const mq = window.matchMedia('(prefers-color-scheme: dark)')
  const onScheme = () => {
    if (themePreference.value === 'system') {
      applyThemeToDocument()
      emitAppearanceChanged()
    }
  }
  mq.addEventListener('change', onScheme)
  removeSchemeListener = () => mq.removeEventListener('change', onScheme)

  // Detached from any one caller: Settings may unmount while the shell stays open.
  appearanceScope = effectScope(true)
  appearanceScope.run(() => {
    watch(themePreference, (v) => {
      try {
        localStorage.setItem(THEME_STORAGE_KEY, v)
      }
      catch {
        /* unavailable storage does not prevent an in-session theme change */
      }
      applyThemeToDocument()
      emitAppearanceChanged()
    }, { flush: 'sync' })
    watch(uiScaleIndex, (i) => {
      const scale = UI_SCALE_STEPS[i] ?? 1
      try {
        localStorage.setItem(SCALE_STORAGE_KEY, String(scale))
      }
      catch {
        /* ignore */
      }
      applyScaleToDocument()
      emitAppearanceChanged()
    }, { flush: 'sync' })
  })
}

export function useOcliveAppearance() {
  const { t } = useI18n()

  onMounted(() => {
    mountCount += 1
    if (mountCount === 1)
      installAppearance()
  })
  onUnmounted(() => {
    mountCount = Math.max(0, mountCount - 1)
    if (mountCount === 0) {
      appearanceScope?.stop()
      appearanceScope = undefined
      removeSchemeListener?.()
      removeSchemeListener = undefined
    }
  })

  const themeCycleLabel = computed(() => t(`app.theme.${themePreference.value}`))

  function cycleTheme() {
    const order: ThemePreference[] = ['system', 'light', 'dark']
    const i = order.indexOf(themePreference.value)
    themePreference.value = order[(i + 1) % order.length]!
  }

  function bumpScale(delta: number) {
    uiScaleIndex.value = Math.max(
      0,
      Math.min(UI_SCALE_STEPS.length - 1, uiScaleIndex.value + delta),
    )
  }

  function toggleDarkTheme(): void {
    themePreference.value = effectiveTheme.value === 'dark' ? 'light' : 'dark'
  }

  const scaleLabel = computed(
    () => `${Math.round((UI_SCALE_STEPS[uiScaleIndex.value] ?? 1) * 100)}%`,
  )

  return {
    themePreference,
    effectiveTheme,
    themeCycleLabel,
    cycleTheme,
    toggleDarkTheme,
    bumpScale,
    scaleLabel,
  }
}
