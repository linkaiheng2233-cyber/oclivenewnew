import type { Component } from 'vue'
import { readPluginAssetText } from '@oclive/shared/api'
import { ApiInvokeError } from '@oclive/shared/api/helpers'
import { i18n } from '@oclive/shared/i18n'
import * as Vue from 'vue'

const SCHEME = 'oclive-plugin://'

/** Human-readable error when `vue3-sfc-loader` compile fails (for slot UI). */
export class PluginVueCompileError extends Error {
  readonly pluginId: string
  readonly componentPath: string
  readonly friendlyMessage: string
  readonly rawMessage: string

  constructor(
    pluginId: string,
    componentPath: string,
    friendlyMessage: string,
    rawMessage: string,
  ) {
    super(friendlyMessage)
    this.name = 'PluginVueCompileError'
    this.pluginId = pluginId
    this.componentPath = componentPath
    this.friendlyMessage = friendlyMessage
    this.rawMessage = rawMessage
  }
}

function uri(pluginId: string, rel: string): string {
  const r = rel.replace(/\\/g, '/').replace(/^\/+/, '')
  return `${SCHEME}${pluginId}/${r}`
}

function dirname(rel: string): string {
  const i = rel.lastIndexOf('/')
  return i === -1 ? '' : rel.slice(0, i)
}

function joinUnder(baseDir: string, rel: string): string {
  const parts = `${baseDir}/${rel}`.split('/').filter(Boolean)
  const stack: string[] = []
  for (const p of parts) {
    if (p === '..')
      stack.pop()
    else if (p !== '.')
      stack.push(p)
  }
  return stack.join('/')
}

function stripQuery(rel: string): string {
  return rel.split('?')[0]?.split('#')[0] ?? rel
}

/** Extract plugin-root-relative path from loader URL, if present. */
function pluginRelFromUrl(pluginId: string, url: string): string | null {
  const p = stripQuery(String(url).replace(/\\/g, '/'))
  if (p.startsWith(SCHEME)) {
    const body = p.slice(SCHEME.length)
    const slash = body.indexOf('/')
    if (slash === -1 || body.slice(0, slash) !== pluginId)
      return null
    return stripQuery(body.slice(slash + 1))
  }
  const needle = `/plugins/${pluginId}/`
  const idx = p.lastIndexOf(needle)
  if (idx !== -1)
    return stripQuery(p.slice(idx + needle.length))
  return null
}

/** Map vue3-sfc-loader request paths to plugin-root-relative paths. */
function resolvePluginAssetRel(
  pluginId: string,
  entryRel: string,
  requestPath: string,
): string {
  const fromUrl = pluginRelFromUrl(pluginId, requestPath)
  if (fromUrl)
    return fromUrl

  const baseDir = dirname(entryRel)
  const raw = stripQuery(String(requestPath).replace(/\\/g, '/'))
  // Already plugin-root-relative (avoid slots/ + slots/foo → slots/slots/foo).
  if (baseDir && (raw === baseDir || raw.startsWith(`${baseDir}/`)))
    return raw
  if (raw.includes('/') && !raw.startsWith('./') && !raw.startsWith('../'))
    return raw.replace(/^\/+/, '')
  return joinUnder(baseDir, raw)
}

async function readPluginAssetWithExtensions(
  pluginId: string,
  rel: string,
): Promise<string> {
  const base = rel.replace(/\\/g, '/').replace(/^\/+/, '')
  const fileName = base.split('/').pop() ?? base
  const hasExt = /\.[a-z0-9]+$/i.test(fileName)
  const candidates: string[] = hasExt ? [base] : [base, `${base}.ts`, `${base}.js`, `${base}.vue`]
  if (base.endsWith('.js'))
    candidates.push(`${base.slice(0, -3)}.ts`)
  let lastErr: unknown
  const tried: string[] = []
  for (const candidate of [...new Set(candidates)]) {
    tried.push(candidate)
    try {
      return await readPluginAssetText(pluginId, candidate)
    }
    catch (e) {
      lastErr = e
    }
  }
  if (import.meta.env.DEV) {
    console.warn('[compilePluginVueSfc] read_plugin_asset_text failed', {
      pluginId,
      rel,
      tried,
      lastErr,
    })
  }
  throw lastErr
}

function buildCompileError(
  pluginId: string,
  vueRel: string,
  err: unknown,
): PluginVueCompileError {
  const raw
    = err instanceof Error ? err.stack || err.message : String(err ?? 'unknown error')
  let short = err instanceof Error ? err.message : String(err ?? '')
  if (err instanceof ApiInvokeError && err.kernel?.message)
    short = err.kernel.message
  const lineHint
    = short.match(/\((\d+),(\d+)\)|:(\d+):(\d+)|line\s*(\d+)/i)?.[0] ?? short.slice(0, 240)
  const friendly = String(
    i18n.global.t('pluginWorkbench.slotEmbed.vueCompileFailed', {
      pluginId,
      path: vueRel,
      detail: lineHint,
    }),
  )
  return new PluginVueCompileError(pluginId, vueRel, friendly, raw)
}

export interface LoadPluginVueOptions {
  /** Entry `.vue` source already loaded (e.g. after security scan); avoids second `read_plugin_asset_text`. */
  preloadedEntrySource?: string
}

/**
 * Compile and load `.vue` from a directory plugin root for explicit unsafe DEV mode.
 * Compile failure throws {@link PluginVueCompileError}; disk/network issues return `null` for iframe fallback.
 */
export async function loadPluginVueComponent(
  pluginId: string,
  vueRel: string,
  opts?: LoadPluginVueOptions,
): Promise<Component | null> {
  if (!import.meta.env.DEV)
    return null
  const rel0 = vueRel.replace(/\\/g, '/').replace(/^\/+/, '')
  const entry = uri(pluginId, rel0)
  const pre = opts?.preloadedEntrySource

  const moduleCache = Object.assign(Object.create(null), {
    vue: Vue,
  })

  const getFile = async (path: { toString: () => string }) => {
    const p = String(path)
    const rel = resolvePluginAssetRel(pluginId, rel0, p)
    if (import.meta.env.DEV) {
      // Useful only while diagnosing third-party SFC asset resolution.
      // eslint-disable-next-line no-console
      console.debug('[compilePluginVueSfc.getFile]', { pluginId, request: p, rel })
    }
    if (pre !== undefined && pre.length > 0 && stripQuery(p) === stripQuery(entry)) {
      const text = pre
      return {
        getContentData: (asBinary: boolean) =>
          asBinary
            ? Promise.resolve(new TextEncoder().encode(text).buffer)
            : Promise.resolve(text),
      }
    }
    const text = await readPluginAssetWithExtensions(pluginId, rel)
    return {
      getContentData: (asBinary: boolean) =>
        asBinary
          ? Promise.resolve(new TextEncoder().encode(text).buffer)
          : Promise.resolve(text),
    }
  }

  try {
    const { loadModule } = await import('vue3-sfc-loader')
    try {
      const mod = await loadModule(entry, {
        moduleCache,
        getFile,
        addStyle(styleText: string) {
          const el = document.createElement('style')
          el.textContent = styleText
          document.head.appendChild(el)
        },
      } as never)
      const m = mod as { default?: Component }
      return (m.default ?? (mod as Component)) ?? null
    }
    catch (e) {
      throw buildCompileError(pluginId, rel0, e)
    }
  }
  catch (e) {
    if (e instanceof PluginVueCompileError) {
      throw e
    }
    console.warn('[loadPluginVueComponent]', pluginId, vueRel, e)
    return null
  }
}
