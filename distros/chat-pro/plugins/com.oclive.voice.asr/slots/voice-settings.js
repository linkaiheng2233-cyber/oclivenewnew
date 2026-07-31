(() => {
  'use strict'

  const bridge = window.OclivePluginBridge
  const pluginId = 'com.oclive.voice.asr'
  const configEvent = `${pluginId}:config-updated`
  const byId = id => document.getElementById(id)
  const controls = {
    submitMode: byId('submit-mode'), asrProfile: byId('asr-profile'), asrImport: byId('asr-import'),
    ttsEnabled: byId('tts-enabled'), autoTts: byId('auto-tts'), synthProvider: byId('synth-provider'),
    localEndpoint: byId('local-endpoint'), cloudUrl: byId('cloud-url'), cloudToken: byId('cloud-token'),
    cloudVoice: byId('cloud-voice'), cloudModel: byId('cloud-model'), ttsProfile: byId('tts-profile'),
    directorProfile: byId('director-profile'), ttsImport: byId('tts-import'), adapterImport: byId('adapter-import'),
    roleList: byId('role-list'), roleCount: byId('role-count'),
  }
  const defaults = {
    submit_mode: 'send', tts_expansion_enabled: false, auto_tts: false,
    role_tts_enabled: {},
    asr_profile: 'sherpa-paraformer-zh-small', tts_profile: 'bundled-cosyvoice2-zh',
    director_profile: 'rules-v1', synth_provider: 'bundled',
    local_synth_endpoint: 'http://127.0.0.1:50000', cloud_tts_url: '', cloud_tts_token: '',
    cloud_tts_voice_id: '', cloud_tts_model: 'tts-1',
  }
  let profiles = []
  let roleRows = []
  let roleTtsEnabled = {}
  let rolePolicyExplicit = false

  function setMessage(error = '', ok = '') {
    byId('error').textContent = error
    byId('ok').textContent = ok
  }

  function errorText(reason) {
    return reason instanceof Error ? reason.message : String(reason)
  }

  async function rpc(method, params = {}) {
    if (!bridge) throw new Error('OCLive bridge unavailable')
    return bridge.invoke('plugin_rpc_invoke', { method, params })
  }

  function setOptions(select, rows, selected) {
    const existing = [...select.options].filter(option => option.value === 'none')
    select.replaceChildren(...existing)
    for (const row of rows) {
      const option = document.createElement('option')
      option.value = row.id
      option.textContent = row.label || row.id
      select.append(option)
    }
    if ([...select.options].some(option => option.value === selected)) select.value = selected
  }

  function renderList(id, rows, label) {
    const list = byId(id); list.replaceChildren()
    for (const row of rows) {
      const item = document.createElement('li')
      item.textContent = label(row)
      list.append(item)
    }
  }

  function normalizeRoleTtsEnabled(value) {
    if (!value || typeof value !== 'object' || Array.isArray(value)) return {}
    return Object.fromEntries(
      Object.entries(value)
        .map(([roleId, enabled]) => [roleId.trim(), enabled])
        .filter(([roleId, enabled]) => roleId && enabled === true)
        .map(([roleId]) => [roleId, true]),
    )
  }

  function applyConfig(config) {
    rolePolicyExplicit = Boolean(
      config &&
      Object.prototype.hasOwnProperty.call(config, 'role_tts_enabled') &&
      config.role_tts_enabled &&
      typeof config.role_tts_enabled === 'object' &&
      !Array.isArray(config.role_tts_enabled),
    )
    const value = { ...defaults, ...(config || {}) }
    controls.submitMode.value = value.submit_mode === 'fill' ? 'fill' : 'send'
    controls.ttsEnabled.checked = value.tts_expansion_enabled === true
    controls.autoTts.checked = value.auto_tts === true
    controls.asrProfile.dataset.selected = value.asr_profile
    controls.ttsProfile.dataset.selected = value.tts_profile
    controls.directorProfile.dataset.selected = value.director_profile || 'none'
    controls.synthProvider.value = ['bundled', 'local_http', 'cloud'].includes(value.synth_provider) ? value.synth_provider : 'bundled'
    controls.localEndpoint.value = value.local_synth_endpoint || defaults.local_synth_endpoint
    controls.cloudUrl.value = value.cloud_tts_url || ''
    controls.cloudToken.value = value.cloud_tts_token || ''
    controls.cloudVoice.value = value.cloud_tts_voice_id || ''
    controls.cloudModel.value = value.cloud_tts_model || defaults.cloud_tts_model
    roleTtsEnabled = normalizeRoleTtsEnabled(value.role_tts_enabled)
    renderRoleRows()
    updateVisibility()
  }

  function collectConfig() {
    return {
      submit_mode: controls.submitMode.value,
      tts_expansion_enabled: controls.ttsEnabled.checked,
      auto_tts: controls.ttsEnabled.checked && controls.autoTts.checked,
      role_tts_enabled: roleTtsEnabled,
      asr_profile: controls.asrProfile.value,
      tts_profile: controls.ttsProfile.value,
      director_profile: controls.directorProfile.value,
      synth_provider: controls.synthProvider.value,
      local_synth_endpoint: controls.localEndpoint.value.trim(),
      cloud_tts_url: controls.cloudUrl.value.trim(),
      cloud_tts_token: controls.cloudToken.value,
      cloud_tts_voice_id: controls.cloudVoice.value.trim(),
      cloud_tts_model: controls.cloudModel.value.trim() || defaults.cloud_tts_model,
    }
  }

  function updateVisibility() {
    byId('local-endpoint-row').classList.toggle('hidden', controls.synthProvider.value !== 'local_http')
    byId('cloud-fields').classList.toggle('hidden', controls.synthProvider.value !== 'cloud')
    controls.autoTts.disabled = !controls.ttsEnabled.checked
    byId('role-policy').classList.toggle('hidden', !controls.ttsEnabled.checked)
  }

  function renderRoleRows() {
    controls.roleList.replaceChildren()
    controls.roleCount.textContent = `已启用 ${Object.keys(roleTtsEnabled).length} / ${roleRows.length}`
    for (const role of roleRows) {
      const item = document.createElement('li')
      item.className = 'role-row'
      const label = document.createElement('label')
      const checkbox = document.createElement('input')
      checkbox.type = 'checkbox'
      checkbox.checked = roleTtsEnabled[role.id] === true
      checkbox.disabled = role.voiceConfigured !== true
      checkbox.addEventListener('change', () => {
        rolePolicyExplicit = true
        if (checkbox.checked) roleTtsEnabled = { ...roleTtsEnabled, [role.id]: true }
        else {
          const next = { ...roleTtsEnabled }
          delete next[role.id]
          roleTtsEnabled = next
        }
        renderRoleRows()
      })
      const name = document.createElement('span')
      name.className = 'role-row__name'
      const strong = document.createElement('strong')
      strong.textContent = role.name || role.id
      const small = document.createElement('small')
      small.textContent = role.id
      name.append(strong, small)
      label.append(checkbox, name)
      const status = document.createElement('span')
      status.className = `role-row__status${role.voiceConfigured === true ? ' ready' : ''}`
      status.textContent = role.voiceConfigured === true
        ? `已配置${role.profileLabel ? ` · ${role.profileLabel}` : ''}`
        : role.voiceConfigured === false ? '未配置角色声线' : '检测失败'
      item.append(label, status)
      controls.roleList.append(item)
    }
  }

  function rolePackPath(value) {
    if (typeof value === 'string') return value.trim()
    return typeof value?.role_path === 'string' ? value.role_path.trim() : ''
  }

  async function inspectRoleVoice(role) {
    try {
      const path = rolePackPath(await bridge.invoke('get_role_pack_path', { roleId: role.id }))
      if (!path) return { ...role, voiceConfigured: null, profileLabel: '' }
      const result = await rpc('voice.read_role_profile', { role_path: path })
      const profile = result?.profile
      return {
        ...role,
        voiceConfigured: profile !== null && typeof profile === 'object',
        profileLabel: profile?.preferred_tts_profile || profile?.synth_profile || '',
      }
    }
    catch {
      return { ...role, voiceConfigured: null, profileLabel: '' }
    }
  }

  async function loadRoles() {
    const result = await bridge.invoke('list_roles', {})
    const roles = Array.isArray(result)
      ? result
          .filter(role => role && typeof role.id === 'string')
          .map(role => ({
            id: role.id.trim(),
            name: typeof role.name === 'string' && role.name.trim() ? role.name.trim() : role.id.trim(),
          }))
          .filter(role => role.id)
      : []
    roleRows = await Promise.all(roles.map(inspectRoleVoice))
    if (!rolePolicyExplicit) {
      roleTtsEnabled = Object.fromEntries(
        roleRows.filter(role => role.voiceConfigured === true).map(role => [role.id, true]),
      )
    }
    else {
      const missing = new Set(
        roleRows.filter(role => role.voiceConfigured === false).map(role => role.id),
      )
      roleTtsEnabled = Object.fromEntries(
        Object.entries(roleTtsEnabled).filter(([roleId]) => !missing.has(roleId)),
      )
    }
    renderRoleRows()
  }

  function applyProfileDefaults() {
    const profile = profiles.find(row => row.id === controls.ttsProfile.value)
    if (['bundled', 'local_http', 'cloud'].includes(profile?.synth_provider))
      controls.synthProvider.value = profile.synth_provider
    if (profile?.sidecar_endpoint) controls.localEndpoint.value = profile.sidecar_endpoint
    const engine = profile?.engine || ''
    byId('compliance').textContent = engine === 'gpt-sovits-http'
      ? 'GPT-SoVITS：参考音色须为原创或已授权来源。'
      : ['qwen3-tts-http', 'fish-speech-http', 'indextts-http'].includes(engine)
        ? '用户本地 TTS：算力与音色来源由用户负责。'
        : engine === 'generic-http-adapter' ? '请确认 endpoint 与请求模板符合目标服务契约。' : ''
    updateVisibility()
  }

  async function pushConfig(config) {
    await rpc('config_updated', { config })
  }

  async function reload() {
    setMessage()
    try {
      const list = await rpc('voice.list_profiles')
      profiles = Array.isArray(list?.profiles) ? list.profiles : []
      const kind = name => profiles.filter(row => (row.kind || 'asr') === name)
      setOptions(controls.asrProfile, kind('asr'), controls.asrProfile.dataset.selected || controls.asrProfile.value)
      setOptions(controls.ttsProfile, kind('tts'), controls.ttsProfile.dataset.selected || controls.ttsProfile.value)
      setOptions(controls.directorProfile, kind('director'), controls.directorProfile.dataset.selected || controls.directorProfile.value)
      applyProfileDefaults()
      const [packs, adapters, asrProbe] = await Promise.all([
        rpc('voice.list_model_packs'), rpc('voice.list_tts_adapters'),
        rpc('voice.probe', { profile: controls.asrProfile.value }),
      ])
      renderList('model-packs', packs?.packs || [], row => `${row.label || row.pack_id} · ${row.installed ? '已安装' : '未安装'}`)
      renderList('adapters', adapters?.adapters || [], row => `${row.label || row.adapter_id}${row.api_style ? ` · ${row.api_style}` : ''}`)
      byId('asr-probe').textContent = JSON.stringify(asrProbe, null, 2)
      byId('tts-probe').textContent = controls.ttsEnabled.checked
        ? JSON.stringify(await rpc('voice.probe_tts', { profile: controls.ttsProfile.value }), null, 2) : 'TTS 扩展未启用'
    }
    catch (reason) { setMessage(errorText(reason)) }
  }

  async function save() {
    setMessage(); byId('save').disabled = true
    try {
      applyProfileDefaults()
      const config = collectConfig()
      await bridge.invoke('set_plugin_settings_config', { pluginId, config })
      await pushConfig(config)
      await bridge.emit(configEvent, {})
      setMessage('', '设置已保存')
      await reload()
    }
    catch (reason) { setMessage(errorText(reason)) }
    finally { byId('save').disabled = false }
  }

  async function importModel(kind) {
    const input = kind === 'asr' ? controls.asrImport : controls.ttsImport
    if (!input.value.trim()) { setMessage('请填写模型目录路径'); return }
    try {
      const result = await rpc('voice.import_model', {
        src_path: input.value.trim(), kind,
        profile: kind === 'asr' ? controls.asrProfile.value : controls.ttsProfile.value,
      })
      if (!result?.ok) throw new Error(result?.message || result?.reason || '导入失败')
      input.value = ''; setMessage('', '模型导入完成'); await reload()
    }
    catch (reason) { setMessage(errorText(reason)) }
  }

  async function importAdapter() {
    if (!controls.adapterImport.value.trim()) { setMessage('请填写 TTS 适配包目录路径'); return }
    try {
      const result = await rpc('voice.import_tts_adapter', { src_path: controls.adapterImport.value.trim() })
      if (!result?.ok) throw new Error(result?.message || result?.reason || '导入失败')
      controls.adapterImport.value = ''; setMessage('', '适配包导入完成'); await reload()
    }
    catch (reason) { setMessage(errorText(reason)) }
  }

  async function warm() {
    const button = byId('warm'); button.disabled = true; setMessage()
    try {
      const result = await rpc('voice.warm', { profile: controls.ttsProfile.value })
      if (!result?.ok && !result?.skipped && !result?.already_warmed)
        throw new Error(result?.message || result?.reason || '预热失败')
      setMessage('', result?.message || (result?.already_warmed ? '侧车已预热' : '预热完成'))
      await reload()
    }
    catch (reason) { setMessage(errorText(reason)) }
    finally { button.disabled = false }
  }

  controls.synthProvider.addEventListener('change', updateVisibility)
  controls.ttsEnabled.addEventListener('change', updateVisibility)
  controls.ttsProfile.addEventListener('change', applyProfileDefaults)
  byId('save').addEventListener('click', save)
  byId('reload').addEventListener('click', reload)
  byId('import-asr').addEventListener('click', () => importModel('asr'))
  byId('import-tts').addEventListener('click', () => importModel('tts'))
  byId('import-adapter').addEventListener('click', importAdapter)
  byId('warm').addEventListener('click', warm)

  async function initialize() {
    if (!bridge) { setMessage('OCLive bridge unavailable'); return }
    try {
      const ui = await bridge.invoke('get_plugin_settings_ui', { pluginId })
      applyConfig(ui?.config)
      await pushConfig(collectConfig())
      await Promise.all([loadRoles(), reload()])
    }
    catch (reason) { setMessage(errorText(reason)) }
  }
  void initialize()
})()
