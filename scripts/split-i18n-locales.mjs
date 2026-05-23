/**
 * Extract top-level locale sections into fragments/*.zh.ts / *.en.ts
 * Run: node scripts/split-i18n-locales.mjs
 */
import fs from 'node:fs'
import path from 'node:path'

const root = path.resolve(import.meta.dirname, '..')
const fragDir = path.join(root, 'src/i18n/locales/fragments')

const sections = [
  { key: 'app', out: 'app' },
  { key: 'pluginManager', out: 'pluginManager', include: ['pluginTerms'] },
  { key: 'settings', out: 'settings', include: ['hotkeys'] },
  { key: 'common', out: 'common', include: ['relation'] },
  { key: 'roleRuntime', out: 'roleRuntime' },
  { key: 'editor', out: 'editor' },
]

function extractSection(source, key) {
  const startRe = new RegExp(`^  ${key}: \\{`, 'm')
  const start = source.search(startRe)
  if (start < 0) throw new Error(`missing ${key}`)
  let i = source.indexOf('{', start)
  let depth = 0
  let end = i
  for (; end < source.length; end += 1) {
    const c = source[end]
    if (c === '{') depth += 1
    else if (c === '}') {
      depth -= 1
      if (depth === 0) {
        end += 1
        break
      }
    }
  }
  return source.slice(i, end)
}

function writeFragment(locale, sectionKey, outName, bodyKeys, source) {
  const parts = bodyKeys.map(k => {
    const obj = extractSection(source, k)
    return `  ${k}: ${obj}`
  })
  const content = `/** ${outName} — ${locale}. */\nexport default {\n${parts.join(',\n')}\n}\n`
  const file = path.join(fragDir, `${outName}.${locale === 'zh' ? 'zh' : 'en'}.ts`)
  fs.writeFileSync(file, content, 'utf8')
  console.log('wrote', path.relative(root, file))
}

for (const locale of ['zh', 'en']) {
  const file = path.join(root, `src/i18n/locales/${locale === 'zh' ? 'zh-CN' : 'en-US'}.ts`)
  const source = fs.readFileSync(file, 'utf8')
  for (const sec of sections) {
    const keys = [sec.key, ...(sec.include ?? [])]
    writeFragment(locale, sec.key, sec.out, keys, source)
  }
}

const zhEntry = `import apiErrors from './fragments/apiErrors.zh'
import app from './fragments/app.zh'
import chat from './fragments/chat.zh'
import common from './fragments/common.zh'
import devTools from './fragments/devTools.zh'
import editor from './fragments/editor.zh'
import emotionUi from './fragments/emotionUi.zh'
import pluginManager from './fragments/pluginManager.zh'
import pluginWorkbench from './fragments/pluginWorkbench.zh'
import roleRuntime from './fragments/roleRuntime.zh'
import settings from './fragments/settings.zh'
import { simplePluginManagerZh as simplePluginManager } from './fragments/simplePluginManager.zh'
import virtualTime from './fragments/virtualTime.zh'

export default {
  apiErrors,
  app,
  chat,
  common,
  devTools,
  editor,
  emotionUi,
  pluginManager,
  pluginWorkbench,
  roleRuntime,
  settings,
  simplePluginManager,
  virtualTime,
}
`

const enEntry = zhEntry
  .replace(/\.zh/g, '.en')
  .replace('simplePluginManagerZh', 'simplePluginManagerEn')

fs.writeFileSync(path.join(root, 'src/i18n/locales/zh-CN.ts'), zhEntry, 'utf8')
fs.writeFileSync(path.join(root, 'src/i18n/locales/en-US.ts'), enEntry, 'utf8')
console.log('updated locale entry files')
