#!/usr/bin/env node
/**
 * T4-PKG: Copy theater role subset into Tauri resources for release bundles.
 * Dev continues to use full repo roles/ via OCLIVE_ROLES_DIR or debug heuristics.
 */
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const THEATER_ROLE_IDS = ['theater-breakfast-a', 'theater-breakfast-b']
const srcRolesRoot = path.join(root, 'roles')
const destRolesRoot = path.join(root, 'src-tauri', 'resources', 'roles')

function copyDirRecursive(src, dest) {
  fs.mkdirSync(dest, { recursive: true })
  for (const entry of fs.readdirSync(src, { withFileTypes: true })) {
    const from = path.join(src, entry.name)
    const to = path.join(dest, entry.name)
    if (entry.isDirectory()) {
      copyDirRecursive(from, to)
    }
    else {
      fs.copyFileSync(from, to)
    }
  }
}

if (fs.existsSync(destRolesRoot)) {
  fs.rmSync(destRolesRoot, { recursive: true, force: true })
}
fs.mkdirSync(destRolesRoot, { recursive: true })

for (const roleId of THEATER_ROLE_IDS) {
  const src = path.join(srcRolesRoot, roleId)
  if (!fs.existsSync(src)) {
    console.error(`[filter-theater-roles] missing role: ${src}`)
    process.exit(1)
  }
  const blueprint = path.join(src, 'pipeline.ocblueprint')
  if (!fs.existsSync(blueprint)) {
    console.error(`[filter-theater-roles] missing blueprint: ${blueprint}`)
    process.exit(1)
  }
  copyDirRecursive(src, path.join(destRolesRoot, roleId))
  console.info(`[filter-theater-roles] copied ${roleId}`)
}

const marker = path.join(root, 'src-tauri', 'resources', '.theater-roles-bundle')
fs.writeFileSync(marker, `roles=${THEATER_ROLE_IDS.join(',')}\n`, 'utf8')
console.info(`[filter-theater-roles] done -> ${destRolesRoot}`)
