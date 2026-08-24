#!/usr/bin/env node
/**
 * Stage only Git-tracked Chat Pro role files into Tauri's resource_dir/roles.
 * This keeps ignored local chat/plugin state out of desktop installers.
 */
import { spawnSync } from 'child_process'
import fs from 'fs'
import path from 'path'
import { fileURLToPath } from 'url'

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const sourcePrefix = 'distros/chat-pro/roles'
const sourceRoot = path.join(repoRoot, ...sourcePrefix.split('/'))
const destinationRoot = path.join(
  repoRoot,
  'distros',
  'desktop-tauri',
  'resources',
  'roles',
)

if (process.env.OCLIVE_TAURI_SHELL === 'theater') {
  console.log('[stage-chat-pro-roles] theater shell keeps its filtered cast')
  process.exit(0)
}

const listed = spawnSync(
  'git',
  ['ls-files', '-z', '--', sourcePrefix],
  { cwd: repoRoot, encoding: 'utf8', windowsHide: true },
)
if (listed.status !== 0) {
  throw new Error(
    `git ls-files failed: ${listed.stderr || listed.stdout || 'unknown error'}`,
  )
}

const listedFiles = listed.stdout.split('\0').filter(Boolean)
if (listedFiles.length === 0) {
  throw new Error(`no tracked role files found under ${sourcePrefix}`)
}

const localStateSegment = '/.oclive_directory_plugin_data/'
const trackedLocalState = listedFiles.find((file) =>
  `/${file.replaceAll('\\', '/')}`.includes(localStateSegment),
)
if (trackedLocalState) {
  throw new Error(`refusing to bundle local role state: ${trackedLocalState}`)
}

// `git ls-files` also reports tracked paths deleted in the current working
// tree. Release staging follows the current source tree, including intentional
// uncommitted deletions, instead of failing halfway through a copy.
const existingTrackedFiles = listedFiles.filter((file) =>
  fs.existsSync(path.join(repoRoot, ...file.replaceAll('\\', '/').split('/'))),
)

// Production role packs always have an immediate author.json. Root-level
// templates/docs, blueprint/, and polish-dev/ are development inputs rather
// than characters shipped to end users.
const productionRoleIds = new Set(
  existingTrackedFiles.flatMap((file) => {
    const normalized = file.replaceAll('\\', '/')
    const relative = path.posix.relative(sourcePrefix, normalized)
    const parts = relative.split('/')
    return parts.length === 2 && parts[1] === 'author.json' ? [parts[0]] : []
  }),
)
if (productionRoleIds.size === 0) {
  throw new Error(`no production role packs found under ${sourcePrefix}`)
}

const trackedFiles = existingTrackedFiles.filter((file) => {
  const normalized = file.replaceAll('\\', '/')
  const relative = path.posix.relative(sourcePrefix, normalized)
  const roleId = relative.split('/')[0]
  return productionRoleIds.has(roleId)
})

fs.rmSync(destinationRoot, { recursive: true, force: true })
fs.mkdirSync(destinationRoot, { recursive: true })

for (const trackedFile of trackedFiles) {
  const normalized = trackedFile.replaceAll('\\', '/')
  const relative = path.posix.relative(sourcePrefix, normalized)
  if (relative.startsWith('../') || path.posix.isAbsolute(relative)) {
    throw new Error(`role file escaped source root: ${trackedFile}`)
  }

  const source = path.join(sourceRoot, ...relative.split('/'))
  const destination = path.join(destinationRoot, ...relative.split('/'))
  const sourceStat = fs.statSync(source)
  if (!sourceStat.isFile()) continue
  fs.mkdirSync(path.dirname(destination), { recursive: true })
  fs.copyFileSync(source, destination)
}

console.log(
  `[stage-chat-pro-roles] staged ${trackedFiles.length} tracked files from ${productionRoleIds.size} production roles -> ${destinationRoot}`,
)
