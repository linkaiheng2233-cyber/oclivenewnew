<script setup lang="ts">
const props = withDefaults(
  defineProps<{
    currentRoleId: string;
    currentRelation: string;
    roles: Array<{ id: string; name: string }>;
    relations: Array<{ id: string; name: string }>;
    loading: boolean;
    /** 与 oclive-new 顶栏一致：可只渲染角色或只渲染身份 */
    sections?: ("role" | "relation")[];
    variant?: "default" | "topbar";
  }>(),
  { sections: () => ["role", "relation"], variant: "default" },
);

const showRole = () => props.sections.includes("role");
const showRelation = () => props.sections.includes("relation");
const emit = defineEmits<{ changeRole: [string]; changeRelation: [string] }>();
</script>

<template>
  <section class="selector-row" :class="{ 'selector-row--topbar': variant === 'topbar' }">
    <template v-if="showRole()">
      <label class="label">🎭 角色</label>
      <select
        class="select"
        :value="currentRoleId"
        :disabled="loading"
        @change="emit('changeRole', ($event.target as HTMLSelectElement).value)"
      >
        <option v-for="r in roles" :key="r.id" :value="r.id">{{ r.name }}</option>
      </select>
    </template>
    <template v-if="showRelation()">
      <label class="label">👤 身份</label>
      <select
        class="select"
        :value="currentRelation"
        :disabled="loading"
        @change="emit('changeRelation', ($event.target as HTMLSelectElement).value)"
      >
        <option v-for="r in relations" :key="r.id" :value="r.id">{{ r.name }}</option>
      </select>
    </template>
  </section>
</template>

<style scoped>
/* 对齐 oclive-new #roleSelector */
.selector-row {
  margin: 0;
  padding: 8px 18px 14px;
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
  background: var(--bg-primary);
}
.label {
  color: var(--text-accent);
  font-size: 14px;
  font-weight: 500;
}
.select {
  border: none;
  border-radius: var(--radius-pill);
  padding: 6px 12px;
  color: var(--text-accent);
  background: var(--border-light);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  outline: none;
}
.selector-row--topbar {
  padding: 0;
  background: transparent;
  gap: 8px;
}
</style>
