import type { UserIdentityStateResponse } from '@oclive/shared/api'
import { computed, onMounted, ref, watch } from 'vue'
import {
  getUserIdentityState,
  OCLIVE_DEFAULT_IDENTITY_SENTINEL,
  setSceneUserIdentity,
  setUserIdentity,
} from '@oclive/shared/api'
import { useRoleStore } from '@oclive/shared/stores/roleStore'
import { useUiStore } from '@oclive/shared/stores/uiStore'

const identityState = ref<UserIdentityStateResponse | null>(null)
const loading = ref(false)
let watchersBound = false

async function refreshIdentityState(): Promise<void> {
  const roleStore = useRoleStore()
  const uiStore = useUiStore()
  const roleId = roleStore.currentRoleId
  if (!roleId) {
    identityState.value = null
    return
  }
  try {
    identityState.value = await getUserIdentityState(
      roleId,
      roleStore.roleInfo.identityBinding === 'per_scene' ? uiStore.sceneId : null,
    )
  }
  catch {
    identityState.value = null
  }
}

/** Shared User Identity catalog state (Settings SSOT + compact / StatusBar consumers). */
export function useUserIdentityState() {
  const roleStore = useRoleStore()
  const uiStore = useUiStore()

  if (!watchersBound) {
    watchersBound = true
    watch(
      () => [roleStore.currentRoleId, uiStore.sceneId, roleStore.roleInfo.identityBinding] as const,
      () => {
        void refreshIdentityState()
      },
    )
  }

  onMounted(() => {
    void refreshIdentityState()
  })

  const hasCatalog = computed(
    () => (identityState.value?.identities?.length ?? 0) > 0,
  )

  const useManifestDefault = computed(
    () => identityState.value?.use_manifest_default ?? true,
  )

  const currentIdentityLabel = computed(() => {
    const state = identityState.value
    if (!state?.identities?.length)
      return null
    if (state.use_manifest_default) {
      const defaultId = state.default_identity_id
      const row = state.identities.find(i => i.id === defaultId)
      return row?.display_name ?? defaultId ?? null
    }
    const current = state.identities.find(i => i.id === state.current_identity_id)
    return current?.display_name ?? state.current_identity_id ?? null
  })

  const identitySelectValue = computed(() => {
    if (!identityState.value)
      return ''
    if (identityState.value.use_manifest_default)
      return OCLIVE_DEFAULT_IDENTITY_SENTINEL
    return identityState.value.current_identity_id
  })

  async function setIdentity(nextId: string): Promise<UserIdentityStateResponse | null> {
    const roleId = roleStore.currentRoleId
    if (!roleId || nextId === identitySelectValue.value)
      return identityState.value
    loading.value = true
    try {
      const perScene = roleStore.roleInfo.identityBinding === 'per_scene'
      identityState.value = perScene
        ? await setSceneUserIdentity(roleId, uiStore.sceneId, nextId)
        : await setUserIdentity(roleId, nextId)
      await roleStore.refreshRoleInfo()
      return identityState.value
    }
    finally {
      loading.value = false
    }
  }

  return {
    identityState,
    loading,
    hasCatalog,
    useManifestDefault,
    currentIdentityLabel,
    identitySelectValue,
    refreshIdentityState,
    setIdentity,
  }
}
