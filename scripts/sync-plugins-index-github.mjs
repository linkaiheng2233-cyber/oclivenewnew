#!/usr/bin/env node
/**
 * Convert data/plugins.json → awesome-oclive-plugins plugins.json shape.
 *
 *   node scripts/sync-plugins-index-github.mjs
 *   node scripts/sync-plugins-index-github.mjs --write ../awesome-oclive-plugins/plugins.json
 */
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const src = path.join(root, 'data', 'plugins.json')
const writeArg = process.argv.indexOf('--write')
const outPath =
  writeArg >= 0 ? path.resolve(process.argv[writeArg + 1]) : null

const data = JSON.parse(fs.readFileSync(src, 'utf8'))
const generatedAt =
  data.generatedAt ?? data.generated_at ?? new Date().toISOString()

const awesome = {
  version: '1',
  generated_at: generatedAt,
  plugins: (data.plugins ?? []).map((p) => {
    const row = { ...p }
    // SSOT 仅用 camelCase gitSubdir；勿再写 git_subdir，避免同条双字段。
    delete row.git_subdir
    return row
  }),
}

const text = `${JSON.stringify(awesome, null, 2)}\n`
if (outPath) {
  fs.mkdirSync(path.dirname(outPath), { recursive: true })
  fs.writeFileSync(outPath, text)
  console.log(`Wrote ${outPath}`)
} else {
  process.stdout.write(text)
}
