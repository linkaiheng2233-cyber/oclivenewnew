import { computed, onMounted, ref, watch } from 'vue'
import { getUserIdentityState } from '../api'
import { useRoleStore } from '../stores/roleStore'
import { useUiStore } from '../stores/uiStore'

/** Read-only display name for the active User Identity catalog entry (chat sidebar / header). */
export function useCurrentIdentityLabel() {
  const roleStore = useRoleStore()
  const uiStore = useUiStore()
  const identityState = ref<Awaited<ReturnType<typeof getUserIdentityState>> | null>(null)

  async function refresh(): Promise<void> {
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

  onMounted(() => {
    void refresh()
  })

  watch(
    () => [roleStore.currentRoleId, uiStore.sceneId, roleStore.roleInfo.identityBinding] as const,
    () => {
      void refresh()
    },
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

  const hasIdentityCatalog = computed(
    () => (identityState.value?.identities?.length ?? 0) > 0,
  )

  return {
    currentIdentityLabel,
    hasIdentityCatalog,
    refreshIdentityLabel: refresh,
  }
}
