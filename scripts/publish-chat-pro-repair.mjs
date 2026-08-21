#!/usr/bin/env node
/** Copy the external recovery wrapper next to every generated Windows installer. */
import fs from 'fs'
import path from 'path'
import { fileURLToPath } from 'url'
import { cargoTargetDir } from './lib/e2e-binary.mjs'

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const sourceDir = path.join(
  repoRoot,
  'distros',
  'desktop-tauri',
  'resources',
  'support',
)
const sourceFiles = ['Repair-AILiveChatPro.cmd', 'Repair-AILiveChatPro.ps1']
const targetRoot = cargoTargetDir(repoRoot)
if (!targetRoot) {
  throw new Error('cannot resolve Cargo target directory')
}

const installerDirs = ['nsis', 'msi']
  .map(kind => path.join(targetRoot, 'release', 'bundle', kind))
  .filter(directory => fs.existsSync(directory))
if (installerDirs.length === 0) {
  throw new Error(`no generated Windows installer directory under ${targetRoot}`)
}

for (const directory of installerDirs) {
  for (const file of sourceFiles) {
    const source = path.join(sourceDir, file)
    const destination = path.join(directory, file)
    fs.copyFileSync(source, destination)
    console.log(`[publish-chat-pro-repair] ${destination}`)
  }
}
