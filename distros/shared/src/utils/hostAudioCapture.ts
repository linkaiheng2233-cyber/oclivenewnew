const MIC_CONSTRAINTS: MediaTrackConstraints = {
  echoCancellation: true,
  noiseSuppression: true,
  autoGainControl: true,
  channelCount: 1,
}

const RECORDER_MIME_TYPES = [
  'audio/webm;codecs=opus',
  'audio/webm',
  'audio/ogg;codecs=opus',
  'audio/mp4',
]

export interface HostAudioCaptureResult {
  audioBase64: string
  mimeType: string
  durationMs: number
}

type CaptureState = 'idle' | 'starting' | 'recording' | 'stopping'

interface CaptureSession {
  generation: number
  stream: MediaStream
  recorder: MediaRecorder
  chunks: Blob[]
  startedAt: number
}

function bytesToBase64(bytes: Uint8Array): string {
  let binary = ''
  for (let offset = 0; offset < bytes.length; offset += 0x8000)
    binary += String.fromCharCode(...bytes.subarray(offset, offset + 0x8000))
  return btoa(binary)
}

function recorderMimeType(): string {
  return RECORDER_MIME_TYPES.find(type => MediaRecorder.isTypeSupported(type)) ?? ''
}

/** Capture microphone audio in the trusted host for an opaque-origin plugin frame. */
export function createHostAudioCapture() {
  let state: CaptureState = 'idle'
  let generation = 0
  let session: CaptureSession | null = null

  function release(active: CaptureSession): void {
    active.stream.getTracks().forEach(track => track.stop())
    if (session === active) {
      session = null
      state = 'idle'
    }
  }

  async function start(): Promise<{ mimeType: string }> {
    if (state !== 'idle')
      throw new Error('audio capture already active')
    if (!navigator.mediaDevices?.getUserMedia)
      throw new Error('host microphone capture unavailable')

    state = 'starting'
    const startGeneration = ++generation
    let acquired: MediaStream
    try {
      acquired = await navigator.mediaDevices.getUserMedia({ audio: MIC_CONSTRAINTS })
    }
    catch (error) {
      if (generation === startGeneration)
        state = 'idle'
      throw error
    }
    if (generation !== startGeneration || state !== 'starting') {
      acquired.getTracks().forEach(track => track.stop())
      throw new Error('audio capture start cancelled')
    }

    try {
      const mimeType = recorderMimeType()
      const recorder = mimeType
        ? new MediaRecorder(acquired, { mimeType })
        : new MediaRecorder(acquired)
      const active: CaptureSession = {
        generation: startGeneration,
        stream: acquired,
        recorder,
        chunks: [],
        startedAt: Date.now(),
      }
      recorder.ondataavailable = (event) => {
        if (event.data.size)
          active.chunks.push(event.data)
      }
      session = active
      state = 'recording'
      recorder.start()
      return { mimeType: recorder.mimeType }
    }
    catch (error) {
      acquired.getTracks().forEach(track => track.stop())
      if (generation === startGeneration) {
        session = null
        state = 'idle'
      }
      throw error
    }
  }

  async function stop(): Promise<HostAudioCaptureResult> {
    const active = session
    if (!active || state !== 'recording' || active.recorder.state === 'inactive')
      throw new Error('audio capture is not active')
    state = 'stopping'

    return new Promise((resolve, reject) => {
      active.recorder.onerror = () => {
        release(active)
        reject(new Error('host microphone recording failed'))
      }
      active.recorder.onstop = async () => {
        const wasCancelled = generation !== active.generation
        const durationMs = Date.now() - active.startedAt
        const mimeType = active.recorder.mimeType || 'audio/webm'
        const blob = new Blob(active.chunks, { type: mimeType })
        release(active)
        if (wasCancelled) {
          reject(new Error('audio capture cancelled'))
          return
        }
        try {
          resolve({
            audioBase64: bytesToBase64(new Uint8Array(await blob.arrayBuffer())),
            mimeType,
            durationMs,
          })
        }
        catch (error) {
          reject(error)
        }
      }
      active.recorder.stop()
    })
  }

  function cancel(): null {
    generation += 1
    state = 'idle'
    const active = session
    session = null
    if (active) {
      if (active.recorder.state !== 'inactive')
        active.recorder.stop()
      active.stream.getTracks().forEach(track => track.stop())
    }
    return null
  }

  return { start, stop, cancel }
}
