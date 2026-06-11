/**
 * Theater director directory plugin — minimal JSON-RPC stub (Phase 4).
 * Methods: theater.director.ping | validate_rules | inject_beat | switch_scene
 * See handoff/RFC_THEATER_DIRECTOR_PLUGIN.md
 */
import http from 'node:http'

const PROTOCOL_HEADER = 'x-oclive-remote-protocol'
const PROTOCOL_VALUE = 'oclive-remote-jsonrpc-v1'

function jsonRpcResult(id, result) {
  return JSON.stringify({ jsonrpc: '2.0', id, result })
}

function jsonRpcError(id, code, message) {
  return JSON.stringify({
    jsonrpc: '2.0',
    id,
    error: { code, message },
  })
}

function handlePing() {
  return { ok: true, plugin: 'com.oclive.theater.director', version: '0.1.0' }
}

function handleValidateRules(params) {
  const sceneId = params?.scene_id ?? 'breakfast'
  const beats = Array.isArray(params?.beats) ? params.beats : []
  const violations = []
  if (beats.length === 0) {
    violations.push('beats_empty')
  }
  if (sceneId !== 'breakfast') {
    violations.push('scene_not_shipped_yet')
  }
  return { valid: violations.length === 0, violations }
}

function handleInjectBeat(params) {
  const beatId = params?.beat_id ?? `injected_${Date.now()}`
  const summary = String(params?.summary ?? '').trim() || '（导演注入）'
  const speaker = params?.speaker === 'b' ? 'b' : 'a'
  return {
    beat: {
      id: beatId,
      speaker,
      text: summary,
      delay_ms: 0,
    },
  }
}

function handleSwitchScene(params) {
  const sceneId = String(params?.scene_id ?? '').trim()
  if (!sceneId) {
    return { ok: false, error: 'scene_id_required' }
  }
  return { ok: true, scene_id: sceneId, skeleton_path: `/theater/${sceneId}/skeleton.json` }
}

const handlers = {
  'theater.director.ping': handlePing,
  'theater.director.validate_rules': handleValidateRules,
  'theater.director.inject_beat': handleInjectBeat,
  'theater.director.switch_scene': handleSwitchScene,
}

const server = http.createServer((req, res) => {
  if (req.method !== 'POST' || !req.url || !req.url.startsWith('/rpc')) {
    res.writeHead(404, { 'Content-Type': 'text/plain; charset=utf-8' })
    res.end('not found')
    return
  }
  const chunks = []
  req.on('data', c => chunks.push(c))
  req.on('end', () => {
    const raw = Buffer.concat(chunks).toString('utf8')
    let msg
    try {
      msg = JSON.parse(raw)
    }
    catch {
      res.writeHead(400, { 'Content-Type': 'application/json; charset=utf-8' })
      res.end(jsonRpcError(null, -32700, 'parse error'))
      return
    }
    const id = msg.id ?? null
    if (msg.jsonrpc !== '2.0' || typeof msg.method !== 'string') {
      res.writeHead(400, { 'Content-Type': 'application/json; charset=utf-8' })
      res.end(jsonRpcError(id, -32600, 'invalid request'))
      return
    }
    res.setHeader('Content-Type', 'application/json; charset=utf-8')
    res.setHeader(PROTOCOL_HEADER, PROTOCOL_VALUE)
    const handler = handlers[msg.method]
    if (!handler) {
      res.writeHead(200)
      res.end(jsonRpcError(id, -32601, `method not found: ${msg.method}`))
      return
    }
    try {
      const result = handler(msg.params ?? {})
      res.writeHead(200)
      res.end(jsonRpcResult(id, result))
    }
    catch (e) {
      res.writeHead(200)
      res.end(jsonRpcError(id, -32603, e instanceof Error ? e.message : String(e)))
    }
  })
})

server.listen(0, '127.0.0.1', () => {
  const addr = server.address()
  const port = typeof addr === 'object' && addr ? addr.port : 0
  process.stdout.write(`OCLIVE_READY http://127.0.0.1:${port}/rpc\n`)
})

process.on('SIGTERM', () => server.close())
process.on('SIGINT', () => server.close())
