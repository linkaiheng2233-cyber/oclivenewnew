import { i18n } from './index'

/** 在非组件模块（Pinia、纯函数）中读取当前 locale 的翻译。 */
export function rt(
  key: string,
  values?: Record<string, string | number | boolean>,
): string {
  return i18n.global.t(key, (values ?? {}) as Record<string, unknown>)
}
