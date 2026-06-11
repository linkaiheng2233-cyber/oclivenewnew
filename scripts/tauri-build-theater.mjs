#!/usr/bin/env node
/**
 * Theater Tauri release build: filter roles subset + bundle kernel + tauri build.
 */
import { execSync } from 'node:child_process'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')

execSync('node scripts/filter-theater-roles.mjs', { cwd: root, stdio: 'inherit' })
execSync('node scripts/bundle-kernel-for-tauri.mjs', { cwd: root, stdio: 'inherit' })

const env = { ...process.env, OCLIVE_TAURI_SHELL: 'theater', VITE_OCLIVE_SHELL: 'theater' }
execSync('npx tauri build --config src-tauri/tauri.theater.conf.json', {
  cwd: root,
  stdio: 'inherit',
  env,
})
