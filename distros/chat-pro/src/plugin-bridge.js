/** Plugin HTML bridge core — Vite IIFE bundle; Rust calls `__oclivSetupPluginBridge(...)` after include. */
const CMD_PERM = {
  get_conversation: 'read:conversation',
  get_roles: 'read:roles',
  get_current_role: 'read:current_role',
  update_memory: 'write:memory',
  delete_memory: 'write:memory',
  update_emotion: 'write:emotion',
  update_event: 'write:event',
  update_prompt: 'write:prompt',
  export_conversation: 'export:conversation',
  import_role: 'import:role',
  delete_role: 'delete:role',
  update_settings: 'write:settings',
  get_conversation_list: 'read:conversations',
}

function bridgeAllowed(n, inv) {
  if (inv.includes(n))
    return true
  const p = CMD_PERM[n]
  return p && inv.includes(p)
}

function injectOclivePluginBridge(pluginId, assetRel, inv, ev) {
  const FRAME_CHANNEL = 'oclive-plugin-frame-bridge-v1'
  const invokeAllowlist = Object.freeze([...inv])
  const eventAllowlist = Object.freeze([...ev])
  const pending = new Map()
  const subscriptions = new Map()
  let frameToken = null
  let resolveFrameBinding
  const frameBinding = new Promise((resolve) => {
    resolveFrameBinding = resolve
  })
  let requestSequence = 0

  function nextRequestId() {
    if (globalThis.crypto && typeof globalThis.crypto.randomUUID === 'function')
      return globalThis.crypto.randomUUID()
    requestSequence += 1
    return `${Date.now().toString(36)}-${requestSequence.toString(36)}`
  }

  function requestThroughParent(kind, payload) {
    return frameBinding.then(token => new Promise((resolve, reject) => {
      const requestId = nextRequestId()
      const timer = window.setTimeout(() => {
        pending.delete(requestId)
        reject(new Error('plugin frame bridge timeout'))
      }, 30_000)
      pending.set(requestId, { resolve, reject, timer })
      window.parent.postMessage({
        channel: FRAME_CHANNEL,
        kind,
        requestId,
        token,
        ...payload,
      }, '*')
    }))
  }

  if (window.parent !== window) {
    window.addEventListener('message', (event) => {
      if (event.source !== window.parent)
        return
      const data = event.data
      if (!data || data.channel !== FRAME_CHANNEL)
        return
      if (data.kind === 'bind') {
        const token = data.value && data.value.token
        if (!frameToken && typeof token === 'string' && token.length >= 32) {
          frameToken = token
          resolveFrameBinding(token)
        }
        return
      }
      if (data.kind === 'event') {
        const callback = subscriptions.get(data.requestId)
        if (callback)
          callback(data.value?.data)
        return
      }
      if (data.kind !== 'result')
        return
      const waiter = pending.get(data.requestId)
      if (!waiter)
        return
      pending.delete(data.requestId)
      window.clearTimeout(waiter.timer)
      if (data.ok)
        waiter.resolve(data.value)
      else
        waiter.reject(new Error(data.error || 'plugin frame bridge rejected'))
    })
  }

  function invoke(n, p) {
    if (!bridgeAllowed(n, invokeAllowlist))
      return Promise.reject(new Error(`invoke denied:${n}`))
    if (window.parent !== window)
      return requestThroughParent('invoke', { command: n, params: p ?? {} })
    const _inv = window.__TAURI__
      && (window.__TAURI__.invoke || (window.__TAURI__.tauri && window.__TAURI__.tauri.invoke))
    if (!_inv)
      return Promise.reject(new Error('no invoke API'))
    return _inv('plugin_bridge_invoke', {
      req: {
        pluginId,
        assetRel,
        command: n,
        params: p ?? {},
      },
    })
  }

  function listen(e, c) {
    const isPluginEvent = e.startsWith(`${pluginId}:`) && e.length > pluginId.length + 1
    if (!isPluginEvent && !eventAllowlist.includes(e))
      return Promise.reject(new Error(`event denied:${e}`))
    if (window.parent !== window) {
      return requestThroughParent('subscribe', { event: e }).then(({ subscriptionId }) => {
        subscriptions.set(subscriptionId, c)
        return () => {
          subscriptions.delete(subscriptionId)
          return requestThroughParent('unsubscribe', { subscriptionId })
        }
      })
    }
    const T = window.__TAURI__
    const t = T && (T.event || (T.tauri && T.tauri.event))
    if (!t)
      return Promise.reject(new Error('no event API'))
    return t.listen(e, c)
  }

  function emit(e, data) {
    if (!e.startsWith(`${pluginId}:`) || e.length === pluginId.length + 1)
      return Promise.reject(new Error(`event namespace denied:${e}`))
    if (window.parent !== window)
      return requestThroughParent('emit', { event: e, data })
    return Promise.reject(new Error('plugin event emit is only available in isolated frames'))
  }

  const audioCapture = Object.freeze({
    start() {
      if (window.parent === window)
        return Promise.reject(new Error('host audio capture requires an isolated frame'))
      return requestThroughParent('audio-start', {})
    },
    stop() {
      if (window.parent === window)
        return Promise.reject(new Error('host audio capture requires an isolated frame'))
      return requestThroughParent('audio-stop', {})
    },
    cancel() {
      if (window.parent === window)
        return Promise.reject(new Error('host audio capture requires an isolated frame'))
      return requestThroughParent('audio-cancel', {})
    },
  })

  window.OclivePluginBridge = {
    invoke,
    listen,
    emit,
    audioCapture,
    allowedInvoke: invokeAllowlist,
    allowedEvents: eventAllowlist,
  }
}

if (typeof window !== 'undefined')
  window.__oclivSetupPluginBridge = injectOclivePluginBridge
