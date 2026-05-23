import * as Sentry from '@sentry/vue'
import { createPinia } from 'pinia'
import piniaPluginPersistedstate from 'pinia-plugin-persistedstate'
import { createApp } from 'vue'
import VueVirtualScroller from 'vue-virtual-scroller'
import App from './App.vue'
import { i18n } from './i18n'
import { tryReplaceWithDirectoryShell } from './utils/directoryShellBootstrap'
import { shouldLoadSentry } from './utils/telemetrySentry'
import 'vue-virtual-scroller/dist/vue-virtual-scroller.css'
import './styles/theme.css'
import './styles/global.css'

void (async () => {
  const tookShell = await tryReplaceWithDirectoryShell()
  if (tookShell) {
    return
  }

  const app = createApp(App)
  app.use(i18n)

  try {
    const sentryDsn = import.meta.env.VITE_SENTRY_DSN
    if (shouldLoadSentry(sentryDsn)) {
      Sentry.init({
        app,
        dsn: sentryDsn,
        environment: import.meta.env.MODE,
        sendDefaultPii: false,
        tracesSampleRate: 0,
        beforeSend(event) {
          try {
            const url = event.request?.url
            if (url) {
              const u = new URL(url)
              event.request.url = `${u.origin}${u.pathname}`
            }
          }
          catch {
            /* ignore malformed URLs */
          }
          return event
        },
      })
    }
  }
  catch (e) {
    console.warn('[sentry] init skipped', e)
  }

  const pinia = createPinia()
  pinia.use(piniaPluginPersistedstate)
  app.use(pinia)
  app.use(VueVirtualScroller)
  app.mount('#app')
})()
