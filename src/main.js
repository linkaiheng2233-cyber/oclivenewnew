import { createApp, watch } from "vue";
import { createPinia } from "pinia";
import piniaPluginPersistedstate from "pinia-plugin-persistedstate";
import VueVirtualScroller from "vue-virtual-scroller";
import "vue-virtual-scroller/dist/vue-virtual-scroller.css";
import App from "./App.vue";
import "./styles/theme.css";
import "./styles/global.css";
import { tryReplaceWithDirectoryShell } from "./utils/directoryShellBootstrap";
import { i18n, setAppLocale } from "./i18n";
import { useUiStore } from "./stores/uiStore";

void (async () => {
  const tookShell = await tryReplaceWithDirectoryShell();
  if (tookShell) {
    return;
  }

  const app = createApp(App);

  const pinia = createPinia();
  pinia.use(piniaPluginPersistedstate);
  app.use(pinia);
  app.use(i18n);

  const uiStore = useUiStore(pinia);
  watch(
    () => uiStore.languagePref,
    () => {
      setAppLocale(uiStore.effectiveLocale);
    },
    { immediate: true },
  );

  app.use(VueVirtualScroller);
  app.mount("#app");

  const sentryDsn = import.meta.env.VITE_SENTRY_DSN;
  if (typeof sentryDsn === "string" && sentryDsn.length > 0) {
    const bootSentry = () => {
      void import("@sentry/vue").then((Sentry) => {
        try {
          const tracesRaw = import.meta.env.VITE_SENTRY_TRACES_SAMPLE_RATE;
          let tracesSampleRate = 0;
          if (typeof tracesRaw === "string" && tracesRaw.trim().length > 0) {
            const n = Number(tracesRaw);
            if (!Number.isNaN(n)) {
              tracesSampleRate = Math.min(1, Math.max(0, n));
            }
          }
          Sentry.init({
            app,
            dsn: sentryDsn,
            environment: import.meta.env.MODE,
            tracesSampleRate,
          });
        } catch (e) {
          console.warn("[sentry] init skipped", e);
        }
      });
    };
    if (typeof requestIdleCallback === "function") {
      requestIdleCallback(bootSentry, { timeout: 4000 });
    } else {
      setTimeout(bootSentry, 0);
    }
  }
})();
