<script setup lang="ts">
import { computed } from 'vue'
import Live2DStageAdapter from '../../components/visual/Live2DStageAdapter.vue'
import { useRoleStore } from '../../stores/roleStore'

const props = defineProps<{
  roleId: string
  roleName: string
}>()

const roleStore = useRoleStore()

const emotion = computed(() => roleStore.roleInfo.currentEmotion || 'neutral')
const visualStateId = computed(() => roleStore.roleInfo.visualStateId)
const performanceDirective = computed(() => {
  const path = roleStore.roleInfo.portraitAssetPath
  if (!path) return null
  return {
    kind: 'image',
    path,
    fallback_image: path,
  }
})
</script>

<template>
  <aside class="theater-stage" aria-label="theater visual stage">
    <Live2DStageAdapter
      :role-id="props.roleId"
      :name="props.roleName"
      :emotion="emotion"
      :visual-state-id="visualStateId"
      :performance-directive="performanceDirective"
    />
  </aside>
</template>

<style scoped>
.theater-stage {
  display: flex;
  justify-content: center;
  padding: 0.5rem 1rem 0;
  max-width: 280px;
  margin: 0 auto;
}
</style>
