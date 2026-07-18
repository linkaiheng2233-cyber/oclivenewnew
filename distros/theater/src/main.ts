import { i18n } from '@oclive/shared/i18n'
import { createPinia } from 'pinia'
import piniaPluginPersistedstate from 'pinia-plugin-persistedstate'
import { createApp } from 'vue'
import App from './App.vue'
import { hydrateTheaterPortraitLayout } from './composables/useTheaterPortraitLayout'
import '@oclive/shared/styles/theme.css'
import '@oclive/shared/styles/theme-theater.css'
import '@oclive/shared/styles/global.css'

hydrateTheaterPortraitLayout()

const app = createApp(App)
app.use(i18n)
app.config.errorHandler = (err, instance, info) => {
  console.error('[oclive-theater] Vue render error', err, info, instance)
}

const pinia = createPinia()
pinia.use(piniaPluginPersistedstate)
app.use(pinia)

app.mount('#app')
