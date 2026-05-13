import { confirm as tauriConfirm } from "@tauri-apps/api/dialog";

function isTauriWebview(): boolean {
  return (
    typeof window !== "undefined" &&
    Object.prototype.hasOwnProperty.call(window, "__TAURI_INTERNALS__")
  );
}

/** 在 Tauri 内用系统对话框（置顶）；否则回退 `window.confirm`（浏览器）。 */
export async function appConfirm(
  message: string,
  options?: { title?: string; type?: "info" | "warning" },
): Promise<boolean> {
  if (isTauriWebview()) {
    return tauriConfirm(message, {
      title: options?.title ?? "Oclive",
      type: options?.type ?? "warning",
    });
  }
  return window.confirm(message);
}
