import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

const SRC_ROOT = path.resolve(import.meta.dirname, '..')
const HAN = /[\u4E00-\u9FFF\u3400-\u4DBF\uF900-\uFAFF]/

function listSourceFiles(dir: string, out: string[] = []): string[] {
  for (const name of fs.readdirSync(dir)) {
    const full = path.join(dir, name)
    const stat = fs.statSync(full)
    if (stat.isDirectory()) {
      if (name === 'i18n')
        continue
      listSourceFiles(full, out)
      continue
    }
    if (/\.(?:vue|ts)$/.test(name))
      out.push(full)
  }
  return out
}

/** Strip block/line comments and template literals to reduce false positives. */
function stripNonUiLiterals(source: string): string {
  let s = source.replace(/\/\*[\s\S]*?\*\//g, '')
  s = s.replace(/\/\/[^\n]*/g, '')
  s = s.replace(/<!--[\s\S]*?-->/g, '')
  s = s.replace(/`(?:\\[\s\S]|[^\\`])*`/g, '""')
  return s
}

const ALLOWLIST_PATHS = new Set([
  'utils/roleplayReplySplit.ts',
  'utils/identitySurpriseTriggers.ts',
  'composables/theater/theaterPortrait.ts',
])

function lineHasHanOutsideString(line: string): boolean {
  const trimmed = line.trim()
  if (!HAN.test(trimmed))
    return false
  const quoted = [...trimmed.matchAll(/"([^"\\]|\\.)*"|'([^'\\]|\\.)*'/g)].map(m => m[0])
  const withoutQuoted = quoted.reduce((acc, q) => acc.replace(q, '""'), trimmed)
  return HAN.test(withoutQuoted)
}

describe('i18n hardcoded UI Chinese guard (A6.1)', () => {
  it('has no user-visible Han outside i18n locale catalogs', () => {
    const offenders: string[] = []
    for (const file of listSourceFiles(SRC_ROOT)) {
      const rel = path.relative(SRC_ROOT, file).replace(/\\/g, '/')
      if (rel.startsWith('i18n/locales/'))
        continue
      if (ALLOWLIST_PATHS.has(rel))
        continue
      const raw = fs.readFileSync(file, 'utf8')
      const stripped = stripNonUiLiterals(raw)
      const lines = stripped.split(/\r?\n/)
      lines.forEach((line, idx) => {
        if (lineHasHanOutsideString(line)) {
          offenders.push(`${rel}:${idx + 1}: ${line.trim().slice(0, 120)}`)
        }
      })
    }
    expect(
      offenders,
      `Move user-visible copy to i18n locales:\n${offenders.join('\n')}`,
    ).toEqual([])
  })
})
