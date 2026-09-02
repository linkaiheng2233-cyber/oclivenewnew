import assert from 'node:assert/strict'
import { Buffer } from 'node:buffer'
import { spawn } from 'node:child_process'
import { randomBytes } from 'node:crypto'
import { readFileSync, rmSync } from 'node:fs'
import { createServer } from 'node:http'
import { dirname, join, resolve } from 'node:path'
// This script is executed directly by Node's built-in test runner.
// eslint-disable-next-line test/no-import-node-test
import test from 'node:test'
import { fileURLToPath } from 'node:url'

import { LIVE_CONFIRMATION } from './runtime-event-onebot-live-probe.mjs'
import {
  parsePrivateStoreProbeConfig,
  PRIVATE_RECONCILE_CONFIRMATION,
  PRIVATE_STORE_CONFIRMATION,
} from './runtime-event-onebot-private-store-probe.mjs'

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const scriptPath = join(repoRoot, 'scripts/runtime-event-onebot-private-store-probe.mjs')
const tokenSentinel = 'must-not-export-private-store-token'
const targetSentinel = '1000010000'
const providerIdSentinel = 51515151

function envFor(endpoint, outputDir, overrides = {}) {
  return {
    ...process.env,
    OCLIVE_ONEBOT_ACCESS_TOKEN: tokenSentinel,
    OCLIVE_ONEBOT_BASE_URL: endpoint,
    OCLIVE_ONEBOT_IMPLEMENTATION_LABEL: 'onebot-private-store-test-double-1.0.0',
    OCLIVE_ONEBOT_PRIVATE_STORE_KEY: randomBytes(32).toString('hex'),
    OCLIVE_ONEBOT_PRIVATE_STORE_KEY_ID: 'synthetic-key-1',
    OCLIVE_ONEBOT_PROBE_SYNTHETIC: '1',
    OCLIVE_ONEBOT_TEST_USER_ID: targetSentinel,
    OCLIVE_ONEBOT_TIMEOUT_MS: '3000',
    TEST_OUTPUT_DIR: outputDir,
    ...overrides,
  }
}

function prepareArgs(recordId) {
  return [
    'prepare',
    '--confirm-live',
    LIVE_CONFIRMATION,
    '--confirm-private-store',
    PRIVATE_STORE_CONFIRMATION,
    '--record-id',
    recordId,
  ]
}

function reconcileArgs(recordId) {
  return [
    'reconcile',
    '--confirm-live',
    LIVE_CONFIRMATION,
    '--confirm-reconcile',
    PRIVATE_RECONCILE_CONFIRMATION,
    '--record-id',
    recordId,
  ]
}

function withOutputDir(args, outputDir) {
  return [...args, '--output-dir', outputDir]
}

async function readJsonBody(request) {
  const chunks = []
  for await (const chunk of request)
    chunks.push(chunk)
  return JSON.parse(Buffer.concat(chunks).toString('utf8'))
}

async function startOneBotStub(handler) {
  const requests = []
  const server = createServer(async (request, response) => {
    const body = await readJsonBody(request)
    requests.push({ body, path: request.url })
    const result = handler(request.url, body)
    response.writeHead(result.status ?? 200, {
      'Content-Type': 'application/json',
    })
    response.end(typeof result.body === 'string'
      ? result.body
      : JSON.stringify(result.body))
  })
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve))
  const address = server.address()
  return {
    endpoint: `http://127.0.0.1:${address.port}`,
    requests,
    stop: () => new Promise((resolve, reject) => server.close(error => (
      error ? reject(error) : resolve()
    ))),
  }
}

function responseData(data) {
  return { body: { status: 'ok', retcode: 0, data } }
}

function runChild(args, env) {
  return new Promise((resolvePromise, reject) => {
    const child = spawn(process.execPath, [scriptPath, ...args], {
      cwd: repoRoot,
      env,
      stdio: ['ignore', 'pipe', 'pipe'],
      windowsHide: true,
    })
    const stdout = []
    const stderr = []
    child.stdout.on('data', chunk => stdout.push(chunk))
    child.stderr.on('data', chunk => stderr.push(chunk))
    child.on('error', reject)
    child.on('close', code => resolvePromise({
      code,
      stderr: Buffer.concat(stderr).toString('utf8'),
      stdout: Buffer.concat(stdout).toString('utf8'),
    }))
  })
}

