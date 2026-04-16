import { invoke } from "@tauri-apps/api/tauri";
import { createApp } from "vue";
import { createPinia } from "pinia";
import DirectoryShellApp from "../DirectoryShellApp.vue";
import type { DirectoryPluginBootstrap } from "./tauri-api";
import { readPluginAssetText } from "./tauri-api";

export function isTauriRuntime(): boolean {
  return (
    typeof window !== "undefined" &&
    Object.prototype.hasOwnProperty.call(window, "__TAURI_INTERNALS__")
  );
}

/**
 * 若配置了整壳目录插件：优先在 **`shell.vueEntry` + 非强制 iframe** 时用宿主 Vue 挂载整壳；
 * 否则在 **`shellUrl`** 与当前页不同时执行 `location.replace(shellUrl)`（HTML 整壳）。
 *
 * @returns 若已处理整壳（Vue 已挂载或已发起 HTML 跳转）则为 true，调用方不应再挂载应用根组件。
 */
export async function tryReplaceWithDirectoryShell(): Promise<boolean> {
  if (!isTauriRuntime()) return false;
  try {
    const boot = await invoke<DirectoryPluginBootstrap>("get_directory_plugin_bootstrap", {
      role_id: null,
    });
    const shellUrl =
      typeof boot?.shellUrl === "string" && boot.shellUrl.length > 0
        ? boot.shellUrl
        : null;
    const shellPid =
      typeof boot?.shellPluginId === "string" && boot.shellPluginId.trim().length > 0
        ? boot.shellPluginId.trim()
        : null;
    if (!shellUrl || !shellPid) {
      return false;
    }

    const forceIframe = boot.forceIframeMode === true;
    const vueEntry =
      typeof boot.shellVueEntry === "string" ? boot.shellVueEntry.trim() : "";

    function redirectShellError(reason: string): void {
      const u = new URL("plugin-shell-error.html", window.location.href);
      u.searchParams.set("reason", reason);
      window.location.replace(u.toString());
    }

    async function shellHtmlReachable(url: string): Promise<boolean> {
      try {
        const r = await fetch(url, { method: "GET", cache: "no-store" });
        return r.ok;
      } catch {
        return false;
      }
    }

    if (!forceIframe && vueEntry.length > 0) {
      try {
        await readPluginAssetText(shellPid, vueEntry);
      } catch {
        redirectShellError(
          encodeURIComponent(`无法读取整壳 Vue 入口：${vueEntry}`),
        );
        return true;
      }
      const pinia = createPinia();
      const app = createApp(DirectoryShellApp, {
        pluginId: shellPid,
        vueEntry,
        bridgeAssetRel: vueEntry.replace(/\\/g, "/"),
        htmlFallbackUrl: shellUrl,
        developerMode: boot.developerMode === true,
      });
      app.use(pinia);
      app.mount("#app");
      return true;
    }

    const here = window.location.href.split("#")[0];
    const target = shellUrl.split("#")[0];
    if (here !== target) {
      const ok = await shellHtmlReachable(shellUrl);
      if (!ok) {
        redirectShellError(
          encodeURIComponent("无法加载整壳 HTML 入口，请检查 shell.entry 路径与文件是否存在"),
        );
        return true;
      }
      window.location.replace(shellUrl);
      return true;
    }
  } catch (e) {
    console.warn("[oclive] directory shell bootstrap skipped", e);
  }
  return false;
}
