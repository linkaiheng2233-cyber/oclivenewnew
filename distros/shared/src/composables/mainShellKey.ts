import type { InjectionKey } from 'vue'
import type { useMainShell } from './useMainShell'

export type MainShellContext = ReturnType<typeof useMainShell>

export const MAIN_SHELL_KEY: InjectionKey<MainShellContext> = Symbol('ocliveMainShell')
