import { createApp } from "vue";
import { createPinia } from "pinia";
import piniaPluginPersistedstate from "pinia-plugin-persistedstate";
import * as Sentry from "@sentry/vue";
import VueVirtualScroller from "vue-virtual-scroller";
import { invoke } from "@tauri-apps/api/tauri";
import "vue-virtual-scroller/dist/vue-virtual-scroller.css";
import App from "./App.vue";
import "./styles/theme.css";
import "./styles/global.css";

function isTauriRuntime() {
  return (
    typeof window !== "undefined" &&
    Object.prototype.hasOwnProperty.call(window, "__TAURI_INTERNALS__")
  );
}

/** B1：若宿主配置了整壳插件，在挂载 Vue 前跳转到 `https://ocliveplugin.localhost/...`。 */
async function maybeReplaceWithDirectoryShell() {
  if (!isTauriRuntime()) return false;
  try {
    const boot = await invoke("get_directory_plugin_bootstrap");
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

void (async () => {
  const redirected = await maybeReplaceWithDirectoryShell();
  if (redirected) {
    return;
  }

  const app = createApp(App);

  try {
    const sentryDsn = import.meta.env.VITE_SENTRY_DSN;
    if (typeof sentryDsn === "string" && sentryDsn.length > 0) {
      Sentry.init({
        app,
        dsn: sentryDsn,
        environment: import.meta.env.MODE,
      });
    }
  } catch (e) {
    console.warn("[sentry] init skipped", e);
  }

  const pinia = createPinia();
  pinia.use(piniaPluginPersistedstate);
  app.use(pinia);
  app.use(VueVirtualScroller);
  app.mount("#app");
})();
