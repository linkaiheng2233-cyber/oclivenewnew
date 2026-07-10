#!/usr/bin/env node
/**
 * Lightweight voice/TTS ratchet (no GPU, no sidecar spawn).
 * - rpc_server.mjs syntax
 * - Python TtsEngine registry >= 9 engines
 */
import { spawnSync } from 'child_process'
import fs from 'fs'
import path from 'path'
import { fileURLToPath } from 'url'

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const rpcServer = path.join(
  repoRoot,
  'distros/chat-pro/plugins/com.oclive.voice.asr/rpc_server.mjs',
)
const voiceLoop = path.join(repoRoot, 'examples/voice-loop-minimal')
const MIN_ENGINES = 9

function fail(msg) {
  console.error(`[voice-tts-ratchet] ${msg}`)
  process.exit(1)
}

if (!fs.existsSync(rpcServer)) {
  fail(`missing ${rpcServer}`)
}

const syntax = spawnSync(process.execPath, ['--check', rpcServer], {
  cwd: repoRoot,
  encoding: 'utf8',
})
if (syntax.status !== 0) {
  fail(syntax.stderr?.trim() || 'rpc_server.mjs syntax check failed')
}

const manifestPath = path.join(
  repoRoot,
  'distros/chat-pro/plugins/com.oclive.voice.asr/manifest.json',
)
const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'))
for (const method of ['voice.import_tts_adapter', 'voice.list_tts_adapters']) {
  if (!manifest.rpcMethods?.includes(method)) {
    fail(`manifest.json missing rpcMethods entry: ${method}`)
  }
}

const pySnippet = `import sys; sys.path.insert(0, r'${voiceLoop.replace(/\\/g, '/')}'); from tts.engines.registry import get_registry; n=len(get_registry().list_engine_ids()); assert n>=${MIN_ENGINES}, f'expected>=${MIN_ENGINES}, got {n}'; print('engines', n)`
const customPy = process.env.OCLIVE_VOICE_PYTHON?.trim()
const pyCmd = customPy || (process.platform === 'win32' ? 'py' : 'python3')
const pyArgs = customPy
  ? ['-c', pySnippet]
  : process.platform === 'win32'
    ? ['-3', '-c', pySnippet]
    : ['-c', pySnippet]

const registry = spawnSync(pyCmd, pyArgs, {
  cwd: voiceLoop,
  encoding: 'utf8',
  env: { ...process.env, PYTHONPATH: voiceLoop },
})
if (registry.status !== 0) {
  fail(
    [registry.stderr, registry.stdout].filter(Boolean).join('\n').trim()
      || 'Python registry check failed (is Python 3.10+ available?)',
  )
}

console.log('[voice-tts-ratchet] PASS')
