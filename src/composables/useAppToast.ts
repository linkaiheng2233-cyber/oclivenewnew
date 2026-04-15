import { ref } from "vue";

export type ToastType = "success" | "error" | "warning" | "info";

const TOAST_MS = 3000;

const toast = ref({
  show: false,
  type: "info" as ToastType,
  message: "",
});

/** 全局轻提示（与 `Toast.vue` 绑定；单例，任意组件调用同一套状态） */
export function useAppToast() {
  function showToast(type: ToastType, message: string): void {
    toast.value = { show: true, type, message };
    window.setTimeout(() => {
      toast.value.show = false;
    }, TOAST_MS);
  }

  return { toast, showToast };
}
