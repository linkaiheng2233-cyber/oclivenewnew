import { hydrateLayoutWidths } from '@oclive/shared/composables/useLayoutWidths'
import { i18n } from '@oclive/shared/i18n'
import { hostEventBus } from '@oclive/shared/lib/hostEventBus'
import { useChatStore } from '@oclive/shared/stores/chatStore'
import { tryReplaceWithDirectoryShell } from '@oclive/shared/utils/directoryShellBootstrap'
import { shouldLoadSentry } from '@oclive/shared/utils/telemetrySentry'
import { createPinia } from 'pinia'
import piniaPluginPersistedstate from 'pinia-plugin-persistedstate'
import { createApp } from 'vue'
import App from './App.vue'
import '@oclive/shared/styles/theme.css'
import '@oclive/shared/styles/theme-tool.css'
import '@oclive/shared/styles/theme-tool-management.css'
import '@oclive/shared/styles/chat-tool.css'
import '@oclive/shared/styles/global.css'
import '@oclive/shared/styles/win98/tokens.css'
import '@oclive/shared/styles/win98/primitives.css'

hydrateLayoutWidths()

void (async () => {
  const shellPromise = Promise.resolve().then(() => tryReplaceWithDirectoryShell())

  const app = createApp(App)
  app.use(i18n)
  app.config.errorHandler = (err, instance, info) => {
    console.error('[oclive] Vue render error', err, info, instance)
  }

  const tookShell = await shellPromise
  if (tookShell) {
    // This one-time startup diagnostic is intentionally visible in development consoles.
    // eslint-disable-next-line no-console
    console.info('[oclive] directory shell plugin active (main UI skipped). Set VITE_OCLIVE_DISABLE_DIRECTORY_SHELL=1 to force main app.')
    return
  }

  const pinia = createPinia()
  pinia.use(piniaPluginPersistedstate)
  app.use(pinia)

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

  // CP-INT R2: read-only observation handle for the live verification batch only.
  //
  // `VITE_OCLIVE_TEST_HARNESS` is a Vite compile-time define, so with the variable unset this whole
  // block (and every string in it) is eliminated from the normal bundle. It subscribes to the real
  // `hostEventBus` singleton and records the real `message:sent` emissions; it never replaces the
  // emitter, the stores, `fetch`, `invoke`, request UUIDs or generated replies.
  if (import.meta.env.VITE_OCLIVE_TEST_HARNESS === '1') {
    const messageSent: Array<Record<string, unknown>> = []
    let subscriptions = 0
    hostEventBus.on('message:sent', (payload) => {
      const event = (payload ?? {}) as Record<string, unknown>
      messageSent.push({
        seq: messageSent.length + 1,
        at: Date.now(),
        role_id: event.role_id ?? null,
        scene_id: event.scene_id ?? null,
        turn_id: event.turn_id ?? null,
        message: event.message ?? null,
        reply: event.reply ?? null,
        reply_aside: event.reply_aside ?? null,
      })
    })
    subscriptions += 1
    // `_s` is Pinia's internal store registry; it is what the live verification reads, so the cast
    // is confined to this compile-time-gated block.
    const storeRegistry = (pinia as unknown as { _s: Map<string, unknown> })._s
    ;(window as unknown as { __OCLIVE_TEST_HARNESS__?: unknown }).__OCLIVE_TEST_HARNESS__ = {
      version: 1,
      kinds: ['message:sent'],
      subscriber: 'hostEventBus.on',
      subscriptions,
      /** Number of real `message:sent` emissions observed so far. */
      messageSentCount: () => messageSent.length,
      /** Checkpoint: pass a previous count to read only the events since then. */
      messageSentSince: (mark = 0) => messageSent.slice(mark),
      messageSentAll: () => messageSent.slice(),
      storeIds: () => Array.from(storeRegistry.keys()).sort(),
      store: (id: string) => storeRegistry.get(id) ?? null,
    }
  }

  void (async () => {
    try {
      const sentryDsn = import.meta.env.VITE_SENTRY_DSN
      if (shouldLoadSentry(sentryDsn)) {
        const Sentry = await import('@sentry/vue')
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
