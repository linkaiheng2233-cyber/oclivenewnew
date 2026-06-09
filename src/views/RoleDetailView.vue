<script setup lang="ts">
import CharacterInfo from '../components/role/CharacterInfo.vue'
import PluginRoleDetailSlots from '../components/PluginRoleDetailSlots.vue'
import { useRoleStore } from '../stores/roleStore'

const roleStore = useRoleStore()

defineProps<{
  roleId: string
  name: string
  emotion: string
  layout: 'stack' | 'sidebar'
  bootstrapEpoch: number
}>()
</script>

<template>
  <div class="role-detail-view">
    <CharacterInfo
      class="character-block"
      :layout="layout"
      :role-id="roleId"
      :name="name"
      :emotion="emotion"
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
