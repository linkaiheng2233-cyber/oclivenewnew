import { computed, ref, watch } from 'vue'
import { usePluginStore } from '../stores/pluginStore'
import { usePluginTraceStore } from '../stores/pluginTraceStore'

export function usePluginManagerWorkspace() {
  const pluginStore = usePluginStore()
  const traceStore = usePluginTraceStore()

  const batchMode = ref(false)
  const batchSelected = ref<Record<string, boolean>>({})
  const selectedWorkspacePluginId = ref('')

  const selectedWorkspacePlugin = computed(() =>
    pluginStore.catalog.find(c => c.id === selectedWorkspacePluginId.value) ?? null,
  )

  function selectWorkspacePlugin(id: string): void {
    selectedWorkspacePluginId.value = id
  }

  function focusAdjacentCatalog(delta: number): void {
    const ids = pluginStore.catalog.map(c => c.id)
    if (!ids.length)
      return
    const idx = ids.indexOf(selectedWorkspacePluginId.value)
    const next = ids[(idx + delta + ids.length) % ids.length]
    if (next)
      selectWorkspacePlugin(next)
  }

  function clearBatchSelection(): void {
    batchSelected.value = {}
  }

  watch(batchMode, (v) => {
    if (!v)
      clearBatchSelection()
  })

  watch(
    () => pluginStore.catalog.map(c => c.id).join('\n'),
    () => {
      const next: Record<string, boolean> = {}
      for (const p of pluginStore.catalog) {
        if (batchSelected.value[p.id])
          next[p.id] = true
      }
      batchSelected.value = next
      const ids = pluginStore.catalog.map(c => c.id)
      if (ids.length === 0) {
        selectedWorkspacePluginId.value = ''
        return
      }
      if (
        !selectedWorkspacePluginId.value
        || !ids.includes(selectedWorkspacePluginId.value)
      ) {
        selectedWorkspacePluginId.value = ids[0] ?? ''
      }
    },
    { immediate: true },
  )

  watch(
    () => traceStore.focusPluginId,
    (id) => {
      if (id && pluginStore.catalog.some(c => c.id === id)) {
        selectedWorkspacePluginId.value = id
        traceStore.clearFocusInstalledPlugin()
      }
    },
  )

  const batchSelectedCount = computed(
    () => Object.values(batchSelected.value).filter(Boolean).length,
  )
  const batchSelectedIds = computed(() =>
    Object.entries(batchSelected.value)
      .filter(([, v]) => v)
      .map(([k]) => k),
  )

  function setBatchSelected(id: string, v: boolean): void {
    batchSelected.value = { ...batchSelected.value, [id]: v }
  }

  return {
    batchMode,
    batchSelected,
    selectedWorkspacePluginId,
    selectedWorkspacePlugin,
    batchSelectedCount,
    batchSelectedIds,
    selectWorkspacePlugin,
    focusAdjacentCatalog,
    clearBatchSelection,
    setBatchSelected,
  }
}
