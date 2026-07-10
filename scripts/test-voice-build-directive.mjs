#!/usr/bin/env node
/**
 * Smoke-test voice.build_directive persona derivation for four roles.
 * Spawns rpc_server.mjs unless OCLIVE_VOICE_RPC_URL is set.
 */
import { spawn } from 'child_process'
import fs from 'fs'
import path from 'path'
import { fileURLToPath } from 'url'

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const rpcScript = path.join(
  repoRoot,
  'distros/chat-pro/plugins/com.oclive.voice.asr/rpc_server.mjs',
)

const ROLES = [
  { id: 'mumu', dir: 'distros/chat-pro/roles/mumu', expectHandwritten: true },
  { id: 'shimeng', dir: 'distros/chat-pro/roles/shimeng', expectSharp: true },
  { id: '枫侵月', dir: 'distros/chat-pro/roles/枫侵月', expectMasculine: true },
  { id: 'polish-dev', dir: 'distros/chat-pro/roles/polish-dev', expectGeneric: true },
]

const EMOTIONS = ['neutral', 'happy', 'shy']

async function waitForRpcUrl(child, timeoutMs = 15_000) {
  return new Promise((resolve, reject) => {
    let buf = ''
    const timer = setTimeout(() => {
      reject(new Error('rpc_server startup timeout'))
    }, timeoutMs)
    child.stdout.on('data', (chunk) => {
      buf += chunk.toString()
      const m = buf.match(/OCLIVE_READY (http:\/\/[^\s]+)/)
      if (m) {
        clearTimeout(timer)
        resolve(m[1])
      }
    })
    child.stderr.on('data', (chunk) => {
      buf += chunk.toString()
    })
    child.on('exit', (code) => {
      if (!buf.includes('OCLIVE_READY')) {
        clearTimeout(timer)
        reject(new Error(`rpc_server exited ${code}: ${buf.slice(-500)}`))
      }
    })
  })
}

async function rpcCall(base, method, params) {
  const res = await fetch(`${base}/rpc`, {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      'x-oclive-remote-protocol': 'oclive-remote-jsonrpc-v1',
    },
    body: JSON.stringify({ jsonrpc: '2.0', id: 1, method, params }),
    signal: AbortSignal.timeout(30_000),
  })
  const body = await res.json()
  return body?.result ?? body
}

function assert(cond, msg) {
  if (!cond) throw new Error(msg)
}

async function main() {
  let rpcUrl = process.env.OCLIVE_VOICE_RPC_URL?.trim()
  let child = null
  if (!rpcUrl) {
    child = spawn(process.execPath, [rpcScript], {
      cwd: repoRoot,
      env: process.env,
      stdio: ['ignore', 'pipe', 'pipe'],
    })
    rpcUrl = await waitForRpcUrl(child)
  }

  let failed = 0
  try {
    for (const role of ROLES) {
      const rolePath = path.join(repoRoot, role.dir)
      assert(fs.existsSync(rolePath), `missing role dir ${rolePath}`)
      for (const botEmotion of EMOTIONS) {
        const result = await rpcCall(rpcUrl, 'voice.build_directive', {
          role_path: rolePath.replace(/\\/g, '/'),
          bot_emotion: botEmotion,
        })
        if (!result?.ok) {
          console.error('FAIL', role.id, botEmotion, result)
          failed += 1
          continue
        }
        const emo = result.directive?.emo_text || ''
        console.log('OK', role.id, botEmotion, emo.slice(0, 80))
        if (role.expectHandwritten && botEmotion === 'neutral') {
          const vp = JSON.parse(
            fs.readFileSync(path.join(rolePath, 'voice_profile.json'), 'utf8'),
          )
          assert(
            emo.includes(vp.emo_text_template?.replace('{tone}', '').slice(0, 6) || '软萌'),
            `${role.id}: expected handwritten voice_profile emo_text`,
          )
        }
        if (role.expectSharp && botEmotion === 'neutral') {
          assert(/清冷|冷淡|锋利|毒舌/.test(emo), `${role.id}: expected sharp/cool tone in emo_text`)
        }
        if (role.expectMasculine && botEmotion === 'neutral') {
          assert(/少年|男声|温和/.test(emo), `${role.id}: expected masculine/gentle tone`)
        }
        if (role.expectGeneric && botEmotion === 'neutral') {
          assert(emo.length > 4, `${role.id}: expected generic emo_text`)
        }
      }
    }
  } finally {
    if (child) {
      child.kill('SIGTERM')
      await new Promise(r => setTimeout(r, 300))
    }
  }

  if (failed > 0) {
    console.error(`build_directive matrix: ${failed} failures`)
    process.exit(1)
  }
  console.log('build_directive matrix: PASS (4 roles × 3 emotions)')
}

main().catch((err) => {
  console.error(err)
  process.exit(1)
})
