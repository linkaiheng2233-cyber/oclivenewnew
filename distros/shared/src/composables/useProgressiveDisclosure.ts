import { useAppToast } from '@oclive/shared/composables/useAppToast'
import { useDistroUxProfile } from '@oclive/shared/composables/useDistroUxProfile'
import { useInteractionModeSettings } from '@oclive/shared/composables/useInteractionModeSettings'
import { useEngagementStore } from '@oclive/shared/stores/engagementStore'
import { useRoleStore } from '@oclive/shared/stores/roleStore'
import { computed, onMounted, reactive, ref } from 'vue'

const IMMERSIVE_INTRO_KEY = 'oclive_immersive_intro_seen'

export function useProgressiveDisclosure() {
  const roleStore = useRoleStore()
  const engagementStore = useEngagementStore()
  const { showToast } = useAppToast()
  const { immersiveUnlockHintAfterTurns, allowModeSwitch, ensureDistroUxProfileLoaded } = useDistroUxProfile()
  const { setInteractionMode } = useInteractionModeSettings()

  const immersiveIntroVisible = ref(false)

  onMounted(() => {
    void ensureDistroUxProfileLoaded()
  })

  const roleId = computed(() => roleStore.currentRoleId)

  const engagement = computed(() =>
    roleId.value ? engagementStore.roleState(roleId.value) : null,
  )

  const showImmersiveUnlockBanner = computed(() => {
    if (roleStore.interactionImmersive)
      return false
    if (!allowModeSwitch.value)
      return false
    const e = engagement.value
    if (!e || e.immersiveHintDismissed)
      return false
    return e.turnCount >= immersiveUnlockHintAfterTurns.value
  })

  function recordTurn(): void {
    if (!roleId.value)
      return
    engagementStore.recordSuccessfulTurn(roleId.value)
  }

  async function tryStoryMode(): Promise<void> {
    // Discovery path from ImmersiveUnlockBanner; user-facing switch is InteractionModeBar + Settings.
    if (!roleId.value)
      return
    try {
      await setInteractionMode('immersive')
      engagementStore.dismissImmersiveHint(roleId.value)
      if (!localStorage.getItem(IMMERSIVE_INTRO_KEY)) {
        immersiveIntroVisible.value = true
        localStorage.setItem(IMMERSIVE_INTRO_KEY, '1')
      }
    }
    catch (err) {
      showToast('error', err instanceof Error ? err.message : String(err))
    }
  }

  function dismissImmersiveHint(): void {
    if (roleId.value)
      engagementStore.dismissImmersiveHint(roleId.value)
  }

  function dismissImmersiveIntro(): void {
    immersiveIntroVisible.value = false
  }

  return reactive({
    showImmersiveUnlockBanner,
    immersiveIntroVisible,
    recordTurn,
    tryStoryMode,
    dismissImmersiveHint,
    dismissImmersiveIntro,
  })
}
