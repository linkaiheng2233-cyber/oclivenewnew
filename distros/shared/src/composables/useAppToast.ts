import { ref } from 'vue'

export type ToastType = 'success' | 'error' | 'warning' | 'info'
export type AppToastFn = (type: ToastType, message: string) => void

const TOAST_MS_DEFAULT = 3200
const TOAST_MS_ERROR = 5600
const TOAST_MS_WARNING = 4200

const toast = ref({
  show: false,
  type: 'info' as ToastType,
  message: '',
})

function toastDurationMs(type: ToastType): number {
  if (type === 'error') {
    return TOAST_MS_ERROR
  }
  if (type === 'warning') {
    return TOAST_MS_WARNING
  }
  return TOAST_MS_DEFAULT
}

/** Global lightweight toast (bound to `Toast.vue`; singleton shared by all callers). */
export function useAppToast() {
  function showToast(type: ToastType, message: string): void {
    toast.value = { show: true, type, message }
    window.setTimeout(() => {
      toast.value.show = false
    }, toastDurationMs(type))
  }

  return { toast, showToast }
}
