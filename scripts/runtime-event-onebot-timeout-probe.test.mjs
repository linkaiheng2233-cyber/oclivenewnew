import assert from 'node:assert/strict'
import { Buffer } from 'node:buffer'
import { existsSync, readFileSync, rmSync } from 'node:fs'
import { createServer } from 'node:http'
import { join, resolve } from 'node:path'
// This script is executed directly by Node's built-in test runner.
// eslint-disable-next-line test/no-import-node-test
import test from 'node:test'

import { LIVE_CONFIRMATION } from './runtime-event-onebot-live-probe.mjs'
import {
  executeTimeoutProbe,
  parseTimeoutProbeConfig,
  RECONCILE_CONFIRMATION,
  runTimeoutProbe,
  TIMEOUT_CONFIRMATION,
} from './runtime-event-onebot-timeout-probe.mjs'

const tokenSentinel = 'must-not-export-timeout-token'
const targetSentinel = '1000010000'
const providerIdSentinel = 42424242
const source = { commit: 'b'.repeat(40), worktreeDirty: false }
const confirmations = [
  '--confirm-live',
  LIVE_CONFIRMATION,
  '--confirm-timeout',
  TIMEOUT_CONFIRMATION,
  '--confirm-reconcile',
  RECONCILE_CONFIRMATION,
]

