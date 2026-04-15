import { createApp } from "vue";
import { createPinia } from "pinia";
import piniaPluginPersistedstate from "pinia-plugin-persistedstate";
import * as Sentry from "@sentry/vue";
import VueVirtualScroller from "vue-virtual-scroller";
import "vue-virtual-scroller/dist/vue-virtual-scroller.css";
import App from "./App.vue";
import "./styles/theme.css";
import "./styles/global.css";
import { tryReplaceWithDirectoryShell } from "./utils/directoryShellBootstrap";

void (async () => {
  const redirected = await tryReplaceWithDirectoryShell();
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
