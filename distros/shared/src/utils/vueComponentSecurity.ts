/**
 * Static scan of Vue slot source (denylist).
 * When extending rules: add tokens to `DANGEROUS_PATTERNS` first, then AST rules as needed; keep messages readable.
 * acorn / acorn-walk load dynamically only after a denylist token hit, avoiding main-screen bundle cost.
 */

export interface ScanResult {
  warnings: string[]
}

export const DANGEROUS_PATTERNS = [
  {
    token: 'window.__TAURI__',
    warning: 'Access to `window.__TAURI__` / `window.tauri` detected',
  },
  {
    token: 'window.tauri',
    warning: 'Access to `window.__TAURI__` / `window.tauri` detected',
  },
  { token: 'fetch(', warning: '`fetch()` call detected' },
  { token: 'XMLHttpRequest', warning: '`XMLHttpRequest` usage detected' },
  { token: 'document.cookie', warning: '`document.cookie` access detected' },
  { token: 'localStorage.setItem', warning: '`localStorage` read/write detected' },
  { token: 'localStorage.getItem', warning: '`localStorage` read/write detected' },
  { token: 'localStorage.removeItem', warning: '`localStorage` read/write detected' },
  { token: 'sessionStorage.', warning: '`sessionStorage` access detected' },
  { token: 'indexedDB.open', warning: '`indexedDB` access detected' },
  { token: 'indexedDB.', warning: '`indexedDB` access detected' },
  { token: 'new WebSocket', warning: '`WebSocket` connection detected' },
  { token: 'WebSocket(', warning: '`WebSocket` connection detected' },
  { token: 'eval(', warning: '`eval()` call detected' },
] as const

type AcornNode = import('acorn').Node

let acornLoader: Promise<{
  parse: typeof import('acorn').parse
  simple: typeof import('acorn-walk').simple
}> | null = null

function loadAcornParser() {
  if (!acornLoader) {
    acornLoader = Promise.all([
      import('acorn'),
      import('acorn-walk'),
    ]).then(([acorn, walk]) => ({
      parse: acorn.parse,
      simple: walk.simple,
    }))
  }
  return acornLoader
}

function extractScriptBodies(sfc: string): string[] {
  const out: string[] = []
  const re = /<script\b[^>]*>([\s\S]*?)<\/script>/gi
  let m: RegExpExecArray | null
  while ((m = re.exec(sfc)) !== null) {
    const body = m[1]?.trim()
    if (body)
      out.push(m[1]!)
  }
  return out.length > 0 ? out : [sfc]
}

function pushDedupe(set: Set<string>, list: string[], msg: string): void {
  if (!set.has(msg)) {
    set.add(msg)
    list.push(msg)
  }
}

function shouldRunAstScan(source: string): boolean {
  return DANGEROUS_PATTERNS.some(p => source.includes(p.token))
}

function scanByStringPatterns(source: string, dedupe: Set<string>, warnings: string[]): void {
  for (const p of DANGEROUS_PATTERNS) {
    if (source.includes(p.token)) {
      pushDedupe(dedupe, warnings, p.warning)
    }
  }
}

async function scanScriptAst(
  source: string,
  dedupe: Set<string>,
  warnings: string[],
): Promise<void> {
  const { parse, simple } = await loadAcornParser()
  let ast: AcornNode
  try {
    ast = parse(source, {
      ecmaVersion: 2024,
      sourceType: 'module',
    }) as AcornNode
  }
  catch {
    return
  }

  simple(ast, {
    MemberExpression(node: AcornNode) {
      const n = node as unknown as {
        object: { type: string, name?: string }
        property: { type: string, name?: string, value?: string }
      }
      if (
        n.object.type === 'Identifier'
        && n.object.name === 'window'
        && n.property.type === 'Identifier'
        && (n.property.name === '__TAURI__' || n.property.name === 'tauri')
      ) {
        pushDedupe(
          dedupe,
          warnings,
          'Access to `window.__TAURI__` / `window.tauri` detected',
        )
      }
      if (
        n.object.type === 'Identifier'
        && n.object.name === 'document'
        && n.property.type === 'Identifier'
        && n.property.name === 'cookie'
      ) {
        pushDedupe(dedupe, warnings, '`document.cookie` access detected')
      }
      if (
        n.object.type === 'Identifier'
        && n.object.name === 'localStorage'
        && n.property.type === 'Identifier'
        && (n.property.name === 'setItem' || n.property.name === 'getItem')
      ) {
        pushDedupe(dedupe, warnings, '`localStorage` read/write detected')
      }
    },
    CallExpression(node: AcornNode) {
      const n = node as unknown as {
        callee: { type: string, name?: string }
      }
      if (n.callee.type === 'Identifier' && n.callee.name === 'fetch') {
        pushDedupe(dedupe, warnings, '`fetch()` call detected')
      }
      if (n.callee.type === 'Identifier' && n.callee.name === 'eval') {
        pushDedupe(dedupe, warnings, '`eval()` call detected')
      }
    },
    NewExpression(node: AcornNode) {
      const n = node as unknown as {
        callee: { type: string, name?: string }
      }
      if (n.callee.type === 'Identifier' && n.callee.name === 'XMLHttpRequest') {
        pushDedupe(dedupe, warnings, '`XMLHttpRequest` usage detected')
      }
    },
  })
}

/** Static scan of `.vue` or plain script snippet (denylist); no zero false positive/negative guarantee. */
export async function scanVueComponentSource(source: string): Promise<ScanResult> {
  const warnings: string[] = []
  const dedupe = new Set<string>()
  if (!shouldRunAstScan(source)) {
    return { warnings }
  }
  scanByStringPatterns(source, dedupe, warnings)
  for (const block of extractScriptBodies(source)) {
    await scanScriptAst(block, dedupe, warnings)
  }
  return { warnings }
}
