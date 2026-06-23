#!/usr/bin/env node
/**
 * Theater 15s engineering proxy — validates skeleton + simulates poke fork insert.
 * Not a substitute for stranger acceptance (P7).
 *
 * Optional: `--cast-a <roleId>` / `--cast-b <roleId>` for local cast binding smoke.
 */
import fs from 'fs'
import path from 'path'
import { fileURLToPath } from 'url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const repoRoot = path.resolve(__dirname, '..')
const skeletonPath = path.join(repoRoot, 'distros', 'theater', 'public', 'theater', 'breakfast.skeleton.json')

function parseArgs(argv) {
  const out = { castA: null, castB: null }
  for (let i = 2; i < argv.length; i += 1) {
    const arg = argv[i]
    if (arg === '--cast-a' && argv[i + 1]) {
      out.castA = argv[++i]
    }
    else if (arg === '--cast-b' && argv[i + 1]) {
      out.castB = argv[++i]
    }
  }
  return out
}

function fail(msg) {
  console.error(`[theater-smoke] FAIL: ${msg}`)
  process.exit(1)
}

if (!fs.existsSync(skeletonPath))
  fail(`missing ${skeletonPath}`)

const { castA, castB } = parseArgs(process.argv)

const skeleton = JSON.parse(fs.readFileSync(skeletonPath, 'utf8'))

if (skeleton.scene !== 'breakfast')
  fail('scene must be breakfast')

const expectedCastA = castA ?? 'mumu'
const expectedCastB = castB ?? '枫侵月'

if (skeleton.cast?.a?.roleId !== expectedCastA || skeleton.cast?.b?.roleId !== expectedCastB) {
  if (castA || castB) {
    fail(`cast override expected ${expectedCastA} × ${expectedCastB}, got ${skeleton.cast?.a?.roleId} × ${skeleton.cast?.b?.roleId}`)
  }
  fail('cast must be mumu × 枫侵月')
}

if (skeleton.cast?.a?.side !== 'left' || skeleton.cast?.b?.side !== 'right')
  fail('cast must declare side left/right')
if (!Array.isArray(skeleton.beats) || skeleton.beats.length < 8)
  fail('beats must have at least 8 lines')

const chipIds = ['tea', 'late', 'biteTongue', 'nickname']
for (const chipId of chipIds) {
  const forks = skeleton.forks?.[chipId]
  if (!Array.isArray(forks) || forks.length < 1)
    fail(`forks.${chipId} missing`)
  const fork = forks[0]
  if (!fork.insertAfterBeatId || !Array.isArray(fork.patchLines) || fork.patchLines.length < 1)
    fail(`forks.${chipId}[0] invalid`)
  if (fork.insertAfterBeatId === 'b10')
    fail(`forks.${chipId} must insert at mid anchor, not b10 tail`)
}

let lines = skeleton.beats.map(b => ({ ...b }))
const pokeChip = 'tea'
const fork = skeleton.forks[pokeChip][0]
const idx = lines.findIndex(l => l.id === fork.insertAfterBeatId)
if (idx < 0)
  fail(`insertAfterBeatId ${fork.insertAfterBeatId} not in beats`)
lines = [
  ...lines.slice(0, idx + 1),
  ...fork.patchLines,
  ...lines.slice(idx + 1),
]

if (lines.length <= skeleton.beats.length)
  fail('poke insert did not grow script')

const profilePath = path.join(repoRoot, 'examples', 'distro-profiles', 'theater.oclive.toml')
if (!fs.existsSync(profilePath))
  fail('missing theater.oclive.toml profile')

const shellFile = path.join(repoRoot, 'distros', 'theater', 'src', 'shells', 'theater', 'TheaterShell.vue')
if (!fs.existsSync(shellFile))
  fail('missing TheaterShell.vue')

const castConfigFile = path.join(repoRoot, 'distros', 'theater', 'src', 'composables', 'theater', 'theaterCastConfig.ts')
if (!fs.existsSync(castConfigFile))
  fail('missing theaterCastConfig.ts')

console.log('[theater-smoke] skeleton ok')
console.log('[theater-smoke] poke insert ok', { chip: pokeChip, lines: lines.length })
console.log('[theater-smoke] profile + shell present')
if (castA || castB) {
  console.log('[theater-smoke] cast override mode', { castA: expectedCastA, castB: expectedCastB })
}
console.log('[theater-smoke] PASS')
