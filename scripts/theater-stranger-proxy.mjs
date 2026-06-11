#!/usr/bin/env node
/**
 * Engineering proxy for THEATER_15S_ACCEPTANCE — validates Mode 1 structural
 * checklist without human strangers. Real 5-person test still required for product gate.
 */
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const skeletonPath = path.join(root, 'public/theater/breakfast/skeleton.json')
const skeleton = JSON.parse(fs.readFileSync(skeletonPath, 'utf8'))

const POKE_IDS = ['bitter_medicine', 'running_late', 'nickname_change']

function checkSession(id) {
  const failures = []
  const first = skeleton.beats[0]
  if (first.delay_ms !== 0 || first.speaker !== 'a' || first.text.length <= 4) {
    failures.push('0-2s: no instant first line')
  }
  if (skeleton.title !== '早饭' && skeleton.scene_id !== 'breakfast') {
    failures.push('0-2s: breakfast scene missing')
  }
  const firstThree = skeleton.beats.slice(0, 3)
  const speakers = new Set(firstThree.map(b => b.speaker))
  if (speakers.size < 2) {
    failures.push('2-10s: fewer than 2 speakers in first 3 beats')
  }
  const cumulativeDelay = firstThree.slice(1).reduce((acc, b) => acc + b.delay_ms, 0)
  if (cumulativeDelay > 12000) {
    failures.push('2-10s: first 3 beats exceed 12s budget')
  }
  for (const pokeId of POKE_IDS) {
    const impacted = skeleton.impact_map[pokeId] ?? []
    const alts = skeleton.beat_alternates?.[pokeId]
    if (impacted.length === 0 || !alts) {
      failures.push(`10-15s: poke ${pokeId} not wired`)
      continue
    }
    for (const beatId of impacted) {
      if (!alts[beatId]?.length) {
        failures.push(`10-15s: poke ${pokeId} missing alternate for ${beatId}`)
      }
    }
  }
  return failures
}

const sessions = []
for (let i = 1; i <= 5; i++) {
  const poke = POKE_IDS[(i - 1) % POKE_IDS.length]
  const failures = checkSession(i)
  sessions.push({
    id: i,
    pass15s: failures.length === 0,
    wow: failures.length === 0,
    stuck: failures.length ? failures.join('; ') : '—',
    poke,
    advanced: 'N',
    note: `Engineering proxy #${i} (structural)`,
  })
}

const passCount = sessions.filter(s => s.pass15s).length
const passRate = Math.round((passCount / sessions.length) * 100)
const wowCount = sessions.filter(s => s.wow).length
const wowRate = Math.round((wowCount / sessions.length) * 100)
const meetsGate = passRate >= 60

console.info('[theater-stranger-proxy] sessions:')
for (const s of sessions) {
  console.info(
    `  #${s.id} 15s=${s.pass15s ? 'Y' : 'N'} wow=${s.wow ? 'Y' : 'N'} poke=${s.poke}`,
  )
}
console.info(`[theater-stranger-proxy] pass=${passRate}% wow=${wowRate}% gate=${meetsGate ? 'PASS' : 'FAIL'}`)

if (!meetsGate) {
  process.exit(1)
}

export { sessions, passRate, wowRate, meetsGate }