test('private store probe fails closed on mode, confirmation, key, and remote scope', () => {
  const outputDir = 'target/oclive-event/onebot-private-store-probe-self-test/parse'
  const env = envFor('http://127.0.0.1:5700', outputDir)
  assert.throws(
    () => parsePrivateStoreProbeConfig([], env),
    /ONEBOT_PRIVATE_STORE_MODE_REQUIRED/,
  )
  assert.throws(
    () => parsePrivateStoreProbeConfig(['prepare'], env),
    /ONEBOT_LIVE_CONFIRMATION_REQUIRED/,
  )
  assert.throws(
    () => parsePrivateStoreProbeConfig(prepareArgs('record-1'), {
      ...env,
      OCLIVE_ONEBOT_PRIVATE_STORE_KEY: 'short',
    }),
    /OCLIVE_ONEBOT_PRIVATE_STORE_KEY_INVALID/,
  )
  assert.throws(
    () => parsePrivateStoreProbeConfig(
      [...prepareArgs('record-1'), '--allow-remote'],
      envFor('https://example.com', outputDir),
    ),
    /ONEBOT_PRIVATE_STORE_PROBE_LOOPBACK_ONLY/,
  )
})

test('separate prepare and reconcile processes keep locator encrypted and remove it', async (t) => {
  const stub = await startOneBotStub((path) => {
    if (path === '/get_version_info') {
      return responseData({
        app_name: 'onebot-private-store-test-double',
        app_version: '1.0.0',
        protocol_version: 'v11',
      })
    }
    if (path === '/send_private_msg')
      return responseData({ message_id: providerIdSentinel })
    if (path === '/delete_msg')
      return responseData(null)
    return { status: 404, body: '' }
  })
  t.after(stub.stop)

  const outputDir = resolve(
    'target/oclive-event/onebot-private-store-probe-self-test/cross-process',
  )
  rmSync(outputDir, { recursive: true, force: true })
  t.after(() => rmSync(outputDir, { recursive: true, force: true }))
  const recordId = 'synthetic-cross-process'
  const env = envFor(stub.endpoint, outputDir)

  const prepared = await runChild(
    withOutputDir(prepareArgs(recordId), outputDir),
    env,
  )
  assert.equal(prepared.code, 0, prepared.stderr)
  const storePath = join(outputDir, 'private-store', `${recordId}.json`)
  const encrypted = readFileSync(storePath, 'utf8')
  const encryptedJson = JSON.parse(encrypted)
  assert.equal(encryptedJson.state, 'delivered_pending_reconciliation')
  assert.equal(encryptedJson.cipher.algorithm, 'aes-256-gcm')
  assert.equal(typeof encryptedJson.cipher.ciphertext_base64url, 'string')
  for (const forbidden of [
    tokenSentinel,
    targetSentinel,
    String(providerIdSentinel),
    `${messagePrefixForTest()} ${recordId}`,
  ]) {
    assert.equal(encrypted.includes(forbidden), false)
  }

  const reconciled = await runChild(
    withOutputDir(reconcileArgs(recordId), outputDir),
    env,
  )
  assert.equal(reconciled.code, 0, reconciled.stderr)
  const tombstone = JSON.parse(readFileSync(storePath, 'utf8'))
  assert.equal(tombstone.state, 'recalled_locator_removed')
  assert.equal(tombstone.cipher, null)
  assert.notEqual(
    tombstone.audit.prepare_instance_id,
    tombstone.audit.reconcile_instance_id,
  )

  const evidencePath = join(
    outputDir,
    recordId,
    'onebot-private-store-probe.evidence.json',
  )
  const evidenceText = readFileSync(evidencePath, 'utf8')
  const evidence = JSON.parse(evidenceText)
  assert.equal(evidence.synthetic, true)
  assert.equal(evidence.probe_success, true)
  assert.equal(evidence.private_store.algorithm, 'aes-256-gcm')
  assert.equal(evidence.private_store.key_persisted_in_store, false)
  assert.equal(
    evidence.private_store.sensitive_plaintext_absent_before_reconciliation,
    true,
  )
  assert.equal(evidence.process_boundary.distinct_invocations, true)
  assert.equal(evidence.reconciliation.attempts, 1)
  assert.equal(evidence.reconciliation.outcome, 'acknowledged')
  assert.deepEqual(evidence.remaining_gaps, {
    encrypted_probe_private_store_tested: false,
    restart_reconciliation_tested: false,
    provider_accept_to_locator_persist_crash_window_closed: false,
    multi_host_owner_lease_tested: false,
    production_stream_connected: false,
  })
  for (const forbidden of [
    env.OCLIVE_ONEBOT_PRIVATE_STORE_KEY,
    tokenSentinel,
    targetSentinel,
    String(providerIdSentinel),
    stub.endpoint,
  ]) {
    assert.equal(evidenceText.includes(forbidden), false)
  }
  assert.deepEqual(stub.requests.map(request => request.path), [
    '/get_version_info',
    '/send_private_msg',
    '/get_version_info',
    '/delete_msg',
  ])
  assert.deepEqual(stub.requests[3].body, {
    message_id: providerIdSentinel,
  })
})

