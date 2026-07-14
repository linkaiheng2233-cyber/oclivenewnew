#!/usr/bin/env node
/**
 * Resolve Tauri frontendDist + bundle resources from OCLIVE_TAURI_SHELL (chat-pro | theater).
 * Mutates distros/desktop-tauri/tauri.conf.json in place for the active shell.
 */
import fs from 'fs'
import path from 'path'
import { fileURLToPath } from 'url'

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const confPath = path.join(repoRoot, 'distros', 'desktop-tauri', 'tauri.conf.json')
const shell = process.env.OCLIVE_TAURI_SHELL === 'theater' ? 'theater' : 'chat-pro'

const productName = shell === 'theater' ? 'OCLive Theater' : 'OCLive Chat Pro'
const frontendDist = shell === 'theater' ? '../theater/dist' : '../chat-pro/dist'
const rolesResource = shell === 'theater' ? 'resources/roles' : '../chat-pro/roles'

const raw = fs.readFileSync(confPath, 'utf8')
const parsed = JSON.parse(raw)
parsed.productName = productName
parsed.build = parsed.build || {}
parsed.build.frontendDist = frontendDist
if (parsed.build.distDir !== undefined) {
  delete parsed.build.distDir
}
if (parsed.app?.windows?.[0]) {
  parsed.app.windows[0].title = productName
}

const ROLE_PATHS = new Set(['../roles', '../chat-pro/roles', 'resources/roles'])
const bundle = parsed.bundle || {}
const resources = (bundle.resources || []).filter((e) => !ROLE_PATHS.has(e))
resources.unshift(rolesResource)
bundle.resources = resources
parsed.bundle = bundle

fs.writeFileSync(confPath, `${JSON.stringify(parsed, null, 2)}\n`, 'utf8')
console.log(`[tauri-shell-dist] shell=${shell} frontendDist=${frontendDist}`)
