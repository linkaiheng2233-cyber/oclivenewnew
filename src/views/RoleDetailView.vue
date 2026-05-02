<script setup lang="ts">
import CharacterInfo from "../components/CharacterInfo.vue";

defineProps<{
  roleId: string;
  name: string;
  emotion: string;
  layout: "stack" | "sidebar";
}>();
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
