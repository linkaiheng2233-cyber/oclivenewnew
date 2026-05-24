import type { I18n } from 'vue-i18n'
import { i18n as appI18n } from './index'

let pluginWorkbenchI18nLoaded = false

/** 插件工作台 i18n 片段：仅在打开插件管理/市场时加载，避免进入首屏 bundle。 */
export async function ensurePluginWorkbenchI18n(i18n: I18n = appI18n): Promise<void> {
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
