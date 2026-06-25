#!/usr/bin/env node
/**
 * Remove theater template lines from daily chat mirror (mumu/home by default).
 * Matches messages containing theater markers like 「剧场即兴」 or speaker prefix 「木木：」.
 *
 * Usage:
 *   node scripts/clean-theater-chat-pollution.mjs
 *   node scripts/clean-theater-chat-pollution.mjs --dry-run
 *   node scripts/clean-theater-chat-pollution.mjs --role mumu --scene home
 */

import fs from 'node:fs'
import path from 'node:path'
import os from 'node:os'

const args = process.argv.slice(2)
const dryRun = args.includes('--dry-run')
const roleIdx = args.indexOf('--role')
const sceneIdx = args.indexOf('--scene')
const roleId = roleIdx >= 0 ? args[roleIdx + 1] : 'mumu'
const sceneId = sceneIdx >= 0 ? args[sceneIdx + 1] : 'home'

const POLLUTION_PATTERNS = [
  /【剧场即兴/u,
  /^木木[：:]/u,
  /剧场即兴 ·/u,
]

function defaultAppDataDir() {
  const local = process.env.LOCALAPPDATA
  if (local) return path.join(local, 'OCLive', 'data')
  return path.join(os.homedir(), '.local', 'share', 'OCLive', 'data')
}

function isPolluted(text) {
  const t = String(text ?? '')
  return POLLUTION_PATTERNS.some(re => re.test(t))
}

function walkJsonFiles(dir, out = []) {
  if (!fs.existsSync(dir)) return out
  for (const name of fs.readdirSync(dir)) {
    const p = path.join(dir, name)
    const st = fs.statSync(p)
    if (st.isDirectory()) walkJsonFiles(p, out)
    else if (name.endsWith('.json')) out.push(p)
  }
  return out
}

function cleanFile(filePath) {
  let raw
  try {
    raw = fs.readFileSync(filePath, 'utf8')
  }
  catch {
    return { removed: 0, skipped: true }
  }
  let data
  try {
    data = JSON.parse(raw)
  }
  catch {
    return { removed: 0, skipped: true }
  }

  let removed = 0
  const scrub = (obj) => {
    if (Array.isArray(obj)) {
      const kept = []
      for (const item of obj) {
        if (item && typeof item === 'object') {
          const text = item.content ?? item.text ?? item.reply ?? item.message
          if (typeof text === 'string' && isPolluted(text)) {
            removed += 1
            continue
          }
          scrub(item)
        }
        kept.push(item)
      }
      obj.length = 0
      obj.push(...kept)
      return
    }
    if (obj && typeof obj === 'object') {
      if (Array.isArray(obj.messages)) scrub(obj.messages)
      if (Array.isArray(obj.turns)) scrub(obj.turns)
      for (const v of Object.values(obj)) {
        if (v && typeof v === 'object') scrub(v)
      }
    }
  }

  scrub(data)
  if (removed > 0 && !dryRun) {
    fs.writeFileSync(filePath, `${JSON.stringify(data, null, 2)}\n`, 'utf8')
  }
  return { removed, skipped: false }
}

const chatRoot = path.join(defaultAppDataDir(), 'chats', roleId, sceneId)
const files = walkJsonFiles(chatRoot)
let totalRemoved = 0

console.log(`Scanning ${chatRoot} (${files.length} json files)${dryRun ? ' [dry-run]' : ''}`)

for (const f of files) {
  const { removed, skipped } = cleanFile(f)
  if (!skipped && removed > 0) {
    console.log(`${dryRun ? 'would clean' : 'cleaned'} ${removed} message(s): ${f}`)
    totalRemoved += removed
  }
}

if (totalRemoved === 0) {
  console.log('No polluted theater messages found.')
}
else {
  console.log(`${dryRun ? 'Would remove' : 'Removed'} ${totalRemoved} polluted message(s) total.`)
}
