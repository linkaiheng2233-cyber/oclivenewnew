export const PRESET_PICKER_DONE_KEY = 'oclive_preset_picker_done'

export interface PresetRoleOption {
  id: string
  name: string
  description?: string
  featured?: boolean
  preset_order?: number
  interaction_mode_suggestion?: string | null
}

export function hasCompletedPresetPicker(): boolean {
  try {
    return localStorage.getItem(PRESET_PICKER_DONE_KEY) === '1'
  }
  catch {
    return false
  }
}

export function markPresetPickerDone(): void {
  try {
    localStorage.setItem(PRESET_PICKER_DONE_KEY, '1')
  }
  catch {
    // ignore quota / private mode
  }
}

export function sortPresetRoles(roles: PresetRoleOption[]): PresetRoleOption[] {
  return [...roles].sort((a, b) => {
    const orderA = a.preset_order ?? 999
    const orderB = b.preset_order ?? 999
    if (orderA !== orderB)
      return orderA - orderB
    return a.name.localeCompare(b.name, 'zh-CN')
  })
}

export function resolveDefaultRoleId(roles: PresetRoleOption[]): string {
  const sorted = sortPresetRoles(roles)
  const featured = sorted.find(r => r.featured)
  return featured?.id ?? sorted[0]?.id ?? ''
}

/** First-run gallery: skip when user already has a valid persisted role (upgrade path). */
export function shouldShowPresetPicker(
  roles: PresetRoleOption[],
  currentRoleId: string,
): boolean {
  if (hasCompletedPresetPicker())
    return false
  if (roles.length <= 1)
    return false
  const trimmed = currentRoleId.trim()
  if (trimmed && roles.some(r => r.id === trimmed)) {
    markPresetPickerDone()
    return false
  }
  return true
}

export function presetGalleryRoles(roles: PresetRoleOption[]): PresetRoleOption[] {
  const featured = sortPresetRoles(roles.filter(r => r.featured))
  if (featured.length > 0)
    return featured
  return sortPresetRoles(roles)
}
