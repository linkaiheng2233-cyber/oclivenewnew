import { i18n as appI18n } from './index'

let pluginWorkbenchI18nLoaded = false

/** Plugin workbench i18n fragments; loaded only when opening plugin manager/market (keeps first-screen bundle lean). */
export async function ensurePluginWorkbenchI18n(i18n: typeof appI18n = appI18n): Promise<void> {
  if (pluginWorkbenchI18nLoaded)
    return
  const [zhMod, enMod] = await Promise.all([
    import('./locales/fragments/pluginWorkbench.zh'),
    import('./locales/fragments/pluginWorkbench.en'),
  ])
  i18n.global.mergeLocaleMessage('zh-CN', { pluginWorkbench: zhMod.default })
  i18n.global.mergeLocaleMessage('en-US', { pluginWorkbench: enMod.default })
  pluginWorkbenchI18nLoaded = true
}
