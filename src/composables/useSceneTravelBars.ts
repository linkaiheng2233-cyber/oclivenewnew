import type { ComputedRef } from 'vue'
import type { JumpTimeResponse } from '../api'
import { computed, nextTick, ref } from 'vue'
import { useRoleStore } from '../stores/roleStore'
import { useUiStore } from '../stores/uiStore'

export interface UseSceneTravelBarsOptions {
  applySceneDestination: (sceneId: string, together: boolean) => Promise<void>
  sceneLabelForId: (sceneId: string) => string
}

export function useSceneTravelBars(opts: UseSceneTravelBarsOptions) {
  const roleStore = useRoleStore()
  const uiStore = useUiStore()

  const postReplySceneBarVisible = ref(false)
  const postReplySceneSelectedId = ref('')
  const togetherTravelBarVisible = ref(false)
  const togetherTravelSelectedId = ref('')
  const topBarSceneDialogVisible = ref(false)
  const pendingTopBarSceneId = ref('')
  const topBarSceneOpenerFocus = ref<HTMLElement | null>(null)
  const autonomousSceneNotice = ref<{
    visible: boolean
    fromLabel: string
    toLabel: string
  }>({ visible: false, fromLabel: '', toLabel: '' })

  const allSceneOptions: ComputedRef<Array<{ id: string, label: string }>> = computed(() => {
    if (!roleStore.interactionImmersive)
      return []
    const labels = roleStore.roleInfo.sceneLabels ?? []
    const scenes = roleStore.roleInfo.scenes ?? []
    if (labels.length > 0)
      return labels.map(s => ({ id: s.id, label: s.label }))
    return scenes.map(id => ({ id, label: id }))
  })

  const sceneDestinationOptions = computed(() => {
    const cur = uiStore.sceneId
    return allSceneOptions.value.filter(s => s.id !== cur)
  })

  function resetPureChatSceneUi(): void {
    postReplySceneBarVisible.value = false
    postReplySceneSelectedId.value = ''
    togetherTravelBarVisible.value = false
    togetherTravelSelectedId.value = ''
    topBarSceneDialogVisible.value = false
    pendingTopBarSceneId.value = ''
    autonomousSceneNotice.value = {
      visible: false,
      fromLabel: '',
      toLabel: '',
    }
  }

  function dismissPostReplySceneBar(): void {
    postReplySceneBarVisible.value = false
    postReplySceneSelectedId.value = ''
  }

  function dismissTogetherTravelBar(): void {
    togetherTravelBarVisible.value = false
    togetherTravelSelectedId.value = ''
  }

  async function confirmPostReplyScene(together: boolean): Promise<void> {
    if (!roleStore.interactionImmersive)
      return
    const id = postReplySceneSelectedId.value.trim()
    postReplySceneBarVisible.value = false
    postReplySceneSelectedId.value = ''
    await opts.applySceneDestination(id, together)
  }

  async function confirmTogetherTravel(together: boolean): Promise<void> {
    if (!roleStore.interactionImmersive)
      return
    const id = togetherTravelSelectedId.value.trim()
    togetherTravelBarVisible.value = false
    togetherTravelSelectedId.value = ''
    await opts.applySceneDestination(id, together)
  }

  function onTopBarSceneChange(ev: Event): void {
    if (!roleStore.interactionImmersive)
      return
    const sel = ev.target as HTMLSelectElement
    const next = sel.value
    if (next === uiStore.sceneId)
      return
    const a = document.activeElement
    topBarSceneOpenerFocus.value = a instanceof HTMLElement ? a : null
    pendingTopBarSceneId.value = next
    topBarSceneDialogVisible.value = true
    sel.value = uiStore.sceneId
  }

  function dismissTopBarSceneDialog(): void {
    topBarSceneDialogVisible.value = false
    pendingTopBarSceneId.value = ''
    const el = topBarSceneOpenerFocus.value
    topBarSceneOpenerFocus.value = null
    void nextTick(() => el?.focus({ preventScroll: true }))
  }

  async function confirmTopBarScene(together: boolean): Promise<void> {
    if (!roleStore.interactionImmersive)
      return
    const id = pendingTopBarSceneId.value.trim()
    topBarSceneDialogVisible.value = false
    pendingTopBarSceneId.value = ''
    const el = topBarSceneOpenerFocus.value
    topBarSceneOpenerFocus.value = null
    void nextTick(() => el?.focus({ preventScroll: true }))
    await opts.applySceneDestination(id, together)
  }

  function onPluginQuickActionTravel(payload: unknown): void {
    if (!roleStore.interactionImmersive)
      return
    const sceneId = (payload as { sceneId?: string } | null)?.sceneId
    const togetherRaw = (payload as { together?: boolean } | null)?.together
    const id = typeof sceneId === 'string' ? sceneId.trim() : ''
    if (!id)
      return
    if (!allSceneOptions.value.some(s => s.id === id))
      return
    const together = togetherRaw === true
    void opts.applySceneDestination(id, together)
  }

  function onVirtualTimeJumpComplete(res: JumpTimeResponse): void {
    if (!roleStore.interactionImmersive)
      return
    if (res.autonomous_scene_from && res.autonomous_scene_to) {
      autonomousSceneNotice.value = {
        visible: true,
        fromLabel: opts.sceneLabelForId(res.autonomous_scene_from),
        toLabel: opts.sceneLabelForId(res.autonomous_scene_to),
      }
    }
  }

  function dismissAutonomousSceneNotice(): void {
    autonomousSceneNotice.value = { visible: false, fromLabel: '', toLabel: '' }
  }

  function offerSceneBarsAfterReply(offerTogether: boolean, offerPicker: boolean): void {
    if (!roleStore.interactionImmersive)
      return
    if (offerTogether && sceneDestinationOptions.value.length > 0) {
      togetherTravelBarVisible.value = true
    }
    else if (offerPicker && sceneDestinationOptions.value.length > 0) {
      postReplySceneBarVisible.value = true
    }
  }

  function clearSceneBarsBeforeSend(): void {
    postReplySceneBarVisible.value = false
    postReplySceneSelectedId.value = ''
    togetherTravelBarVisible.value = false
    togetherTravelSelectedId.value = ''
  }

  return {
    allSceneOptions,
    sceneDestinationOptions,
    postReplySceneBarVisible,
    postReplySceneSelectedId,
    togetherTravelBarVisible,
    togetherTravelSelectedId,
    topBarSceneDialogVisible,
    pendingTopBarSceneId,
    autonomousSceneNotice,
    resetPureChatSceneUi,
    dismissPostReplySceneBar,
    dismissTogetherTravelBar,
    confirmPostReplyScene,
    confirmTogetherTravel,
    onTopBarSceneChange,
    dismissTopBarSceneDialog,
    confirmTopBarScene,
    onPluginQuickActionTravel,
    onVirtualTimeJumpComplete,
    dismissAutonomousSceneNotice,
    offerSceneBarsAfterReply,
    clearSceneBarsBeforeSend,
  }
}
