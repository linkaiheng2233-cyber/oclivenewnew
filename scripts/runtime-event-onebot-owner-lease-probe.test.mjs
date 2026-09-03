import assert from 'node:assert/strict'
import { Buffer } from 'node:buffer'
import { spawn } from 'node:child_process'
import { randomUUID } from 'node:crypto'
import {
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from 'node:fs'
import { createServer } from 'node:http'
import { dirname, join, resolve } from 'node:path'
// This script is executed directly by Node's built-in test runner.
// eslint-disable-next-line test/no-import-node-test
import test from 'node:test'
import { fileURLToPath } from 'node:url'

import {
  beginOwnerAttempt,
  claimOwnerLease,
  finishOwnerAttempt,
  openOwnerLeaseProbeStore,
  OWNER_LEASE_ATTEMPT_EXIT_CODE,
  OWNER_LEASE_CONFIRMATION,
  ownerLeaseDatabasePath,
  ownerLeaseEvidence,
  parseOwnerLeaseProbeConfig,
  readOwnerLease,
} from './runtime-event-onebot-owner-lease-probe.mjs'

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const scriptPath = join(
  repoRoot,
  'scripts/runtime-event-onebot-owner-lease-probe.mjs',
)
const tokenSentinel = 'must-not-export-owner-lease-token'
const targetSentinel = '1000030000'
const providerIdSentinel = 61616161

function outputDir(label) {
  const path = join(
    repoRoot,
    'target/oclive-event/onebot-owner-lease-probe-self-test',
    `${label}-${randomUUID()}`,
  )
  mkdirSync(path, { recursive: true })
  return path
}

function envFor(endpoint, directory, overrides = {}) {
  return {
    ...process.env,
    OCLIVE_ONEBOT_ACCESS_TOKEN: tokenSentinel,
    OCLIVE_ONEBOT_BASE_URL: endpoint,
    OCLIVE_ONEBOT_IMPLEMENTATION_LABEL: 'onebot-owner-lease-test-double-1.0.0',
    OCLIVE_ONEBOT_OWNER_LEASE_PAUSE_MS: '0',
    OCLIVE_ONEBOT_OWNER_LEASE_TTL_MS: '2000',
    OCLIVE_ONEBOT_PROBE_SYNTHETIC: '1',
    OCLIVE_ONEBOT_TEST_GROUP_ID: targetSentinel,
    OCLIVE_ONEBOT_TIMEOUT_MS: '3000',
    TEST_OUTPUT_DIR: directory,
    ...overrides,
  }
}

function ownerArgs(mode, recordId, ownerId) {
  return [
    mode,
    '--confirm-owner-lease',
    OWNER_LEASE_CONFIRMATION,
    '--record-id',
    recordId,
    '--owner-id',
    ownerId,
  ]
}

function withOutputDir(args, directory) {
  return [...args, '--output-dir', directory]
}

async function readJsonBody(request) {
  const chunks = []
  for await (const chunk of request)
    chunks.push(chunk)
  return JSON.parse(Buffer.concat(chunks).toString('utf8'))
}

async function startOneBotStub() {
  const requests = []
  const server = createServer(async (request, response) => {
    const body = await readJsonBody(request)
    requests.push({ body, path: request.url })
    const data = request.url === '/get_version_info'
      ? { protocol_version: 'v11' }
      : request.url === '/send_group_msg'
        ? { message_id: providerIdSentinel }
        : null
    const status = data === null ? 'failed' : 'ok'
    response.writeHead(200, { 'Content-Type': 'application/json' })
    response.end(JSON.stringify({
      status,
      retcode: data === null ? 100 : 0,
      data,
    }))
  })
  await new Promise(resolvePromise => server.listen(0, '127.0.0.1', resolvePromise))
  const address = server.address()
  return {
    endpoint: `http://127.0.0.1:${address.port}`,
    requests,
    stop: () => new Promise((resolvePromise, reject) => server.close(error => (
      error ? reject(error) : resolvePromise()
    ))),
  }
}

function spawnProbe(args, env) {
  const child = spawn(process.execPath, [
    '--disable-warning=ExperimentalWarning',
    scriptPath,
    ...args,
  ], {
    cwd: repoRoot,
    env,
    stdio: ['ignore', 'pipe', 'pipe'],
    windowsHide: true,
  })
  const stdout = []
  const stderr = []
  let lineBuffer = ''
  let claimSignal = null
  let resolveClaim
  const claimed = new Promise((resolvePromise) => {
    resolveClaim = resolvePromise
  })
  child.stdout.on('data', (chunk) => {
    stdout.push(chunk)
    lineBuffer += chunk.toString('utf8')
    const lines = lineBuffer.split(/\r?\n/)
    lineBuffer = lines.pop() ?? ''
    for (const line of lines) {
      try {
        const value = JSON.parse(line)
        if (value.event === 'lease_claimed' && claimSignal === null) {
          claimSignal = value
          resolveClaim(value)
        }
      }
      catch {
        // Non-JSON output cannot become a synchronization signal.
      }
    }
  })
  child.stderr.on('data', chunk => stderr.push(chunk))
  const done = new Promise((resolvePromise, reject) => {
    child.on('error', reject)
    child.on('close', (code) => {
      if (claimSignal === null)
        resolveClaim(null)
      resolvePromise({
        code,
        stderr: Buffer.concat(stderr).toString('utf8'),
        stdout: Buffer.concat(stdout).toString('utf8'),
      })
    })
  })
  return { claimed, done }
}

async function runProbe(args, env) {
  return spawnProbe(args, env).done
}

function assertNoSensitiveOutput(result) {
  const output = `${result.stdout}\n${result.stderr}`
  assert.doesNotMatch(output, new RegExp(tokenSentinel, 'u'))
  assert.doesNotMatch(output, new RegExp(targetSentinel, 'u'))
  assert.doesNotMatch(output, new RegExp(String(providerIdSentinel), 'u'))
  assert.doesNotMatch(output, /owner lease probe [a-z0-9.-]+/iu)
}

function sendRequests(stub) {
  return stub.requests.filter(request => request.path === '/send_group_msg')
}

test('owner lease probe is synthetic-only, explicitly confirmed, and loopback-only', () => {
  const directory = outputDir('parse')
  const env = envFor('http://127.0.0.1:5700', directory)
  try {
    assert.throws(
      () => parseOwnerLeaseProbeConfig([], env),
      /ONEBOT_OWNER_LEASE_MODE_REQUIRED/,
    )
    assert.throws(
      () => parseOwnerLeaseProbeConfig(
        ownerArgs('claim-only', 'record-1', 'owner-a').slice(0, 1),
        env,
      ),
      /ONEBOT_OWNER_LEASE_CONFIRMATION_REQUIRED/,
    )
    assert.throws(
      () => parseOwnerLeaseProbeConfig(
        ownerArgs('claim-only', 'record-1', 'owner-a'),
        { ...env, OCLIVE_ONEBOT_PROBE_SYNTHETIC: '0' },
      ),
      /ONEBOT_OWNER_LEASE_SYNTHETIC_ONLY/,
    )
    assert.throws(
      () => parseOwnerLeaseProbeConfig(
        [...ownerArgs('claim-only', 'record-1', 'owner-a'), '--allow-remote'],
        envFor('https://example.com', directory),
      ),
      /ONEBOT_OWNER_LEASE_PROBE_LOOPBACK_ONLY/,
    )
    assert.throws(
      () => parseOwnerLeaseProbeConfig(
        ownerArgs('claim-only', 'record-1', 'owner-a'),
        { ...env, OCLIVE_ONEBOT_OWNER_LEASE_TTL_MS: '99' },
      ),
      /OCLIVE_ONEBOT_OWNER_LEASE_TTL_MS_INVALID/,
    )
  }
  finally {
    rmSync(directory, { force: true, recursive: true })
  }
})

test('single-host owner lease scenarios stay fenced and emit sanitized evidence', async (t) => {
  await t.test('revision and epoch reject stale owners before effect and completion', () => {
    const directory = outputDir('state-machine')
    const database = openOwnerLeaseProbeStore(directory)
    try {
      const first = claimOwnerLease(database, {
        leaseTtlMs: 100,
        nowMs: 1000,
        ownerId: 'owner-a',
        recordId: 'state-machine',
      })
      assert.equal(first.acquired, true)
      assert.equal(first.token.leaseEpoch, 1)
      const active = claimOwnerLease(database, {
        leaseTtlMs: 100,
        nowMs: 1050,
        ownerId: 'owner-b',
        recordId: 'state-machine',
      })
      assert.deepEqual(active, {
        acquired: false,
        outcome: 'lease_active',
        token: null,
      })
      const takeover = claimOwnerLease(database, {
        leaseTtlMs: 100,
        nowMs: 1100,
        ownerId: 'owner-b',
        recordId: 'state-machine',
      })
      assert.equal(takeover.acquired, true)
      assert.equal(takeover.token.leaseEpoch, 2)
      assert.equal(beginOwnerAttempt(database, first.token, 1101).accepted, false)
      const attempt = beginOwnerAttempt(database, takeover.token, 1101)
      assert.equal(attempt.accepted, true)
      assert.equal(
        finishOwnerAttempt(database, first.token, 'delivered', 1102).accepted,
        false,
      )
      assert.equal(
        finishOwnerAttempt(database, attempt.token, 'delivered', 1102).accepted,
        true,
      )
      assert.equal(readOwnerLease(database, 'state-machine').delivery_state, 'delivered')
    }
    finally {
      database.close()
      rmSync(directory, { force: true, recursive: true })
    }
  })

  await t.test('two processes contend but only the lease owner sends', async () => {
    const directory = outputDir('concurrent')
    const stub = await startOneBotStub()
    try {
      const first = spawnProbe(
        withOutputDir(ownerArgs('claim-send', 'concurrent', 'owner-a'), directory),
        envFor(stub.endpoint, directory, {
          OCLIVE_ONEBOT_OWNER_LEASE_PAUSE_MS: '300',
        }),
      )
      const claimSignal = await first.claimed
      assert.equal(claimSignal?.lease_epoch, 1)
      const second = await runProbe(
        withOutputDir(ownerArgs('claim-send', 'concurrent', 'owner-b'), directory),
        envFor(stub.endpoint, directory),
      )
      const firstResult = await first.done
      assert.equal(firstResult.code, 0, firstResult.stderr)
      assert.equal(second.code, 2, second.stderr)
      assert.equal(sendRequests(stub).length, 1)
      assertNoSensitiveOutput(firstResult)
      assertNoSensitiveOutput(second)

      const database = openOwnerLeaseProbeStore(directory)
      try {
        const row = readOwnerLease(database, 'concurrent')
        assert.equal(row.delivery_state, 'delivered')
        assert.equal(Number(row.lease_epoch), 1)
      }
      finally {
        database.close()
      }
      const databaseBytes = readFileSync(ownerLeaseDatabasePath(directory), 'utf8')
      assert.doesNotMatch(databaseBytes, new RegExp(tokenSentinel, 'u'))
      assert.doesNotMatch(databaseBytes, new RegExp(targetSentinel, 'u'))
      assert.doesNotMatch(databaseBytes, new RegExp(String(providerIdSentinel), 'u'))
      assert.doesNotMatch(databaseBytes, /OCLive A\.2\.2\.2 owner lease probe/u)
    }
    finally {
      await stub.stop()
      rmSync(directory, { force: true, recursive: true })
    }
  })

  await t.test('expired pre-attempt lease is taken over by a new process', async () => {
    const directory = outputDir('pre-attempt-takeover')
    const stub = await startOneBotStub()
    const env = envFor(stub.endpoint, directory, {
      OCLIVE_ONEBOT_OWNER_LEASE_TTL_MS: '100',
    })
    try {
      const abandoned = await runProbe(
        withOutputDir(ownerArgs('claim-only', 'takeover', 'owner-a'), directory),
        env,
      )
      assert.equal(abandoned.code, 0, abandoned.stderr)
      await new Promise(resolvePromise => setTimeout(resolvePromise, 160))
      const recovered = await runProbe(
        withOutputDir(ownerArgs('claim-send', 'takeover', 'owner-b'), directory),
        env,
      )
      assert.equal(recovered.code, 0, recovered.stderr)
      assert.equal(sendRequests(stub).length, 1)
      assertNoSensitiveOutput(abandoned)
      assertNoSensitiveOutput(recovered)
      const database = openOwnerLeaseProbeStore(directory)
      try {
        const row = readOwnerLease(database, 'takeover')
        assert.equal(row.delivery_state, 'delivered')
        assert.equal(Number(row.lease_epoch), 2)
      }
      finally {
        database.close()
      }
    }
    finally {
      await stub.stop()
      rmSync(directory, { force: true, recursive: true })
    }
  })

  await t.test('persisted attempting state blocks failover after process exit', async () => {
    const directory = outputDir('post-attempt-crash')
    const stub = await startOneBotStub()
    const env = envFor(stub.endpoint, directory, {
      OCLIVE_ONEBOT_OWNER_LEASE_TTL_MS: '100',
    })
    try {
      const crashed = await runProbe(
        withOutputDir(ownerArgs('attempt-crash', 'attempting', 'owner-a'), directory),
        env,
      )
      assert.equal(crashed.code, OWNER_LEASE_ATTEMPT_EXIT_CODE)
      await new Promise(resolvePromise => setTimeout(resolvePromise, 160))
      const contender = await runProbe(
        withOutputDir(ownerArgs('claim-send', 'attempting', 'owner-b'), directory),
        env,
      )
      assert.equal(contender.code, 2, contender.stderr)
      assert.equal(sendRequests(stub).length, 0)
      assertNoSensitiveOutput(crashed)
      assertNoSensitiveOutput(contender)
      const database = openOwnerLeaseProbeStore(directory)
      try {
        const row = readOwnerLease(database, 'attempting')
        assert.equal(row.delivery_state, 'attempting')
        assert.equal(Number(row.lease_epoch), 1)
      }
      finally {
        database.close()
      }
    }
    finally {
      await stub.stop()
      rmSync(directory, { force: true, recursive: true })
    }
  })

  const evidence = ownerLeaseEvidence()
  assert.equal(evidence.probe_success, true)
  assert.equal(evidence.production_runtime_enabled, false)
  assert.equal(evidence.remaining_gaps.multi_host_owner_lease_tested, false)
  const evidenceDirectory = join(
    repoRoot,
    'target/oclive-event/onebot-owner-lease-probe/r5-synthetic',
  )
  mkdirSync(evidenceDirectory, { recursive: true })
  writeFileSync(
    join(evidenceDirectory, 'onebot-owner-lease-probe.evidence.json'),
    `${JSON.stringify(evidence, null, 2)}\n`,
    { encoding: 'utf8', mode: 0o600 },
  )
})
