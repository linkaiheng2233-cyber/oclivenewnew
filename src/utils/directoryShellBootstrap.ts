import { invoke } from "@tauri-apps/api/tauri";
import type { DirectoryPluginBootstrap } from "./tauri-api";

export function isTauriRuntime(): boolean {
  return (
    typeof window !== "undefined" &&
    Object.prototype.hasOwnProperty.call(window, "__TAURI_INTERNALS__")
  );
}

/**
 * B1：若宿主配置了整壳目录插件，在挂载 Vue 前跳转到 `https://ocliveplugin.localhost/...`。
 *
 * @returns 若已发起 `location.replace` 则为 true（调用方不应再挂载应用根组件）。
 */
export async function tryReplaceWithDirectoryShell(): Promise<boolean> {
  if (!isTauriRuntime()) return false;
  try {
    const boot = await invoke<DirectoryPluginBootstrap>(
      "get_directory_plugin_bootstrap",
    );
    const url =
      typeof boot?.shellUrl === "string" && boot.shellUrl.length > 0
        ? boot.shellUrl
        : null;
    if (!url) return false;
    const here = window.location.href.split("#")[0];
    const target = url.split("#")[0];
    if (here !== target) {
      window.location.replace(url);
      return true;
    }
  } catch (e) {
    console.warn("[oclive] directory shell bootstrap skipped", e);
  }
  return false;
}
