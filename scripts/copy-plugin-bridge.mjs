import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = path.dirname(fileURLToPath(import.meta.url))
const src = path.join(root, '..', 'dist', 'plugin-bridge', 'plugin-bridge.iife.js')
const dest = path.join(root, '..', 'distros', 'desktop-tauri', 'assets', 'plugin-bridge.iife.js')
fs.mkdirSync(path.dirname(dest), { recursive: true })
fs.copyFileSync(src, dest)
