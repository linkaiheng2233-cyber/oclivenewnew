import { computed, onMounted, reactive, ref } from 'vue'
import { setRoleInteractionMode } from '../api'
import { useAppToast } from './useAppToast'
import { useDistroUxProfile } from './useDistroUxProfile'
import { useUserIdentityState } from './useUserIdentityState'
import { messageHintsUserIdentity } from '../utils/identitySurpriseTriggers'
import { useEngagementStore } from '../stores/engagementStore'
import { useRoleStore } from '../stores/roleStore'
const IMMERSIVE_INTRO_KEY = 'oclive_immersive_intro_seen'

export function useProgressiveDisclosure() {
  const roleStore = useRoleStore()
  const engagementStore = useEngagementStore()
  const { showToast } = useAppToast()
  const { immersiveUnlockHintAfterTurns, allowModeSwitch, ensureDistroUxProfileLoaded } = useDistroUxProfile()
  const { identityState, hasCatalog, setIdentity } = useUserIdentityState()

  const immersiveIntroVisible = ref(false)
  const identitySheetVisible = ref(false)

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

  const identitySurpriseUnlocked = computed(() => {
    const e = engagement.value
    return Boolean(e?.identitySurpriseSeen)
  })

  const showIdentityControls = computed(() => {
    if (roleStore.interactionImmersive)
      return true
    return identitySurpriseUnlocked.value
  })

  const identitySurpriseOptions = computed(() => {
    const rows = identityState.value?.identities ?? []
    return rows.slice(0, 3).map(r => ({
      id: r.id,
      name: r.display_name || r.id,
    }))
  })

  function recordTurn(userMessage: string): void {
    if (!roleId.value)
      return
    const count = engagementStore.recordSuccessfulTurn(roleId.value)
    maybeShowIdentitySurprise(count, userMessage)
  }

  function maybeShowIdentitySurprise(turnCount: number, userMessage: string): void {
    if (!roleId.value || !hasCatalog.value)
      return
    const e = engagementStore.roleState(roleId.value)
    if (e.identitySurpriseSeen)
      return
    const enoughTurns = turnCount >= 5
    const hinted = messageHintsUserIdentity(userMessage)
    if (!enoughTurns && !hinted)
      return
    if (identitySurpriseOptions.value.length < 2)
      return
    identitySheetVisible.value = true
  }

  async function tryStoryMode(): Promise<void> {
    if (!roleId.value)
      return
    try {
      const info = await setRoleInteractionMode(roleId.value, 'immersive')
      roleStore.applyRoleInfo(info)
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

  async function pickIdentity(id: string): Promise<void> {
    identitySheetVisible.value = false
    if (!roleId.value)
      return
    try {
      await setIdentity(id)
      engagementStore.markIdentitySurpriseSeen(roleId.value)
    }
    catch (err) {
      showToast('error', err instanceof Error ? err.message : String(err))
    }
  }

  function keepIdentity(): void {
    identitySheetVisible.value = false
    if (roleId.value)
      engagementStore.markIdentitySurpriseSeen(roleId.value)
  }

  // reactive() so refs/computeds stay tracked when accessed as `progressive.*` via provide/inject.
  return reactive({
    showImmersiveUnlockBanner,
    showIdentityControls,
    identitySurpriseOptions,
    identitySheetVisible,
    immersiveIntroVisible,
    recordTurn,
    tryStoryMode,
    dismissImmersiveHint,
    dismissImmersiveIntro,
    pickIdentity,
    keepIdentity,
  })
}
