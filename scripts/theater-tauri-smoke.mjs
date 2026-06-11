#!/usr/bin/env node
/**
 * T4-PKG-01: Theater distro smoke — validates shell env, bundled profile, unit tests.
 * Full Windows installer build: npm run tauri:build with VITE_OCLIVE_SHELL=theater.
 */
import { execSync } from 'node:child_process'
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')

function assert(condition, message) {
  if (!condition) {
    console.error(`[theater-smoke] FAIL: ${message}`)
    process.exit(1)
  }
}

function read(rel) {
  return fs.readFileSync(path.join(root, rel), 'utf8')
}

console.info('[theater-smoke] checking theater distro prerequisites…')

const envTheater = read('.env.theater').trim()
assert(envTheater.includes('VITE_OCLIVE_SHELL=theater'), '.env.theater must set VITE_OCLIVE_SHELL=theater')

assert(
  fs.existsSync(path.join(root, 'src-tauri/tauri.theater.conf.json')),
  'tauri.theater.conf.json missing',
)
assert(
  fs.existsSync(path.join(root, 'scripts/filter-theater-roles.mjs')),
  'filter-theater-roles.mjs missing',
)

function tomlBody(text) {
  return text
    .split(/\r?\n/)
    .map(line => line.trim())
    .filter(line => line && !line.startsWith('#'))
    .join('\n')
}

const exampleProfile = tomlBody(read('examples/distro-profiles/theater.oclive.toml'))
const bundledProfile = tomlBody(read('src-tauri/resources/distro-profiles/theater.oclive.toml'))
assert(exampleProfile === bundledProfile, 'bundled theater.oclive.toml body must match examples/distro-profiles/theater.oclive.toml')

assert(fs.existsSync(path.join(root, 'public/theater/breakfast/skeleton.json')), 'breakfast skeleton missing')
assert(fs.existsSync(path.join(root, 'public/theater/scenes.json')), 'theater scenes index missing')

console.info('[theater-smoke] filtering theater roles subset…')
execSync('node scripts/filter-theater-roles.mjs', { cwd: root, stdio: 'inherit' })
const bundledRoles = path.join(root, 'src-tauri/resources/roles')
for (const id of ['theater-breakfast-a', 'theater-breakfast-b']) {
  assert(
    fs.existsSync(path.join(bundledRoles, id, 'pipeline.ocblueprint')),
    `bundled role missing: ${id}`,
  )
}
const bundledIds = fs.readdirSync(bundledRoles, { withFileTypes: true })
  .filter(e => e.isDirectory())
  .map(e => e.name)
assert(bundledIds.length === 2, `expected 2 bundled roles, got ${bundledIds.length}`)

console.info('[theater-smoke] running theater unit tests…')
execSync('npm run test:unit -- src/theater/', { cwd: root, stdio: 'inherit' })

console.info('[theater-smoke] running 15s engineering proxy…')
execSync('node scripts/theater-stranger-proxy.mjs', { cwd: root, stdio: 'inherit' })

console.info('[theater-smoke] PASS')
console.info('[theater-smoke] Tauri theater install package (Windows):')
console.info('  npm run tauri:build:theater')
console.info('[theater-smoke] Or manual:')
console.info('  node scripts/filter-theater-roles.mjs')
console.info('  $env:OCLIVE_TAURI_SHELL = "theater"')
console.info('  $env:VITE_OCLIVE_SHELL = "theater"')
console.info('  npm run tauri:build -- --config src-tauri/tauri.theater.conf.json')
