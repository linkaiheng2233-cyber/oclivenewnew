/** Browser mic blob → 16 kHz mono WAV (base64). SSOT mirror: inlined in `VoiceToolbar.vue` (ui_slots vue3-sfc-loader cannot load sibling .ts). */

export const TARGET_SAMPLE_RATE = 16000
export const MIN_RECORD_MS = 350

export const MIC_CONSTRAINTS: MediaTrackConstraints = {
  echoCancellation: true,
  noiseSuppression: true,
  autoGainControl: true,
  channelCount: 1,
}

function bytesToBase64(bytes: Uint8Array): string {
  let binary = ''
  const chunk = 0x8000
  for (let i = 0; i < bytes.length; i += chunk) {
    binary += String.fromCharCode(...bytes.subarray(i, i + chunk))
  }
  return btoa(binary)
}

function encodeWavPcm16(samples: Float32Array, sampleRate: number): Uint8Array {
  const dataSize = samples.length * 2
  const buffer = new ArrayBuffer(44 + dataSize)
  const view = new DataView(buffer)
  const writeAscii = (offset: number, text: string) => {
    for (let i = 0; i < text.length; i += 1)
      view.setUint8(offset + i, text.charCodeAt(i))
  }
  writeAscii(0, 'RIFF')
  view.setUint32(4, 36 + dataSize, true)
  writeAscii(8, 'WAVE')
  writeAscii(12, 'fmt ')
  view.setUint32(16, 16, true)
  view.setUint16(20, 1, true)
  view.setUint16(22, 1, true)
  view.setUint32(24, sampleRate, true)
  view.setUint32(28, sampleRate * 2, true)
  view.setUint16(32, 2, true)
  view.setUint16(34, 16, true)
  writeAscii(36, 'data')
  view.setUint32(40, dataSize, true)
  let offset = 44
  for (let i = 0; i < samples.length; i += 1) {
    const clamped = Math.max(-1, Math.min(1, samples[i]))
    view.setInt16(offset, clamped < 0 ? clamped * 0x8000 : clamped * 0x7fff, true)
    offset += 2
  }
  return new Uint8Array(buffer)
}

function mixToMono(decoded: AudioBuffer): Float32Array {
  const length = decoded.length
  const mono = new Float32Array(length)
  const ch0 = decoded.getChannelData(0)
  if (decoded.numberOfChannels === 1) {
    mono.set(ch0)
    return mono
  }
  for (let i = 0; i < length; i += 1) {
    let sum = ch0[i]
    for (let c = 1; c < decoded.numberOfChannels; c += 1)
      sum += decoded.getChannelData(c)[i]
    mono[i] = sum / decoded.numberOfChannels
  }
  return mono
}

async function resampleTo16kMono(mono: Float32Array, srcRate: number, durationSec: number): Promise<Float32Array> {
  const offline = new OfflineAudioContext(
    1,
    Math.max(1, Math.ceil(durationSec * TARGET_SAMPLE_RATE)),
    TARGET_SAMPLE_RATE,
  )
  const monoBuffer = offline.createBuffer(1, mono.length, srcRate)
  monoBuffer.copyToChannel(mono, 0)
  const source = offline.createBufferSource()
  source.buffer = monoBuffer
  source.connect(offline.destination)
  source.start(0)
  const rendered = await offline.startRendering()
  return rendered.getChannelData(0)
}

/** Decode WebM/Opus (or other browser codec) and emit WAV base64 for the Python ASR engine. */
export async function blobToWav16kMonoBase64(blob: Blob): Promise<string> {
  if (!blob.size)
    throw new Error('录音为空')
  const arrayBuffer = await blob.arrayBuffer()
  if (arrayBuffer.byteLength >= 4) {
    const head = new Uint8Array(arrayBuffer, 0, 4)
    if (head[0] === 0x52 && head[1] === 0x49 && head[2] === 0x46 && head[3] === 0x46)
      return bytesToBase64(new Uint8Array(arrayBuffer))
  }
  const ctx = new AudioContext()
  try {
    const decoded = await ctx.decodeAudioData(arrayBuffer.slice(0))
    if (decoded.duration * 1000 < MIN_RECORD_MS)
      throw new Error('录音太短，请按住多说一会')
    const mono = mixToMono(decoded)
    const pcm = await resampleTo16kMono(mono, decoded.sampleRate, decoded.duration)
    return bytesToBase64(encodeWavPcm16(pcm, TARGET_SAMPLE_RATE))
  }
  finally {
    await ctx.close()
  }
}

export function pickMediaRecorderMime(): string {
  const candidates = [
    'audio/webm;codecs=opus',
    'audio/webm',
    'audio/ogg;codecs=opus',
    'audio/mp4',
  ]
  for (const mime of candidates) {
    if (MediaRecorder.isTypeSupported(mime))
      return mime
  }
  return ''
}
