#!/usr/bin/env node
/**
 * Stage the production Chat Pro directory plugins into Tauri resources.
 *
 * Only Git-tracked files from the explicit allowlist are copied. This keeps
 * developer fixtures and ignored local plugin state out of desktop installers.
 */
import { spawnSync } from 'child_process'
import fs from 'fs'
import path from 'path'
import { fileURLToPath } from 'url'

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const sourcePrefix = 'distros/chat-pro/plugins'
const sourceRoot = path.join(repoRoot, ...sourcePrefix.split('/'))
const destinationRoot = path.join(
  repoRoot,
  'distros',
  'desktop-tauri',
  'resources',
  'plugins',
)

const productionPluginIds = [
  'com.oclive.mumu.chat-header-status',
  'com.oclive.mumu.quick-actions',
  'com.oclive.mumu.role-detail-card',
  'com.oclive.mumu.settings-panel',
  'com.oclive.mumu.sidebar-glance',
  'com.oclive.theater_director_official',
  'com.oclive.voice.asr',
]

if (process.env.OCLIVE_TAURI_SHELL === 'theater') {
  console.log('[stage-chat-pro-plugins] theater shell keeps its filtered plugins')
  process.exit(0)
}

const listed = spawnSync(
  'git',
  ['ls-files', '-z', '--', ...productionPluginIds.map((id) => `${sourcePrefix}/${id}`)],
  { cwd: repoRoot, encoding: 'utf8', windowsHide: true },
)
if (listed.status !== 0) {
  throw new Error(
    `git ls-files failed: ${listed.stderr || listed.stdout || 'unknown error'}`,
  )
}

const trackedFiles = listed.stdout.split('\0').filter(Boolean)
for (const pluginId of productionPluginIds) {
  const manifestRelative = `${sourcePrefix}/${pluginId}/manifest.json`
  if (!trackedFiles.includes(manifestRelative)) {
    throw new Error(`production plugin is missing a tracked manifest: ${pluginId}`)
  }
  const manifest = JSON.parse(
    fs.readFileSync(path.join(sourceRoot, pluginId, 'manifest.json'), 'utf8'),
  )
  if (manifest.id !== pluginId) {
    throw new Error(
      `production plugin manifest id mismatch: expected ${pluginId}, got ${manifest.id}`,
    )
  }
}

fs.rmSync(destinationRoot, { recursive: true, force: true })
fs.mkdirSync(destinationRoot, { recursive: true })

for (const trackedFile of trackedFiles) {
  const normalized = trackedFile.replaceAll('\\', '/')
  const relative = path.posix.relative(sourcePrefix, normalized)
  if (relative.startsWith('../') || path.posix.isAbsolute(relative)) {
    throw new Error(`plugin file escaped source root: ${trackedFile}`)
  }

  const source = path.join(sourceRoot, ...relative.split('/'))
  const destination = path.join(destinationRoot, ...relative.split('/'))
  const sourceStat = fs.statSync(source)
  if (!sourceStat.isFile()) continue
  fs.mkdirSync(path.dirname(destination), { recursive: true })
  fs.copyFileSync(source, destination)
}

console.log(
  `[stage-chat-pro-plugins] staged ${trackedFiles.length} tracked files from ${productionPluginIds.length} production plugins -> ${destinationRoot}`,
)
