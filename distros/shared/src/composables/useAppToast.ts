import { ref } from 'vue'
import { recordDiagnostic } from './useDiagnosticHistory'

export type ToastType = 'success' | 'error' | 'warning' | 'info'
export type AppToastFn = (type: ToastType, message: string, details?: string) => void

const TOAST_MS_DEFAULT = 3200
const TOAST_MS_WARNING = 4200

const toast = ref({
  show: false,
  type: 'info' as ToastType,
  message: '',
  details: undefined as string | undefined,
})
let toastTimer: ReturnType<typeof setTimeout> | undefined

function toastDurationMs(type: ToastType): number {
  if (type === 'warning') {
    return TOAST_MS_WARNING
  }
  return TOAST_MS_DEFAULT
}

/** Global lightweight toast (bound to `Toast.vue`; singleton shared by all callers). */
export function useAppToast() {
  function dismissToast(): void {
    clearTimeout(toastTimer)
    toast.value.show = false
  }

  function showToast(type: ToastType, message: string, details?: string): void {
    if (type === 'error' || type === 'warning')
      recordDiagnostic({ level: type, source: 'interface', summary: message, details: details ?? '' })
    clearTimeout(toastTimer)
    toast.value = { show: true, type, message, details }
    if (type !== 'error' && !details)
      toastTimer = setTimeout(dismissToast, toastDurationMs(type))
  }

  return { toast, showToast, dismissToast }
}
