import type { TheaterSessionTurn, TheaterSpeaker } from './types'
import { sendMessage } from '../api/chat'
import { probeOllamaAvailable } from './useTheaterBeatPatch'
import { injectDirectorBeat } from './theaterDirectorClient'
import {
  buildImprovFallbackLine,
  buildKernelImprovUserMessage,
  buildOllamaImprovPrompts,
  improvSpeakerLabel,
} from './theaterImprovPrompts'

const OLLAMA_URL = import.meta.env.VITE_OCLIVE_OLLAMA_URL?.trim() || 'http://127.0.0.1:11434'
const IMPROV_MODEL = import.meta.env.VITE_OCLIVE_THEATER_PATCH_MODEL?.trim() || 'qwen2.5:7b'

function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI__' in window
}

export interface ImprovLineContext {
  sceneId: string
  sceneTitle: string
  roleId: string
  speaker: TheaterSpeaker
  roleLabel: string
  priorTurns: TheaterSessionTurn[]
  locale: 'zh' | 'en'
}

export async function generateImprovLine(
  ctx: ImprovLineContext,
  options?: { preferKernel?: boolean },
): Promise<{ text: string, source: 'ollama' | 'kernel' | 'director' | 'fallback' }> {
  const ollamaUp = await probeOllamaAvailable()
  if (ollamaUp && !options?.preferKernel) {
    const fromOllama = await generateViaOllama(ctx)
    if (fromOllama) {
      return { text: fromOllama, source: 'ollama' }
    }
  }

  if (isTauri()) {
    try {
      const fromKernel = await generateViaKernel(ctx)
      if (fromKernel) {
        return { text: fromKernel, source: 'kernel' }
      }
    }
    catch { /* degrade */ }
  }

  const lastUser = [...ctx.priorTurns].reverse().find(t => t.speaker === 'user')
  const directorBeat = await injectDirectorBeat({
    scene_id: ctx.sceneId,
    speaker: ctx.speaker,
    summary: lastUser?.text ?? ctx.sceneTitle,
  })
  if (directorBeat?.text) {
    return { text: directorBeat.text, source: 'director' }
  }

  if (!ollamaUp) {
    const retry = await generateViaOllama(ctx)
    if (retry) {
      return { text: retry, source: 'ollama' }
    }
  }

  return {
    text: buildImprovFallbackLine(ctx.locale, ctx.roleLabel),
    source: 'fallback',
  }
}

async function generateViaOllama(ctx: ImprovLineContext): Promise<string | null> {
  const history = ctx.priorTurns
    .slice(-8)
    .map((t) => {
      const who = improvSpeakerLabel(ctx.locale, t.speaker === 'user' ? 'user' : t.speaker === 'a' ? 'a' : 'b')
      return `${who}: ${t.text}`
    })
    .join('\n')

  const { system, user } = buildOllamaImprovPrompts(
    ctx.locale,
    ctx.sceneTitle,
    ctx.roleLabel,
    history,
  )

  try {
    const res = await fetch(`${OLLAMA_URL.replace(/\/+$/, '')}/api/chat`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      signal: AbortSignal.timeout(15000),
      body: JSON.stringify({
        model: IMPROV_MODEL,
        stream: false,
        messages: [
          { role: 'system', content: system },
          { role: 'user', content: user },
        ],
      }),
    })
    if (!res.ok) {
      return null
    }
    const data = await res.json() as { message?: { content?: string } }
    const content = data?.message?.content?.trim()
    return content || null
  }
  catch {
    return null
  }
}

async function generateViaKernel(ctx: ImprovLineContext): Promise<string | null> {
  const history = ctx.priorTurns
    .slice(-6)
    .map((t) => {
      const who = improvSpeakerLabel(ctx.locale, t.speaker === 'user' ? 'user' : t.speaker === 'a' ? 'a' : 'b')
      return `${who}: ${t.text}`
    })
    .join('\n')

  const userMessage = buildKernelImprovUserMessage(
    ctx.locale,
    ctx.sceneTitle,
    ctx.roleLabel,
    history,
  )

  const res = await sendMessage({
    role_id: ctx.roleId,
    user_message: userMessage,
    scene_id: ctx.sceneId,
  })
  const reply = res.reply?.trim()
  return reply || null
}
