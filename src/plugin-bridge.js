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
  if (inv.indexOf(n) >= 0)
    return true
  const p = CMD_PERM[n]
  return p && inv.indexOf(p) >= 0
}

function injectOclivePluginBridge(pluginId, assetRel, inv, ev) {
  function invoke(n, p) {
    if (!bridgeAllowed(n, inv))
      return Promise.reject(new Error(`invoke denied:${n}`))
    const _inv = window.__TAURI__
      && (window.__TAURI__.invoke || (window.__TAURI__.tauri && window.__TAURI__.tauri.invoke))
    if (!_inv)
      return Promise.reject(new Error('no invoke API'))
    return _inv('plugin_bridge_invoke', {
      req: {
        pluginId,
        assetRel,
        command: n,
        params: p != null ? p : {},
      },
    })
  }

  function listen(e, c) {
    if (!ev.includes(e))
      return Promise.reject(new Error(`event denied:${e}`))
    const T = window.__TAURI__
    const t = T && (T.event || (T.tauri && T.tauri.event))
    if (!t)
      return Promise.reject(new Error('no event API'))
    return t.listen(e, c)
  }

  window.OclivePluginBridge = {
    invoke,
    listen,
    allowedInvoke: inv,
    allowedEvents: ev,
  }
}

if (typeof window !== 'undefined')
  window.__oclivSetupPluginBridge = injectOclivePluginBridge
