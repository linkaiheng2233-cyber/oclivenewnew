import { afterEach, describe, expect, it, vi } from 'vitest'
import { createHostAudioCapture } from './hostAudioCapture'

class FakeMediaRecorder {
  static instances: FakeMediaRecorder[] = []
  static isTypeSupported = vi.fn(() => true)

  state: RecordingState = 'inactive'
  mimeType = 'audio/webm'
  ondataavailable: ((event: BlobEvent) => void) | null = null
  onerror: (() => void) | null = null
  onstop: (() => void) | null = null

  constructor(_stream: MediaStream, _options?: MediaRecorderOptions) {
    FakeMediaRecorder.instances.push(this)
  }

  start() {
    this.state = 'recording'
  }

  stop() {
    this.state = 'inactive'
    queueMicrotask(() => this.onstop?.())
  }
}

function deferred<T>() {
  let resolve!: (value: T) => void
  let reject!: (reason?: unknown) => void
  const promise = new Promise<T>((res, rej) => {
    resolve = res
    reject = rej
  })
  return { promise, resolve, reject }
}

function fakeStream() {
  const stop = vi.fn()
  return {
    stream: { getTracks: () => [{ stop }] } as unknown as MediaStream,
    stop,
  }
}

describe('hostAudioCapture race guards', () => {
  afterEach(() => {
    vi.unstubAllGlobals()
    FakeMediaRecorder.instances = []
  })

  it('cancels a pending microphone grant without starting a recorder', async () => {
    const grant = deferred<MediaStream>()
    const { stream, stop } = fakeStream()
    vi.stubGlobal('navigator', { mediaDevices: { getUserMedia: vi.fn(() => grant.promise) } })
    vi.stubGlobal('MediaRecorder', FakeMediaRecorder)
    const capture = createHostAudioCapture()

    const starting = capture.start()
    capture.cancel()
    grant.resolve(stream)

    await expect(starting).rejects.toThrow('audio capture start cancelled')
    expect(stop).toHaveBeenCalledOnce()
    expect(FakeMediaRecorder.instances).toHaveLength(0)
  })

  it('rejects overlapping starts while permission is pending', async () => {
    const grant = deferred<MediaStream>()
    const { stream } = fakeStream()
    vi.stubGlobal('navigator', { mediaDevices: { getUserMedia: vi.fn(() => grant.promise) } })
    vi.stubGlobal('MediaRecorder', FakeMediaRecorder)
    const capture = createHostAudioCapture()

    const first = capture.start()
    await expect(capture.start()).rejects.toThrow('audio capture already active')
    grant.resolve(stream)
    await expect(first).resolves.toEqual({ mimeType: 'audio/webm' })
    capture.cancel()
  })

  it('rejects an in-flight stop when capture is cancelled', async () => {
    const { stream, stop } = fakeStream()
    vi.stubGlobal('navigator', { mediaDevices: { getUserMedia: vi.fn().mockResolvedValue(stream) } })
    vi.stubGlobal('MediaRecorder', FakeMediaRecorder)
    const capture = createHostAudioCapture()

    await capture.start()
    const stopping = capture.stop()
    capture.cancel()

    await expect(stopping).rejects.toThrow('audio capture cancelled')
    expect(stop).toHaveBeenCalled()
  })
})