test('wrong key refuses reconciliation before any provider recall', async (t) => {
  const stub = await startOneBotStub((path) => {
    if (path === '/get_version_info') {
      return responseData({
        app_name: 'onebot-private-store-test-double',
        app_version: '1.0.0',
        protocol_version: 'v11',
      })
    }
    if (path === '/send_private_msg')
      return responseData({ message_id: providerIdSentinel })
    return responseData(null)
  })
  t.after(stub.stop)

  const outputDir = resolve(
    'target/oclive-event/onebot-private-store-probe-self-test/wrong-key',
  )
  rmSync(outputDir, { recursive: true, force: true })
  t.after(() => rmSync(outputDir, { recursive: true, force: true }))
  const recordId = 'synthetic-wrong-key'
  const env = envFor(stub.endpoint, outputDir)
  const prepared = await runChild(
    withOutputDir(prepareArgs(recordId), outputDir),
    env,
  )
  assert.equal(prepared.code, 0, prepared.stderr)

  const rejected = await runChild(
    withOutputDir(reconcileArgs(recordId), outputDir),
    {
      ...env,
      OCLIVE_ONEBOT_PRIVATE_STORE_KEY: randomBytes(32).toString('hex'),
    },
  )
  assert.equal(rejected.code, 1)
  assert.match(rejected.stderr, /ONEBOT_PRIVATE_STORE_DECRYPT_FAILED/)
  assert.deepEqual(stub.requests.map(request => request.path), [
    '/get_version_info',
    '/send_private_msg',
  ])
})

test('uncertain recall retains the encrypted locator without automatic retry', async (t) => {
  const stub = await startOneBotStub((path) => {
    if (path === '/get_version_info') {
      return responseData({
        app_name: 'onebot-private-store-test-double',
        app_version: '1.0.0',
        protocol_version: 'v11',
      })
    }
    if (path === '/send_private_msg')
      return responseData({ message_id: providerIdSentinel })
    return { status: 500, body: 'not-json' }
  })
  t.after(stub.stop)

  const outputDir = resolve(
    'target/oclive-event/onebot-private-store-probe-self-test/recall-uncertain',
  )
  rmSync(outputDir, { recursive: true, force: true })
  t.after(() => rmSync(outputDir, { recursive: true, force: true }))
  const recordId = 'synthetic-recall-uncertain'
  const env = envFor(stub.endpoint, outputDir)
  const prepared = await runChild(
    withOutputDir(prepareArgs(recordId), outputDir),
    env,
  )
  assert.equal(prepared.code, 0, prepared.stderr)

  const reconciled = await runChild(
    withOutputDir(reconcileArgs(recordId), outputDir),
    env,
  )
  assert.equal(reconciled.code, 1)
  const store = JSON.parse(readFileSync(
    join(outputDir, 'private-store', `${recordId}.json`),
    'utf8',
  ))
  assert.equal(store.state, 'reconciliation_failed_locator_retained')
  assert.equal(store.cipher.algorithm, 'aes-256-gcm')
  assert.equal(store.audit.reconciliation_attempts, 1)
  assert.equal(store.audit.reconciliation_outcome, 'delivery_uncertain')
  assert.deepEqual(stub.requests.map(request => request.path), [
    '/get_version_info',
    '/send_private_msg',
    '/get_version_info',
    '/delete_msg',
  ])
})

function messagePrefixForTest() {
  return 'OCLive A.2.2.2 private store probe'
}
