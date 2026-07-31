<script setup lang="ts">
import PluginRoleDetailSlots from '@oclive/shared/components/PluginRoleDetailSlots.vue'
import CharacterInfo from '@oclive/shared/components/role/CharacterInfo.vue'
import { useRoleStore } from '@oclive/shared/stores/roleStore'

defineProps<{
  roleId: string
  name: string
  emotion: string
  portraitAssetRelPath?: string | null
  layout: 'stack' | 'sidebar'
  bootstrapEpoch: number
}>()

const roleStore = useRoleStore()
</script>

<template>
  <div class="role-detail-view">
    <CharacterInfo
      class="character-block"
      :layout="layout"
      :role-id="roleId"
      :name="name"
      :emotion="emotion"
      :portrait-asset-rel-path="portraitAssetRelPath"
    />
    <PluginRoleDetailSlots
      v-if="roleStore.interactionImmersive"
      :bootstrap-epoch="bootstrapEpoch"
    />
  </div>
</template>

<style scoped>
.role-detail-view {
  display: flex;
  flex-direction: column;
  align-items: stretch;
  width: 100%;
  min-height: 0;
}
</style>
