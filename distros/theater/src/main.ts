import RenderErrorBoundary from '@oclive/shared/components/RenderErrorBoundary.vue'
import { i18n } from '@oclive/shared/i18n'
import { installAppErrorReporting } from '@oclive/shared/utils/appErrorReporting'
import { createPinia } from 'pinia'
import piniaPluginPersistedstate from 'pinia-plugin-persistedstate'
import { createApp, h } from 'vue'
import App from './App.vue'
import { hydrateTheaterPortraitLayout } from './composables/useTheaterPortraitLayout'
import '@oclive/shared/styles/theme.css'
import '@oclive/shared/styles/theme-theater.css'
import '@oclive/shared/styles/global.css'

hydrateTheaterPortraitLayout()

const app = createApp({ render: () => h(RenderErrorBoundary, { root: true }, { default: () => h(App) }) })
app.use(i18n)
installAppErrorReporting(app)

const pinia = createPinia()
pinia.use(piniaPluginPersistedstate)
app.use(pinia)

app.mount('#app')
