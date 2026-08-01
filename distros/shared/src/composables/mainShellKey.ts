import type { InjectionKey, Ref } from 'vue'
import type { useChatStore } from '../stores/chatStore'
import type { AppToastFn } from './useAppToast'

/**
 * Small cross-distro surface exposed by the Chat Pro shell.
 *
 * The full Chat Pro shell state remains private to Chat Pro. Shared components
 * and Theater depend only on this bridge so `distros/shared` never imports a
 * downstream distro implementation.
 */
export interface MainShellContext {
  chatStore: ReturnType<typeof useChatStore>
  settingsViewOpen: Ref<boolean>
  showToast: AppToastFn
  closeModelManager: () => void
  openSettingsToGeneral: () => void
  onInteractionModeChange: (event: Event) => Promise<void> | void
}

export const MAIN_SHELL_KEY: InjectionKey<MainShellContext> = Symbol('ocliveMainShell')
