#!/usr/bin/env node
/**
 * Copy theater cast role packs into Tauri resources for bundled installs.
 * Keeps only mumu + 枫侵月 (cast A/B).
 */
import fs from 'fs'
import path from 'path'
import { fileURLToPath } from 'url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const repoRoot = path.resolve(__dirname, '..')
const rolesRoot = path.join(repoRoot, 'roles')
const destRoot = path.join(repoRoot, 'src-tauri', 'resources', 'roles')
const CAST_ROLE_IDS = ['mumu', '枫侵月']

function copyDir(src, dest) {
  fs.mkdirSync(dest, { recursive: true })
  for (const entry of fs.readdirSync(src, { withFileTypes: true })) {
    const from = path.join(src, entry.name)
    const to = path.join(dest, entry.name)
    if (entry.isDirectory())
      copyDir(from, to)
    else
      fs.copyFileSync(from, to)
  }
}

if (!fs.existsSync(rolesRoot)) {
  console.error('[filter-theater-roles] missing roles root:', rolesRoot)
  process.exit(1)
}

if (fs.existsSync(destRoot))
  fs.rmSync(destRoot, { recursive: true, force: true })

fs.mkdirSync(destRoot, { recursive: true })

for (const roleId of CAST_ROLE_IDS) {
  const src = path.join(rolesRoot, roleId)
  if (!fs.existsSync(src)) {
    console.error(`[filter-theater-roles] missing role pack: ${roleId}`)
    process.exit(1)
  }
  copyDir(src, path.join(destRoot, roleId))
  console.log(`[filter-theater-roles] copied ${roleId}`)
}

const skeletonDir = path.join(repoRoot, 'src-tauri', 'resources', 'theater', 'scenes')
const skeletonPublicDir = path.join(repoRoot, 'public', 'theater', 'scenes')
if (fs.existsSync(skeletonDir)) {
  fs.mkdirSync(skeletonPublicDir, { recursive: true })
  for (const entry of fs.readdirSync(skeletonDir)) {
    if (entry.endsWith('.skeleton.json') || entry === 'index.json') {
      fs.copyFileSync(
        path.join(skeletonDir, entry),
        path.join(skeletonPublicDir, entry),
      )
    }
  }
}

// Legacy single-file path (dev caches)
const legacySkeleton = path.join(repoRoot, 'public', 'theater', 'breakfast.skeleton.json')
const breakfastScene = path.join(skeletonPublicDir, 'breakfast.skeleton.json')
if (fs.existsSync(breakfastScene) && !fs.existsSync(legacySkeleton)) {
  fs.mkdirSync(path.dirname(legacySkeleton), { recursive: true })
  fs.copyFileSync(breakfastScene, legacySkeleton)
}

const officialPluginSrc = path.join(
  repoRoot,
  'plugins',
  'com.oclive.theater_director_official',
)
const officialPluginDest = path.join(
  repoRoot,
  'src-tauri',
  'resources',
  'plugins',
  'com.oclive.theater_director_official',
)
if (fs.existsSync(officialPluginSrc)) {
  if (fs.existsSync(officialPluginDest))
    fs.rmSync(officialPluginDest, { recursive: true, force: true })
  copyDir(officialPluginSrc, officialPluginDest)
  console.log('[filter-theater-roles] copied official theater director plugin')
}

console.log('[filter-theater-roles] done')
