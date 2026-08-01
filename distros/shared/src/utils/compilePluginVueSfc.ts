import type { Component } from 'vue'
import { readPluginAssetText } from '@oclive/shared/api'
import { ApiInvokeError } from '@oclive/shared/api/helpers'
import { i18n } from '@oclive/shared/i18n'
import * as Vue from 'vue'

/** Human-readable error when the development-only SFC compiler fails (for slot UI). */
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

interface CompilePluginVueSourceOptions {
  addStyle?: (css: string) => void
}

type CommonJsExports = Record<string, unknown> & { default?: Component }

function stableScopeId(pluginId: string, vueRel: string): string {
  const input = `${pluginId}/${vueRel}`
  let hash = 2166136261
  for (let index = 0; index < input.length; index += 1) {
    hash ^= input.charCodeAt(index)
    hash = Math.imul(hash, 16777619)
  }
  return (hash >>> 0).toString(16).padStart(8, '0')
}

function formatCompilerErrors(errors: Array<Error | string>): string {
  return errors
    .map(error => error instanceof Error ? error.message : String(error))
    .join('\n')
}

function requirePluginModule(moduleId: string): unknown {
  if (moduleId === 'vue')
    return Vue
  throw new Error(
    `Inline directory-plugin Vue may import only "vue"; unsupported import: ${moduleId}`,
  )
}

/**
 * Compile one self-contained Vue SFC for explicitly enabled unsafe DEV mode.
 * Relative script imports and preprocessors stay unsupported by contract so the
 * host never grows a second package resolver for untrusted directory plugins.
 */
export async function compilePluginVueSource(
  pluginId: string,
  vueRel: string,
  source: string,
  options: CompilePluginVueSourceOptions = {},
): Promise<Component> {
  const [compiler, babel] = await Promise.all([
    import('@vue/compiler-sfc'),
    import('@babel/standalone'),
  ])
  const filename = `${pluginId}/${vueRel}`
  const parsed = compiler.parse(source, { filename })
  if (parsed.errors.length > 0)
    throw new Error(formatCompilerErrors(parsed.errors))

  const { descriptor } = parsed
  if (descriptor.script?.src || descriptor.scriptSetup?.src || descriptor.template?.src)
    throw new Error('External <script src> and <template src> blocks are not supported')
  if (descriptor.template?.lang && descriptor.template.lang !== 'html')
    throw new Error(`Template preprocessor is not supported: ${descriptor.template.lang}`)
  const unsupportedStyle = descriptor.styles.find(style => style.src || (style.lang && style.lang !== 'css'))
  if (unsupportedStyle)
    throw new Error('External styles and style preprocessors are not supported')

  const id = stableScopeId(pluginId, vueRel)
  const hasScopedStyles = descriptor.styles.some(style => style.scoped)
  let moduleCode: string
  if (descriptor.script || descriptor.scriptSetup) {
    const script = compiler.compileScript(descriptor, {
      id,
      inlineTemplate: Boolean(descriptor.template),
      templateOptions: {
        id,
        filename,
        scoped: hasScopedStyles,
        compilerOptions: {
          scopeId: hasScopedStyles ? `data-v-${id}` : undefined,
        },
      },
    })
    moduleCode = script.content
  }
  else if (descriptor.template) {
    const template = compiler.compileTemplate({
      source: descriptor.template.content,
      filename,
      id,
      scoped: hasScopedStyles,
      compilerOptions: {
        scopeId: hasScopedStyles ? `data-v-${id}` : undefined,
      },
    })
    if (template.errors.length > 0)
      throw new Error(formatCompilerErrors(template.errors))
    moduleCode = `${template.code}\nexport default { render }`
  }
  else {
    moduleCode = 'export default {}'
  }

  const transformed = babel.transform(moduleCode, {
    filename,
    sourceType: 'module',
    plugins: ['transform-typescript', 'transform-modules-commonjs'],
  })
  if (!transformed?.code)
    throw new Error('SFC module transform produced no output')

  const module = { exports: {} as CommonJsExports }
  // Same-process Vue is already explicitly unsafe DEV-only. Keep the evaluator
  // constrained to the `vue` module instead of exposing a package resolver.
  // eslint-disable-next-line no-new-func
  const evaluate = new Function('require', 'module', 'exports', transformed.code)
  evaluate(requirePluginModule, module, module.exports)
  const component = module.exports.default ?? module.exports
  if (!component || (typeof component !== 'object' && typeof component !== 'function'))
    throw new Error('SFC module did not export a Vue component')

  if (hasScopedStyles && typeof component === 'object')
    Object.assign(component, { __scopeId: `data-v-${id}` })

  for (const style of descriptor.styles) {
    const result = compiler.compileStyle({
      source: style.content,
      filename,
      id: `data-v-${id}`,
      scoped: style.scoped,
    })
    if (result.errors.length > 0)
      throw new Error(formatCompilerErrors(result.errors))
    if (options.addStyle) {
      options.addStyle(result.code)
    }
    else {
      const element = document.createElement('style')
      element.textContent = result.code
      document.head.appendChild(element)
    }
  }

  return component as Component
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
  const pre = opts?.preloadedEntrySource
  let source: string
  try {
    source = pre !== undefined && pre.length > 0
      ? pre
      : await readPluginAssetWithExtensions(pluginId, rel0)
  }
  catch (e) {
    console.warn('[loadPluginVueComponent]', pluginId, vueRel, e)
    return null
  }
  try {
    return await compilePluginVueSource(pluginId, rel0, source)
  }
  catch (e) {
    throw buildCompileError(pluginId, rel0, e)
  }
}