function envFor(endpoint, overrides = {}) {
  return {
    OCLIVE_ONEBOT_ACCESS_TOKEN: tokenSentinel,
    OCLIVE_ONEBOT_BASE_URL: endpoint,
    OCLIVE_ONEBOT_FAULT_TIMEOUT_MS: '1000',
    OCLIVE_ONEBOT_IMPLEMENTATION_LABEL: 'onebot-timeout-test-double-1.0.0',
    OCLIVE_ONEBOT_TEST_USER_ID: targetSentinel,
    OCLIVE_ONEBOT_TIMEOUT_MS: '3000',
    ...overrides,
  }
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
    requests.push({
      authorization: request.headers.authorization,
      body,
      method: request.method,
      path: request.url,
    })
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

function configFor(endpoint, overrides = {}) {
  return {
    ...parseTimeoutProbeConfig(confirmations, envFor(endpoint)),
    clientTimeoutMs: 20,
    ...overrides,
  }
}

function syntheticDependencies(overrides = {}) {
  return {
    now: () => new Date('2026-09-02T00:00:00.000Z'),
    runId: 'synthetic-timeout',
    source,
    synthetic: true,
    ...overrides,
  }
}

test('timeout probe requires all confirmations and loopback before network', () => {
  assert.throws(
    () => parseTimeoutProbeConfig([], envFor('http://127.0.0.1:5700')),
    /ONEBOT_LIVE_CONFIRMATION_REQUIRED/,
  )
  assert.throws(
    () => parseTimeoutProbeConfig(
      confirmations.slice(0, 4),
      envFor('http://127.0.0.1:5700'),
    ),
    /ONEBOT_RECONCILIATION_CONFIRMATION_REQUIRED/,
  )
  assert.throws(
    () => parseTimeoutProbeConfig(
      [...confirmations, '--allow-remote'],
      envFor('https://example.com'),
    ),
    /ONEBOT_TIMEOUT_PROBE_LOOPBACK_ONLY/,
  )
  assert.throws(
    () => parseTimeoutProbeConfig(
      confirmations,
      envFor('http://127.0.0.1:5700', {
        OCLIVE_ONEBOT_TIMEOUT_MS: '1500',
      }),
    ),
    /OCLIVE_ONEBOT_TIMEOUT_MS_MUST_EXCEED_FAULT_TIMEOUT/,
  )
})

test('withheld real ACK becomes uncertain before one reconciliation recall', async (t) => {
  const stub = await startOneBotStub((path) => {
    if (path === '/get_version_info') {
      return responseData({
        app_name: 'onebot-timeout-test-double',
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

  const result = await runTimeoutProbe(
    configFor(stub.endpoint),
    syntheticDependencies(),
  )

  assert.equal(result.success, true)
  assert.equal(result.evidence.live_adapter_tested, false)
  assert.equal(result.evidence.production_runtime_enabled, false)
  assert.equal(result.evidence.production_ready, false)
  assert.equal(result.evidence.injected_fault.client_abort_observed, true)
  assert.equal(result.evidence.injected_fault.upstream_send_requests, 1)
  assert.equal(result.evidence.probe_observation.send_outcome, 'delivery_uncertain')
  assert.equal(result.evidence.probe_observation.send_http_status, null)
  assert.equal(result.evidence.probe_observation.automatic_retries, 0)
  assert.equal(result.evidence.probe_observation.probe_recall_attempts, 0)
  assert.equal(result.evidence.upstream_observation.send_outcome, 'delivered')
  assert.equal(result.evidence.reconciliation.started_only_after_probe_uncertain, true)
  assert.equal(result.evidence.reconciliation.attempts, 1)
  assert.equal(result.evidence.reconciliation.outcome, 'acknowledged')
  assert.deepEqual(stub.requests.map(request => request.path), [
    '/get_version_info',
    '/send_private_msg',
    '/delete_msg',
  ])
  assert.equal(stub.requests[1].body.user_id, Number(targetSentinel))
  assert.deepEqual(stub.requests[2].body, { message_id: providerIdSentinel })

  const serialized = JSON.stringify(result.evidence)
  for (const forbidden of [
    tokenSentinel,
    targetSentinel,
    stub.endpoint,
    String(providerIdSentinel),
    stub.requests[1].body.message,
  ]) {
    assert.equal(serialized.includes(forbidden), false)
  }
  assert.deepEqual(result.evidence.remaining_gaps, {
    adapter_private_store_tested: false,
    timeout_after_possible_submission_tested: false,
    explicit_reconciliation_tested: false,
    multi_host_owner_lease_tested: false,
    production_stream_connected: false,
  })
})

test('rejected upstream send never attempts reconciliation', async (t) => {
  const stub = await startOneBotStub((path) => {
    if (path === '/get_version_info') {
      return responseData({
        app_name: 'onebot-timeout-test-double',
        app_version: '1.0.0',
        protocol_version: 'v11',
      })
    }
    return { status: 403, body: { status: 'failed', retcode: 1403 } }
  })
  t.after(stub.stop)

  const result = await runTimeoutProbe(
    configFor(stub.endpoint),
    syntheticDependencies({ runId: 'synthetic-rejected' }),
  )

  assert.equal(result.success, false)
  assert.equal(result.evidence.probe_observation.send_outcome, 'delivery_uncertain')
  assert.equal(result.evidence.upstream_observation.send_outcome, 'rejected')
  assert.equal(result.evidence.reconciliation.attempts, 0)
  assert.equal(result.evidence.reconciliation.outcome, 'not_attempted')
  assert.deepEqual(stub.requests.map(request => request.path), [
    '/get_version_info',
    '/send_private_msg',
  ])
})

test('failed reconciliation remains visible without exporting locator', async (t) => {
  const stub = await startOneBotStub((path) => {
    if (path === '/get_version_info') {
      return responseData({
        app_name: 'onebot-timeout-test-double',
        app_version: '1.0.0',
        protocol_version: 'v11',
      })
    }
    if (path === '/send_private_msg')
      return responseData({ message_id: providerIdSentinel })
    return { status: 500, body: 'not-json' }
  })
  t.after(stub.stop)

  const result = await runTimeoutProbe(
    configFor(stub.endpoint),
    syntheticDependencies({ runId: 'synthetic-reconcile-failed' }),
  )

  assert.equal(result.success, false)
  assert.equal(result.evidence.reconciliation.attempts, 1)
  assert.equal(result.evidence.reconciliation.outcome, 'delivery_uncertain')
  assert.equal(JSON.stringify(result.evidence).includes(String(providerIdSentinel)), false)
})

test('collector writes sanitized evidence and releases its lock', async (t) => {
  const stub = await startOneBotStub((path) => {
    if (path === '/get_version_info') {
      return responseData({
        app_name: 'onebot-timeout-test-double',
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
    'target/oclive-event/onebot-timeout-probe-self-test/sanitized-evidence',
  )
  rmSync(outputDir, { recursive: true, force: true })
  t.after(() => rmSync(outputDir, { recursive: true, force: true }))
  const result = await executeTimeoutProbe(
    configFor(stub.endpoint, { outputDir }),
    syntheticDependencies({ runId: 'synthetic-write' }),
  )

  assert.equal(result.success, true)
  assert.equal(existsSync(join(outputDir, '.probe.lock')), false)
  const serialized = readFileSync(result.evidencePath, 'utf8')
  assert.deepEqual(JSON.parse(serialized), result.evidence)
  for (const forbidden of [
    tokenSentinel,
    targetSentinel,
    stub.endpoint,
    String(providerIdSentinel),
    stub.requests[1].body.message,
  ]) {
    assert.equal(serialized.includes(forbidden), false)
  }
})
