export type VoiceSpeakStage = 'stream' | 'rpc'

const REASON_LABELS: Record<string, string> = {
  stream_timeout: '侧车流式请求超时',
  stream_first_chunk_timeout: '侧车首包超时',
  stream_playback_timeout: '侧车播放等待超时',
  stream_read_failed: '侧车流读取失败',
  stream_error: '侧车返回错误',
  http_error: '侧车 HTTP 错误',
  cosyvoice_empty: '侧车未返回音频',
  empty_text: '朗读文本为空',
  sidecar_model_mismatch: '侧车模型不匹配',
  not_warmed: '侧车未预热',
  gpu_admission_denied: '显存安全余量不足，已停止本轮语音',
  tts_expansion_disabled: '语音扩展未开启',
  ref_audio_missing: '缺少参考音频',
}

/** User-facing hint for auto-TTS failures (stream or RPC). */
export function formatVoiceSpeakFailure(
  stage: VoiceSpeakStage,
  result: { reason?: string, message?: string },
): string {
  const reason = result.reason?.trim() || 'unknown'
  const detail = result.message?.trim()
  const label = REASON_LABELS[reason] || reason
  const prefix = stage === 'stream' ? '流式' : 'RPC'
  return `${prefix}朗读：${label}${detail ? `（${detail}）` : ''}`
}

/** Whether stream failure should attempt RPC fallback. */
export function shouldFallbackStreamToRpc(result: { ok?: boolean, reason?: string }): boolean {
  if (result.ok)
    return false
  const reason = result.reason?.trim()
  if (!reason)
    return true
  return ![
    'tts_expansion_disabled',
    'empty_text',
  ].includes(reason)
}
