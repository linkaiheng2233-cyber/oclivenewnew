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
  app.config.errorHandler = (err, instance, info) => {
    console.error('[oclive] Vue render error', err, info, instance)
  }

  const tookShell = await shellPromise
  if (tookShell) {
    console.info('[oclive] directory shell plugin active (main UI skipped). Set VITE_OCLIVE_DISABLE_DIRECTORY_SHELL=1 to force main app.')
    return
  }

  const pinia = createPinia()
  pinia.use(piniaPluginPersistedstate)
  app.use(pinia)

  const { useChatStore } = await import('./stores/chatStore')
  const chatStore = useChatStore()
  try {
    await chatStore.hydrateFromStorage()
    chatStore.migrateAllLegacyMessageBuckets()
  }
  catch (e) {
    console.error('[oclive] chat history hydrate failed; continuing without persisted messages', e)
  }

  if (typeof window !== 'undefined') {
    const flushChat = () => {
      void chatStore.flushPendingPersist()
    }
    window.addEventListener('beforeunload', flushChat)
    window.addEventListener('pagehide', flushChat)
    document.addEventListener('visibilitychange', () => {
      if (document.visibilityState === 'hidden')
        flushChat()
    })
  }

  app.mount('#app')

  void (async () => {
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
})()
