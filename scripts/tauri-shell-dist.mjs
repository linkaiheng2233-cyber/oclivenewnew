#!/usr/bin/env node
/**
 * Resolve Tauri distDir + bundle resources from OCLIVE_TAURI_SHELL (chat-pro | theater).
 */
import fs from 'fs'
import path from 'path'
import { fileURLToPath } from 'url'

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const confPath = path.join(repoRoot, 'distros', 'desktop-tauri', 'tauri.conf.json')
const shell = process.env.OCLIVE_TAURI_SHELL === 'theater' ? 'theater' : 'chat-pro'

const productName = shell === 'theater' ? 'OCLive Theater' : 'OCLive Chat Pro'
const distDir = shell === 'theater' ? '../theater/dist' : '../chat-pro/dist'
const rolesResource = shell === 'theater' ? 'resources/roles' : '../chat-pro/roles'

const raw = fs.readFileSync(confPath, 'utf8')
const parsed = JSON.parse(raw)
parsed.build.distDir = distDir
parsed.package.productName = productName
parsed.tauri.windows[0].title = productName

const resources = parsed.tauri.bundle.resources.filter((e) => e !== '../roles' && e !== 'resources/roles')
resources.unshift(rolesResource)
parsed.tauri.bundle.resources = resources

fs.writeFileSync(confPath, `${JSON.stringify(parsed, null, 2)}\n`, 'utf8')
console.log(`[tauri-shell-dist] shell=${shell} distDir=${distDir}`)
