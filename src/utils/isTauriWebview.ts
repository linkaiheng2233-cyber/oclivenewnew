/**
 * 是否在 Tauri WebView 内。
 * v1 预注入多为 `__TAURI_IPC__` / `__TAURI__`；v2 常见 `__TAURI_INTERNALS__`。
 * 仅检测 `__TAURI_INTERNALS__` 会在 Tauri 1 下恒为 false，导致误走浏览器分支。
 */
export function isTauriWebview(): boolean {
  if (typeof window === "undefined") return false;
  const w = window as unknown as Record<string, unknown>;
  return (
    Object.prototype.hasOwnProperty.call(w, "__TAURI_INTERNALS__") ||
    Object.prototype.hasOwnProperty.call(w, "__TAURI_IPC__") ||
    Object.prototype.hasOwnProperty.call(w, "__TAURI__")
  );
}
