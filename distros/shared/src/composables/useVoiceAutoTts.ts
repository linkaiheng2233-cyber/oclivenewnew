import type { AppToastFn } from '@oclive/shared/composables/useAppToast'

import { directoryPluginInvoke, getPluginSettingsUi } from '@oclive/shared/api'

import { invokeWithFriendlyError } from '@oclive/shared/api/helpers'

import { hostEventBus } from '@oclive/shared/lib/hostEventBus'

import {

  VOICE_ASR_CONFIG_UPDATED_EVENT,

  VOICE_ASR_PLUGIN_ID,

  VOICE_STREAM_SENTENCE_EVENT,

} from '@oclive/shared/lib/voiceAsrEvents'

import { onBeforeUnmount, onMounted } from 'vue'

import { usePluginStore } from '@oclive/shared/stores/pluginStore'



const DEFAULT_TTS_PROFILE = 'bundled-cosyvoice2-zh'



type MessageSentPayload = {

  reply?: string

  bot_emotion?: string

  role_id?: string

}



type StreamSentencePayload = {

  reply?: string

  bot_emotion?: string

  role_id?: string

  sentence?: string

  stream_id?: string

}



type VoiceRuntimeConfig = {

  tts_expansion_enabled: boolean

  auto_tts: boolean

  tts_profile: string

  director_profile: string

}



let cachedConfig: VoiceRuntimeConfig | null = null

let cachedConfigAt = 0

const CONFIG_TTL_MS = 2_000



/** Per stream_id: whether first sentence TTS already fired. */

const streamSentenceSpoken = new Set<string>()



export function invalidateVoiceRuntimeConfig(): void {

  cachedConfig = null

  cachedConfigAt = 0

}



async function loadVoiceRuntimeConfig(

  isPluginDisabled: (id: string) => boolean,

): Promise<VoiceRuntimeConfig | null> {

  if (isPluginDisabled(VOICE_ASR_PLUGIN_ID))

    return null

  const now = Date.now()

  if (cachedConfig && now - cachedConfigAt < CONFIG_TTL_MS)

    return cachedConfig

  try {

    const ui = await getPluginSettingsUi(VOICE_ASR_PLUGIN_ID)

    const cfg = ui.config ?? {}

    cachedConfig = {

      tts_expansion_enabled: cfg.tts_expansion_enabled === true,

      auto_tts: cfg.auto_tts === true,

      tts_profile:

        typeof cfg.tts_profile === 'string' && cfg.tts_profile.trim()

          ? cfg.tts_profile.trim()

          : DEFAULT_TTS_PROFILE,

      director_profile:

        typeof cfg.director_profile === 'string'

          ? cfg.director_profile.trim() || 'none'

          : 'rules-v1',

    }

    cachedConfigAt = now

    return cachedConfig

  }

  catch {

    return null

  }

}



async function resolveRolePackPath(roleId: string): Promise<string> {

  const rid = roleId.trim()

  if (!rid)

    return ''

  try {

    return (await invokeWithFriendlyError<string>('get_role_pack_path', {

      roleId: rid,

    })).trim()

  }

  catch {

    return ''

  }

}



/**

 * Host-side auto TTS for `com.oclive.voice.asr`.

 * Requires voice expansion enabled; reads config on each speak.

 */

export function useVoiceAutoTts(options: { showToast: AppToastFn }) {

  const pluginStore = usePluginStore()

  let audioEl: HTMLAudioElement | null = null

  let speaking = false



  async function speakText(

    text: string,

    payload: { bot_emotion?: string, role_id?: string },

    cfg: VoiceRuntimeConfig,

  ): Promise<void> {

    const reply = text.trim()

    if (!reply || speaking)

      return



    speaking = true

    try {

      let directive: Record<string, unknown> | undefined

      const director = cfg.director_profile

      if (director && director !== 'none') {

        const rolePath = payload.role_id

          ? await resolveRolePackPath(payload.role_id)

          : ''

        const built = (await directoryPluginInvoke(

          VOICE_ASR_PLUGIN_ID,

          'voice.build_directive',

          {

            profile: director,

            bot_emotion: payload.bot_emotion || 'neutral',

            role_path: rolePath,

          },

        )) as { ok?: boolean, directive?: Record<string, unknown> }

        if (built.ok && built.directive)

          directive = built.directive

      }



      const profile

        = (directive?.synth_profile as string | undefined) || cfg.tts_profile

      const res = (await directoryPluginInvoke(

        VOICE_ASR_PLUGIN_ID,

        'voice.speak',

        { text: reply, profile, directive },

      )) as {

        ok?: boolean

        audio_base64?: string

        audio_mime?: string

        reason?: string

        message?: string

      }



      if (!res.ok || !res.audio_base64) {

        const hint = res.reason || res.message || '合成无音频'

        if (res.reason === 'tts_expansion_disabled')

          return

        options.showToast('warning', `语音朗读：${hint}`)

        return

      }



      if (!audioEl)

        audioEl = new Audio()

      const mime = res.audio_mime || 'audio/wav'

      audioEl.src = `data:${mime};base64,${res.audio_base64}`

      await audioEl.play()

    }

    catch (err) {

      const msg = err instanceof Error ? err.message : String(err)

      if (/notallowed|play\(\)/i.test(msg)) {

        options.showToast(

          'info',

          '无法自动播放：请先点击聊天区域或麦克风后再发消息',

        )

      }

      else {

        options.showToast('warning', `语音朗读失败：${msg}`)

      }

    }

    finally {

      speaking = false

    }

  }



  async function speakReply(payload: MessageSentPayload): Promise<void> {

    const reply = payload.reply?.trim()

    if (!reply)

      return



    const cfg = await loadVoiceRuntimeConfig(id => pluginStore.isPluginDisabled(id))

    if (!cfg?.tts_expansion_enabled || !cfg.auto_tts)

      return



    await speakText(reply, payload, cfg)

  }



  async function onStreamSentence(payload: unknown): Promise<void> {

    const p = payload as StreamSentencePayload

    const sentence = p.sentence?.trim()

    const streamId = p.stream_id?.trim()

    if (!sentence || !streamId)

      return

    if (streamSentenceSpoken.has(streamId))

      return



    const cfg = await loadVoiceRuntimeConfig(id => pluginStore.isPluginDisabled(id))

    if (!cfg?.tts_expansion_enabled || !cfg.auto_tts)

      return



    streamSentenceSpoken.add(streamId)

    await speakText(sentence, p, cfg)

  }



  function onMessageSent(payload: unknown): void {

    const p = payload as MessageSentPayload & { stream_id?: string }

    if (p.stream_id && streamSentenceSpoken.has(p.stream_id))

      return

    void speakReply(p)

  }



  function onConfigUpdated(): void {

    invalidateVoiceRuntimeConfig()

    streamSentenceSpoken.clear()

  }



  onMounted(() => {

    hostEventBus.on('message:sent', onMessageSent)

    hostEventBus.on(VOICE_STREAM_SENTENCE_EVENT, onStreamSentence)

    hostEventBus.on(VOICE_ASR_CONFIG_UPDATED_EVENT, onConfigUpdated)

  })



  onBeforeUnmount(() => {

    hostEventBus.off('message:sent', onMessageSent)

    hostEventBus.off(VOICE_STREAM_SENTENCE_EVENT, onStreamSentence)

    hostEventBus.off(VOICE_ASR_CONFIG_UPDATED_EVENT, onConfigUpdated)

  })

}


