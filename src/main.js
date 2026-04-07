import { createApp } from "vue";
import { createPinia } from "pinia";
import piniaPluginPersistedstate from "pinia-plugin-persistedstate";
import * as Sentry from "@sentry/vue";
import VueVirtualScroller from "vue-virtual-scroller";
import "vue-virtual-scroller/dist/vue-virtual-scroller.css";
import App from "./App.vue";
import "./styles/theme.css";
import "./styles/global.css";

const app = createApp(App);

// 仅当构建时注入 VITE_SENTRY_DSN 时启用：上报 Vue 前端未捕获异常（浏览器侧）。
// Rust/Tauri 后端错误默认不进 Sentry，见 README「可观测性与发布」。
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
