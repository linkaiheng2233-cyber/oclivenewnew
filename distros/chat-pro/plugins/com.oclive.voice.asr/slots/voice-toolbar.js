(() => {
  'use strict'

  const bridge = window.OclivePluginBridge
  const pluginId = 'com.oclive.voice.asr'
  const submitEvent = `${pluginId}:submit`
  const holdEvent = `${pluginId}:hold`
  const targetSampleRate = 16000
  const minRecordMs = 350
  const button = document.querySelector('#record')
  const status = document.querySelector('#status')
  const error = document.querySelector('#error')
  let stream = null
  let recorder = null
  let chunks = []
  let startedAt = 0
  let busy = false
  let recording = false
  let submitMode = 'send'
  let asrProfile = 'sherpa-paraformer-zh-small'
  let stopListening = null

  function setState(next = {}) {
    if (typeof next.busy === 'boolean') busy = next.busy
    if (typeof next.recording === 'boolean') recording = next.recording
    if ('status' in next) status.textContent = next.status || ''
    if ('error' in next) error.textContent = next.error || ''
    button.disabled = !bridge || busy
    button.classList.toggle('recording', recording)
    button.textContent = busy ? '识别中…' : recording ? '录音中…' : '🎤 按住说话'
  }

  async function rpc(method, params = {}) {
    if (!bridge) throw new Error('OCLive bridge unavailable')
    return bridge.invoke('plugin_rpc_invoke', { method, params })
  }

  function bytesToBase64(bytes) {
    let binary = ''
    for (let i = 0; i < bytes.length; i += 0x8000)
      binary += String.fromCharCode(...bytes.subarray(i, i + 0x8000))
    return btoa(binary)
  }

  function encodeWav(samples, sampleRate) {
    const buffer = new ArrayBuffer(44 + samples.length * 2)
    const view = new DataView(buffer)
    const ascii = (offset, text) => {
      for (let i = 0; i < text.length; i += 1) view.setUint8(offset + i, text.charCodeAt(i))
    }
    ascii(0, 'RIFF'); view.setUint32(4, 36 + samples.length * 2, true)
    ascii(8, 'WAVE'); ascii(12, 'fmt '); view.setUint32(16, 16, true)
    view.setUint16(20, 1, true); view.setUint16(22, 1, true)
    view.setUint32(24, sampleRate, true); view.setUint32(28, sampleRate * 2, true)
    view.setUint16(32, 2, true); view.setUint16(34, 16, true)
    ascii(36, 'data'); view.setUint32(40, samples.length * 2, true)
    for (let i = 0, offset = 44; i < samples.length; i += 1, offset += 2) {
      const sample = Math.max(-1, Math.min(1, samples[i]))
      view.setInt16(offset, sample < 0 ? sample * 0x8000 : sample * 0x7fff, true)
    }
    return new Uint8Array(buffer)
  }

  async function blobToWavBase64(blob) {
    if (!blob.size) throw new Error('录音为空')
    const raw = await blob.arrayBuffer()
    const audio = new AudioContext()
    try {
      const decoded = await audio.decodeAudioData(raw.slice(0))
      if (decoded.duration * 1000 < minRecordMs) throw new Error('录音太短，请按住多说一会')
      const mono = new Float32Array(decoded.length)
      for (let channel = 0; channel < decoded.numberOfChannels; channel += 1) {
        const input = decoded.getChannelData(channel)
        for (let i = 0; i < input.length; i += 1) mono[i] += input[i] / decoded.numberOfChannels
      }
      const offline = new OfflineAudioContext(1, Math.max(1, Math.ceil(decoded.duration * targetSampleRate)), targetSampleRate)
      const sourceBuffer = offline.createBuffer(1, mono.length, decoded.sampleRate)
      sourceBuffer.copyToChannel(mono, 0)
      const source = offline.createBufferSource()
      source.buffer = sourceBuffer; source.connect(offline.destination); source.start(0)
      const rendered = await offline.startRendering()
      return bytesToBase64(encodeWav(rendered.getChannelData(0), targetSampleRate))
    }
    finally {
      await audio.close()
    }
  }

  function cleanup() {
    stream?.getTracks().forEach(track => track.stop())
    stream = null; recorder = null
    setState({ recording: false })
  }

  async function transcribe(blob) {
    if (busy) return
    setState({ busy: true, error: '', status: '识别中…' })
    try {
      const result = await rpc('voice.transcribe', {
        profile: asrProfile,
        audio_base64: await blobToWavBase64(blob),
      })
      const text = String(result?.text || '').trim()
      if (!result?.ok || !text) {
        const hint = result?.reason === 'audio_too_quiet'
          ? '声音太小或未检测到语音，请靠近麦克风再说'
          : result?.reason === 'bad_audio_format'
            ? '音频格式无法识别，请重试'
            : result?.message || result?.reason || '识别无结果'
        setState({ error: hint, status: '' })
        return
      }
      await bridge.emit(submitEvent, { text, mode: submitMode })
      setState({ status: '识别完成' })
    }
    catch (reason) {
      setState({ error: reason instanceof Error ? reason.message : String(reason), status: '' })
    }
    finally {
      setState({ busy: false })
    }
  }

  async function startRecording() {
    if (!bridge || busy || recording) return
    setState({ error: '' })
    try {
      if (!navigator.mediaDevices?.getUserMedia) throw new Error('此环境不支持麦克风')
      stream = await navigator.mediaDevices.getUserMedia({ audio: {
        echoCancellation: true, noiseSuppression: true, autoGainControl: true, channelCount: 1,
      } })
      chunks = []; startedAt = Date.now()
      const mime = ['audio/webm;codecs=opus', 'audio/webm', 'audio/ogg;codecs=opus', 'audio/mp4']
        .find(type => MediaRecorder.isTypeSupported(type))
      recorder = mime ? new MediaRecorder(stream, { mimeType: mime }) : new MediaRecorder(stream)
      recorder.ondataavailable = event => { if (event.data.size) chunks.push(event.data) }
      recorder.onstop = () => {
        const elapsed = Date.now() - startedAt
        const blob = new Blob(chunks, { type: recorder?.mimeType || 'audio/webm' })
        cleanup()
        if (elapsed < minRecordMs) setState({ error: '录音太短，请按住多说一会', status: '' })
        else void transcribe(blob)
      }
      recorder.start(); setState({ recording: true, status: '录音中…' })
    }
    catch (reason) {
      cleanup()
      setState({ error: reason instanceof Error ? reason.message : String(reason), status: '' })
    }
  }

  function stopRecording() {
    if (recorder && recording) recorder.stop()
  }

  button.addEventListener('pointerdown', (event) => {
    if (event.button !== 0 && event.button !== -1) return
    event.preventDefault(); button.setPointerCapture?.(event.pointerId); void startRecording()
  })
  for (const type of ['pointerup', 'pointercancel', 'pointerleave']) {
    button.addEventListener(type, (event) => {
      if (type === 'pointerup') button.releasePointerCapture?.(event.pointerId)
      stopRecording()
    })
  }

  async function initialize() {
    if (!bridge) { setState({ error: 'OCLive bridge unavailable' }); return }
    try {
      const ui = await bridge.invoke('get_plugin_settings_ui', { pluginId })
      submitMode = ui?.config?.submit_mode === 'fill' ? 'fill' : 'send'
      if (typeof ui?.config?.asr_profile === 'string' && ui.config.asr_profile.trim())
        asrProfile = ui.config.asr_profile.trim()
      const probe = await rpc('voice.probe', { profile: asrProfile })
      setState({ status: probe?.message || (probe?.ok ? '就绪' : probe?.reason || '未就绪'), error: '' })
      stopListening = await bridge.listen(holdEvent, payload => {
        if (payload?.phase === 'start') void startRecording()
        if (payload?.phase === 'stop') stopRecording()
      })
    }
    catch (reason) {
      setState({ error: reason instanceof Error ? reason.message : String(reason) })
    }
  }

  window.addEventListener('pagehide', () => { stopListening?.(); cleanup() }, { once: true })
  void initialize()
})()
