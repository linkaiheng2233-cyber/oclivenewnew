import assert from 'node:assert/strict'
import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import test from 'node:test'
import {
  findWorkspaceDevLlamaServer,
  resolveChatProDevRuntimeEnv,
} from './dev-performance-runtime.mjs'

const executable = process.platform === 'win32' ? 'llama-server.exe' : 'llama-server'

test('finds the newest unpacked sibling llama.cpp runtime', (t) => {
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(), 'oclive-dev-runtime-'))
  t.after(() => fs.rmSync(workspace, { recursive: true, force: true }))
  const repo = path.join(workspace, 'repo')
  const older = path.join(workspace, 'components', 'llama.cpp', 'b100', executable)
  const newer = path.join(workspace, 'components', 'llama.cpp', 'b200', executable)
  fs.mkdirSync(path.dirname(older), { recursive: true })
  fs.mkdirSync(path.dirname(newer), { recursive: true })
  fs.writeFileSync(older, '')
  fs.writeFileSync(newer, '')

  assert.equal(findWorkspaceDevLlamaServer(repo), newer)
})

test('explicit runtime path wins without mutating the source environment', () => {
  const sourceEnv = { OCLIVE_LLAMA_SERVER_PATH: 'X:/explicit/llama-server.exe' }
  const resolved = resolveChatProDevRuntimeEnv('X:/workspace/repo', sourceEnv)

  assert.equal(resolved.env.OCLIVE_LLAMA_SERVER_PATH, sourceEnv.OCLIVE_LLAMA_SERVER_PATH)
  assert.equal(resolved.inferredRuntimePath, null)
  assert.notEqual(resolved.env, sourceEnv)
})
