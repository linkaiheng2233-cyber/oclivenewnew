import { directoryPluginInvoke, getPluginSettingsUi, setPluginSettingsConfig } from '@oclive/shared/api'
import { invokeWithFriendlyError } from '@oclive/shared/api/helpers'
import { hostEventBus } from '@oclive/shared/lib/hostEventBus'
import {
  VOICE_ASR_CONFIG_UPDATED_EVENT,
  VOICE_ASR_PLUGIN_ID,
} from '@oclive/shared/lib/voiceAsrEvents'
import { onBeforeUnmount, onMounted } from 'vue'
import { usePluginStore } from '@oclive/shared/stores/pluginStore'

type RoleVoiceProfile = {
  preferred_tts_profile?: string | null
  synth_profile?: string | null
  director_profile?: string | null
}

type ReadRoleProfileResult = {
  ok?: boolean
  profile?: RoleVoiceProfile | null
}

/** K-VOICE-04: apply role pack `voice_profile.json` TTS hints when switching roles. */
export function useRoleVoiceProfileSync(): void {
  const pluginStore = usePluginStore()

  async function applyVoiceProfileForRole(roleId: string): Promise<void> {
    if (pluginStore.isPluginDisabled(VOICE_ASR_PLUGIN_ID))
      return
    const rolePath = await invokeWithFriendlyError<string>('get_role_pack_path', { roleId })
    const raw = await directoryPluginInvoke<ReadRoleProfileResult>(
      VOICE_ASR_PLUGIN_ID,
      'voice.read_role_profile',
      { role_path: rolePath },
    )
    const profile = raw?.profile
    if (!profile?.preferred_tts_profile?.trim())
      return

    const ui = await getPluginSettingsUi(VOICE_ASR_PLUGIN_ID)
    const cfg = { ...(ui.config ?? {}) }
    const nextTts = profile.preferred_tts_profile.trim()
    if (cfg.tts_profile === nextTts)
      return

    cfg.tts_profile = nextTts
    if (profile.director_profile?.trim())
      cfg.director_profile = profile.director_profile.trim()

    await setPluginSettingsConfig(VOICE_ASR_PLUGIN_ID, cfg)
    await directoryPluginInvoke(VOICE_ASR_PLUGIN_ID, 'config_updated', { config: cfg })
    hostEventBus.emit(VOICE_ASR_CONFIG_UPDATED_EVENT, {})
  }

  function onRoleSwitched(payload: { roleId?: string }) {
    const roleId = payload?.roleId?.trim()
    if (!roleId)
      return
    void applyVoiceProfileForRole(roleId).catch(() => {
      /* optional side-channel; ignore when voice plugin offline */
    })
  }

  onMounted(() => {
    hostEventBus.on('role:switched', onRoleSwitched)
  })
  onBeforeUnmount(() => {
    hostEventBus.off('role:switched', onRoleSwitched)
  })
}
