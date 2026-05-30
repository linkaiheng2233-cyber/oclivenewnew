import type { Component } from 'vue'
import * as Vue from 'vue'
import { i18n } from '../i18n'
import { readPluginAssetText } from '../api'

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

function buildCompileError(
  pluginId: string,
  vueRel: string,
  err: unknown,
): PluginVueCompileError {
  const raw
    = err instanceof Error ? err.stack || err.message : String(err ?? 'unknown error')
  const short = err instanceof Error ? err.message : String(err ?? '')
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
 * Compile and load `.vue` from directory plugin root (runtime `vue3-sfc-loader`).
 * Compile failure throws {@link PluginVueCompileError}; disk/network issues return `null` for iframe fallback.
 */
export async function loadPluginVueComponent(
  pluginId: string,
  vueRel: string,
  opts?: LoadPluginVueOptions,
): Promise<Component | null> {
  const rel0 = vueRel.replace(/\\/g, '/').replace(/^\/+/, '')
  const entry = uri(pluginId, rel0)
  const pre = opts?.preloadedEntrySource

  const moduleCache = Object.assign(Object.create(null), {
    vue: Vue,
  })

  const getFile = async (path: { toString: () => string }) => {
    const p = String(path)
    let full: string
    if (p.startsWith(SCHEME)) {
      full = p
    }
    else {
      full = uri(pluginId, joinUnder(dirname(rel0), p))
    }
    const body = full.slice(SCHEME.length)
    const slash = body.indexOf('/')
    const pid = body.slice(0, slash)
    const rel = body.slice(slash + 1)
    if (pid !== pluginId) {
      throw new Error(`cross-plugin import denied: ${p}`)
    }
    if (pre !== undefined && pre.length > 0 && full === entry) {
      const text = pre
      return {
        getContentData: (asBinary: boolean) =>
          asBinary
            ? Promise.resolve(new TextEncoder().encode(text).buffer)
            : Promise.resolve(text),
      }
    }
    const text = await readPluginAssetText(pid, rel)
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
