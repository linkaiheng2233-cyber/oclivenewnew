import type { PackUiTheme } from '../api'
import { onBeforeUnmount, watch } from 'vue'
import { hostEventBus } from '../lib/hostEventBus'
import { useRoleStore } from '../stores/roleStore'

/**
 * Map role pack `ui.json` → `theme` to Fluent / oclive CSS variables; remove inline overrides on role switch or cleared fields to fall back to built-in theme.
 */
export function usePackUiTheme(): void {
  const roleStore = useRoleStore()
  let applied: string[] = []

  function clearApplied(): void {
    const root = document.documentElement
    for (const k of applied) {
      root.style.removeProperty(k)
    }
    applied = []
  }

  function applyTheme(t: PackUiTheme | undefined): void {
    clearApplied()
    const root = document.documentElement
    const push = (key: string, val: string): void => {
      root.style.setProperty(key, val)
      applied.push(key)
    }
    const pc = t?.primaryColor?.trim()
    if (pc) {
      push('--fluent-accent', pc)
      push('--accent', pc)
      push('--text-accent', pc)
      hostEventBus.emitBuiltin('theme:changed', { primaryColor: pc })
    }
    const bg = t?.backgroundColor?.trim()
    if (bg) {
      push('--fluent-bg-page', bg)
      push('--bg-page', bg)
      push('--shell-page-bg', bg)
    }
    const ff = t?.fontFamily?.trim()
    if (ff) {
      push('--font-ui', `${ff}, system-ui, sans-serif`)
    }
  }

  watch(
    () => ({
      roleId: roleStore.currentRoleId,
      theme: roleStore.roleInfo.packUiConfig?.theme,
    }),
    () => applyTheme(roleStore.roleInfo.packUiConfig?.theme ?? {}),
    { deep: true, immediate: true },
  )

  onBeforeUnmount(() => {
    clearApplied()
  })
}
