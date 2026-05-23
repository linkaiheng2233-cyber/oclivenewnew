import * as Sentry from '@sentry/vue'
import { createPinia } from 'pinia'
import piniaPluginPersistedstate from 'pinia-plugin-persistedstate'
import { createApp } from 'vue'
import App from './App.vue'
import { i18n } from './i18n'
import { tryReplaceWithDirectoryShell } from './utils/directoryShellBootstrap'
import { shouldLoadSentry } from './utils/telemetrySentry'
import './styles/theme.css'
import './styles/global.css'

void (async () => {
  const shellPromise = Promise.resolve().then(() => tryReplaceWithDirectoryShell())

  const app = createApp(App)
  app.use(i18n)

  const sentryReady = (async () => {
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
            const request = event.request
            const url = request?.url
            if (url && request) {
              const u = new URL(url)
              request.url = `${u.origin}${u.pathname}`
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
  })()

  const [tookShell] = await Promise.all([shellPromise, sentryReady])
  if (tookShell) {
    return
  }

  const pinia = createPinia()
  pinia.use(piniaPluginPersistedstate)
  app.use(pinia)
  app.mount('#app')
})()
