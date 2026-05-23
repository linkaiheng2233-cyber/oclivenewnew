/**
 * Verify sister repos mirror oclivenewnew/src/i18n/shared byte-for-byte.
 */
import fs from 'node:fs'
import path from 'node:path'

const root = path.resolve(import.meta.dirname, '..')
const canonicalDir = path.join(root, 'src/i18n/shared')
const mirrors = [
  ['oclive-launcher', path.join(root, '../oclive-launcher/src/i18n/shared')],
  ['oclive-pack-editor', path.join(root, '../oclive-pack-editor/src/i18n/shared')],
]

const files = fs
  .readdirSync(canonicalDir)
  .filter(f => f.endsWith('.ts'))

let failed = false

for (const name of files) {
  const canonical = fs.readFileSync(path.join(canonicalDir, name), 'utf8')
  for (const [repo, dir] of mirrors) {
    const target = path.join(dir, name)
    if (!fs.existsSync(target)) {
      console.error(`missing ${repo}: src/i18n/shared/${name}`)
      failed = true
      continue
    }
    const mirror = fs.readFileSync(target, 'utf8')
    if (mirror !== canonical) {
      console.error(`drift ${repo}: src/i18n/shared/${name}`)
      failed = true
    }
  }
}

if (failed) {
  console.error('Shared i18n out of sync. Run: node scripts/sync-shared-i18n.mjs')
  process.exit(1)
}

console.log(`shared i18n OK (${files.length} files × ${mirrors.length} mirrors)`)
