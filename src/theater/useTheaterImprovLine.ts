import type { TheaterSessionTurn, TheaterSpeaker } from './types'
import { sendMessage } from '../api/chat'
import { probeOllamaAvailable } from './useTheaterBeatPatch'
import { injectDirectorBeat } from './theaterDirectorClient'

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
    text: ctx.locale === 'zh'
      ? `${ctx.roleLabel}：（本地模型不可用，请继续插话推动剧情）`
      : `${ctx.roleLabel}: (local model unavailable — keep improvising)`,
    source: 'fallback',
  }
}

async function generateViaOllama(ctx: ImprovLineContext): Promise<string | null> {
  const history = ctx.priorTurns
    .slice(-8)
    .map((t) => {
      const who = t.speaker === 'user'
        ? (ctx.locale === 'zh' ? '用户' : 'User')
        : t.speaker === 'a'
          ? (ctx.locale === 'zh' ? '角色A' : 'Role A')
          : (ctx.locale === 'zh' ? '角色B' : 'Role B')
      return `${who}: ${t.text}`
    })
    .join('\n')

  const system = ctx.locale === 'zh'
    ? `你是「${ctx.sceneTitle}」场景中的${ctx.roleLabel}。只输出一句对白，不要 markdown、不要解释、不要括号舞台说明超过半句。`
    : `You are ${ctx.roleLabel} in scene "${ctx.sceneTitle}". Reply with one dialogue line only; no markdown.`

  const user = ctx.locale === 'zh'
    ? `已有对白：\n${history || '（尚无）'}\n\n请以${ctx.roleLabel}身份接下一句话：`
    : `Dialogue so far:\n${history || '(none)'}\n\nReply as ${ctx.roleLabel}:`

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
      const who = t.speaker === 'user'
        ? (ctx.locale === 'zh' ? '用户' : 'User')
        : t.speaker === 'a'
          ? (ctx.locale === 'zh' ? '角色A' : 'Role A')
          : (ctx.locale === 'zh' ? '角色B' : 'Role B')
      return `${who}: ${t.text}`
    })
    .join('\n')

  const userMessage = ctx.locale === 'zh'
    ? `【剧场场景：${ctx.sceneTitle}】\n${history ? `已有对白：\n${history}\n` : ''}请以${ctx.roleLabel}身份接一句对白（仅一句，无 markdown）：`
    : `[Theater scene: ${ctx.sceneTitle}]\n${history ? `Dialogue:\n${history}\n` : ''}Reply as ${ctx.roleLabel} with one line only:`

  const res = await sendMessage({
    role_id: ctx.roleId,
    user_message: userMessage,
    scene_id: ctx.sceneId,
  })
  const reply = res.reply?.trim()
  return reply || null
}
