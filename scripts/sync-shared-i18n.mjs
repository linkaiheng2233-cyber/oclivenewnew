/**
 * Mirror canonical shared i18n from oclivenewnew to sister repos.
 * Run from oclivenewnew root: node scripts/sync-shared-i18n.mjs
 */
import fs from 'node:fs'
import path from 'node:path'

const root = path.resolve(import.meta.dirname, '..')
const canonical = path.join(root, 'distros/shared/src/i18n/shared')
const mirrors = [
  path.join(root, '../oclive-launcher/src/i18n/shared'),
  path.join(root, '../oclive-pack-editor/src/i18n/shared'),
]

const files = fs.readdirSync(canonical).filter(f => f !== 'README.md')

for (const destRoot of mirrors) {
  fs.mkdirSync(destRoot, { recursive: true })
  for (const name of files) {
    fs.copyFileSync(path.join(canonical, name), path.join(destRoot, name))
  }
  console.log('synced', files.length, 'files ->', destRoot)
}
