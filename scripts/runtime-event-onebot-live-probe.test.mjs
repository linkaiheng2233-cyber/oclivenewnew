import assert from 'node:assert/strict'
import { existsSync, readFileSync, rmSync } from 'node:fs'
import { createServer } from 'node:http'
import { join, resolve } from 'node:path'
import test from 'node:test'

import {
  LIVE_CONFIRMATION,
  executeLiveProbe,
  parseProbeConfig,
  runLiveProbe,
} from './runtime-event-onebot-live-probe.mjs'

const tokenSentinel = 'must-not-export-access-token'
const targetSentinel = '1000010000'
const source = { commit: 'a'.repeat(40), worktreeDirty: false }

function envFor(endpoint, overrides = {}) {
  return {
    OCLIVE_ONEBOT_ACCESS_TOKEN: tokenSentinel,
    OCLIVE_ONEBOT_BASE_URL: endpoint,
    OCLIVE_ONEBOT_IMPLEMENTATION_LABEL: 'onebot-test-double-1.0.0',
    OCLIVE_ONEBOT_TEST_USER_ID: targetSentinel,
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
      contentType: request.headers['content-type'],
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

function configFor(endpoint) {
  return parseProbeConfig(
    ['--confirm-live', LIVE_CONFIRMATION],
    envFor(endpoint),
  )
}

function responseData(data) {
  return { body: { status: 'ok', retcode: 0, data } }
}

test('live probe fails closed without exact confirmation and safe endpoint rules', () => {
  assert.throws(
    () => parseProbeConfig([], envFor('http://127.0.0.1:5700')),
    /ONEBOT_LIVE_CONFIRMATION_REQUIRED/,
  )
  assert.throws(
    () => parseProbeConfig(
      ['--confirm-live', LIVE_CONFIRMATION, '--allow-remote'],
      envFor('http://example.com'),
    ),
    /ONEBOT_REMOTE_ENDPOINT_REQUIRES_TLS/,
  )
  assert.throws(
    () => parseProbeConfig(
      ['--confirm-live', LIVE_CONFIRMATION],
      envFor('http://127.0.0.1:5700', {
        OCLIVE_ONEBOT_TEST_GROUP_ID: '2000020000',
      }),
    ),
    /EXACTLY_ONE_ONEBOT_TEST_TARGET_REQUIRED/,
  )
  assert.throws(
    () => parseProbeConfig(
      ['--confirm-live', LIVE_CONFIRMATION],
      envFor('http://127.0.0.1:5700', {
        OCLIVE_ONEBOT_IMPLEMENTATION_LABEL: '../private',
      }),
    ),
    /OCLIVE_ONEBOT_IMPLEMENTATION_LABEL_INVALID/,
  )
})

test('synthetic acknowledged send is recalled and evidence stays private', async (t) => {
  const stub = await startOneBotStub((path) => {
    if (path === '/get_version_info') {
      return responseData({
        app_name: 'onebot-test-double',
        app_version: '1.0.0',
        protocol_version: 'v11',
      })
    }
    if (path === '/send_private_msg')
      return responseData({ message_id: 4242 })
    if (path === '/delete_msg')
      return responseData(null)
    return { status: 404, body: '' }
  })
  t.after(stub.stop)

  const result = await runLiveProbe(configFor(stub.endpoint), {
    now: () => new Date('2026-09-02T00:00:00.000Z'),
    runId: 'synthetic-success',
    source,
    synthetic: true,
  })

  assert.equal(result.success, true)
  assert.equal(result.evidence.live_adapter_tested, false)
  assert.equal(result.evidence.production_runtime_enabled, false)
  assert.equal(result.evidence.production_ready, false)
  assert.equal(result.evidence.implementation_label, 'onebot-test-double-1.0.0')
  assert.equal(result.evidence.preflight.outcome, 'accepted')
  assert.equal(result.evidence.send.outcome, 'delivered')
  assert.equal(result.evidence.recall.outcome, 'acknowledged')
  assert.equal(result.evidence.send.automatic_retries, 0)
  assert.equal(stub.requests.length, 3)
  assert.deepEqual(stub.requests.map(request => request.path), [
    '/get_version_info',
    '/send_private_msg',
    '/delete_msg',
  ])
  for (const request of stub.requests) {
    assert.equal(request.method, 'POST')
    assert.equal(request.authorization, `Bearer ${tokenSentinel}`)
    assert.equal(request.contentType, 'application/json')
  }
  assert.equal(stub.requests[1].body.user_id, Number(targetSentinel))
  assert.equal(stub.requests[1].body.auto_escape, true)
  assert.match(stub.requests[1].body.message, /^OCLive A\.2\.2\.2 delivery probe /)
  assert.deepEqual(stub.requests[2].body, { message_id: 4242 })

  const serialized = JSON.stringify(result.evidence)
  assert.equal(serialized.includes(tokenSentinel), false)
  assert.equal(serialized.includes(targetSentinel), false)
  assert.equal(serialized.includes(stub.endpoint), false)
  assert.equal(serialized.includes(stub.requests[1].body.message), false)
  assert.equal(serialized.includes('4242'), false)
  assert.deepEqual(result.evidence.remaining_gaps, {
    adapter_private_store_tested: false,
    timeout_after_possible_submission_tested: false,
    multi_host_owner_lease_tested: false,
    production_stream_connected: false,
  })
})

test('probe run id cannot escape the evidence directory', async () => {
  await assert.rejects(
    runLiveProbe(configFor('http://127.0.0.1:5700'), {
      runId: '../escape',
      source,
      synthetic: true,
    }),
    /ONEBOT_PROBE_RUN_ID_INVALID/,
  )
})

test('collector writes only sanitized evidence and releases its local lock', async (t) => {
  const stub = await startOneBotStub((path) => {
    if (path === '/get_version_info') {
      return responseData({
        app_name: 'onebot-test-double',
        app_version: '1.0.0',
        protocol_version: 'v11',
      })
    }
    if (path === '/send_private_msg')
      return responseData({ message_id: 6553 })
    return responseData(null)
  })
  t.after(stub.stop)

  const outputDir = resolve(
    'target/oclive-event/onebot-live-probe-self-test/sanitized-evidence',
  )
  rmSync(outputDir, { recursive: true, force: true })
  t.after(() => rmSync(outputDir, { recursive: true, force: true }))
  const config = { ...configFor(stub.endpoint), outputDir }
  const result = await executeLiveProbe(config, {
    now: () => new Date('2026-09-02T00:00:00.000Z'),
    runId: 'synthetic-write',
    source,
    synthetic: true,
  })

  assert.equal(result.success, true)
  assert.equal(existsSync(join(outputDir, '.probe.lock')), false)
  const serialized = readFileSync(result.evidencePath, 'utf8')
  assert.deepEqual(JSON.parse(serialized), result.evidence)
  for (const forbidden of [
    tokenSentinel,
    targetSentinel,
    stub.endpoint,
    stub.requests[1].body.message,
    '6553',
  ]) {
    assert.equal(serialized.includes(forbidden), false)
  }
})

test('missing message id becomes uncertain without retry or recall', async (t) => {
  const stub = await startOneBotStub((path) => {
    if (path === '/get_version_info') {
      return responseData({
        app_name: 'onebot-test-double',
        app_version: '1.0.0',
        protocol_version: 'v11',
      })
    }
    return responseData({})
  })
  t.after(stub.stop)

  const result = await runLiveProbe(configFor(stub.endpoint), {
    now: () => new Date('2026-09-02T00:00:00.000Z'),
    runId: 'synthetic-uncertain',
    source,
    synthetic: true,
  })

  assert.equal(result.success, false)
  assert.equal(result.evidence.send.outcome, 'delivery_uncertain')
  assert.equal(result.evidence.send.automatic_retries, 0)
  assert.equal(result.evidence.recall.outcome, 'not_attempted')
  assert.deepEqual(stub.requests.map(request => request.path), [
    '/get_version_info',
    '/send_private_msg',
  ])
})

test('failed recall is reported without exposing its provider locator', async (t) => {
  const stub = await startOneBotStub((path) => {
    if (path === '/get_version_info') {
      return responseData({
        app_name: 'onebot-test-double',
        app_version: '1.0.0',
        protocol_version: 'v11',
      })
    }
    if (path === '/send_private_msg')
      return responseData({ message_id: 7331 })
    return { status: 500, body: 'not-json' }
  })
  t.after(stub.stop)

  const result = await runLiveProbe(configFor(stub.endpoint), {
    now: () => new Date('2026-09-02T00:00:00.000Z'),
    runId: 'synthetic-recall-failure',
    source,
    synthetic: true,
  })

  assert.equal(result.success, false)
  assert.equal(result.evidence.send.outcome, 'delivered')
  assert.equal(result.evidence.recall.outcome, 'delivery_uncertain')
  assert.equal(JSON.stringify(result.evidence).includes('7331'), false)
  assert.equal(stub.requests.length, 3)
})
