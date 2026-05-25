#!/usr/bin/env node
/**
 * Validate data/plugins.json against example manifests (id + version).
 * Usage: node scripts/validate-plugins-index.mjs [path-to-plugins.json]
 */
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const indexPath = process.argv[2]
  ? path.resolve(process.argv[2])
  : path.join(root, 'data', 'plugins.json')

function fail(msg) {
  console.error(`plugins-index: ${msg}`)
  process.exit(1)
}

const raw = fs.readFileSync(indexPath, 'utf8')
let data
try {
  data = JSON.parse(raw)
} catch (e) {
  fail(`invalid JSON: ${e.message}`)
}

const plugins = data.plugins
if (!Array.isArray(plugins) || plugins.length === 0) {
  fail('plugins[] must be a non-empty array')
}

const seen = new Set()
for (const p of plugins) {
  for (const key of ['id', 'name', 'version', 'git']) {
    if (!p[key] || String(p[key]).trim() === '') {
      fail(`entry missing ${key}: ${JSON.stringify(p)}`)
    }
  }
  if (seen.has(p.id)) fail(`duplicate id: ${p.id}`)
  seen.add(p.id)

  const git = String(p.git).trim()
  if (!/^https?:\/\//.test(git) && !git.startsWith('git@')) {
    fail(`git must be http(s) or git@ URL for ${p.id}`)
  }

  if (p.gitSubdir && p.git_subdir) {
    fail(`entry ${p.id} has both gitSubdir and git_subdir; keep only gitSubdir`)
  }
  const sub = p.gitSubdir ?? p.git_subdir
  if (sub) {
    const rel = String(sub).replace(/\\/g, '/').replace(/^\/+/, '')
    const manifestPath = path.join(root, rel, 'manifest.json')
    if (!fs.existsSync(manifestPath)) {
      fail(`gitSubdir manifest not found: ${manifestPath}`)
    }
    const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'))
    const mid = manifest.id ?? manifest.pluginId
    if (mid && mid !== p.id) {
      fail(`id mismatch index=${p.id} manifest=${mid} (${rel})`)
    }
    const mv = manifest.version
    if (mv && mv !== p.version) {
      fail(`version mismatch index=${p.version} manifest=${mv} (${rel})`)
    }
  }
}

console.log(`OK ${plugins.length} plugin(s) in ${indexPath}`)
