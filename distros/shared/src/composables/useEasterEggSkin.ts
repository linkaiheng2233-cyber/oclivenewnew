import { useAppToast } from '@oclive/shared/composables/useAppToast'
import { readHostAppearance } from '@oclive/shared/lib/hostAppearance'
import { hostEventBus } from '@oclive/shared/lib/hostEventBus'
import { onMounted, onUnmounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'

const SKIN_STORAGE_KEY = 'oclive-runtime-skin'
const UNLOCK_STORAGE_KEY = 'oclive-easteregg-unlocked'

const KONAMI_SEQUENCE = [
  'ArrowUp',
  'ArrowUp',
  'ArrowDown',
  'ArrowDown',
  'ArrowLeft',
  'ArrowRight',
  'ArrowLeft',
  'ArrowRight',
  'KeyB',
  'KeyA',
] as const

const skinUnlocked = ref(false)
const win98Enabled = ref(false)

let mountCount = 0
let konamiIndex = 0
let keydownHandler: ((e: KeyboardEvent) => void) | undefined

function readUnlocked(): boolean {
  try {
    return localStorage.getItem(UNLOCK_STORAGE_KEY) === '1'
  }
  catch {
    return false
  }
}

function readSkinEnabled(): boolean {
  try {
    return localStorage.getItem(SKIN_STORAGE_KEY) === 'win98'
  }
  catch {
    return false
  }
}

function isTauriWebview(): boolean {
  return typeof window !== 'undefined' && Object.hasOwn(window, '__TAURI_INTERNALS__')
}

async function syncNativeDecorations(skinEnabled: boolean): Promise<void> {
  if (!isTauriWebview())
    return
  try {
    const { getCurrentWebviewWindow } = await import('@tauri-apps/api/webviewWindow')
    await getCurrentWebviewWindow().setDecorations(!skinEnabled)
  }
  catch {
    /* web build or permission denied */
  }
}

function applySkinToDocument(enabled: boolean): void {
  document.documentElement.setAttribute('data-skin', enabled ? 'win98' : 'default')
  void syncNativeDecorations(enabled)
}

function persistSkin(enabled: boolean): void {
  try {
    if (enabled)
      localStorage.setItem(SKIN_STORAGE_KEY, 'win98')
    else
      localStorage.removeItem(SKIN_STORAGE_KEY)
  }
  catch {
    /* ignore */
  }
}

function persistUnlocked(): void {
  try {
    localStorage.setItem(UNLOCK_STORAGE_KEY, '1')
  }
  catch {
    /* ignore */
  }
}

function emitAppearanceChanged(skin: 'default' | 'win98'): void {
  const snap = readHostAppearance()
  hostEventBus.emitBuiltin('appearance:changed', {
    ...snap,
    skin,
  })
}

export function useEasterEggSkin() {
  const { t } = useI18n()
  const { showToast } = useAppToast()

  function enableWin98(): void {
    win98Enabled.value = true
    persistSkin(true)
    applySkinToDocument(true)
    emitAppearanceChanged('win98')
  }

  function disableWin98(): void {
    win98Enabled.value = false
    persistSkin(false)
    applySkinToDocument(false)
    emitAppearanceChanged('default')
  }

  function unlockAndEnable(): void {
    skinUnlocked.value = true
    persistUnlocked()
    enableWin98()
    showToast('success', t('app.toast.eggUnlocked'))
  }

  function toggleWin98(): void {
    if (!skinUnlocked.value)
      return
    if (win98Enabled.value)
      disableWin98()
    else
      enableWin98()
  }

  function onKeyDown(e: KeyboardEvent): void {
    const expected = KONAMI_SEQUENCE[konamiIndex]
    if (e.code === expected) {
      konamiIndex += 1
      if (konamiIndex >= KONAMI_SEQUENCE.length) {
        konamiIndex = 0
        if (!skinUnlocked.value)
          unlockAndEnable()
        else if (!win98Enabled.value)
          enableWin98()
      }
      return
    }
    konamiIndex = e.code === KONAMI_SEQUENCE[0] ? 1 : 0
  }

  function install(): void {
    skinUnlocked.value = readUnlocked()
    win98Enabled.value = readSkinEnabled()
    applySkinToDocument(win98Enabled.value)
    if (!keydownHandler) {
      keydownHandler = onKeyDown
      window.addEventListener('keydown', keydownHandler)
    }
  }

  function uninstall(): void {
    if (keydownHandler) {
      window.removeEventListener('keydown', keydownHandler)
      keydownHandler = undefined
    }
    konamiIndex = 0
  }

  onMounted(() => {
    mountCount += 1
    if (mountCount === 1)
      install()
  })

  onUnmounted(() => {
    mountCount = Math.max(0, mountCount - 1)
    if (mountCount === 0)
      uninstall()
  })

  return {
    skinUnlocked,
    win98Enabled,
    toggleWin98,
  }
}
