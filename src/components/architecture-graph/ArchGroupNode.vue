<script setup lang="ts">
import { computed, inject } from 'vue'
import { SLOT_TYPE_GROUP_COLORS } from '../../lib/slotRegistry'
import { archGraphActionsKey } from './archGraphContext'

const props = defineProps({
  selected: { type: Boolean, default: false },
  data: { type: Object, default: () => ({}) },
})

const actions = inject(archGraphActionsKey)

const groupId = computed(() => String(props.data?.groupId ?? ''))
const label = computed(() => String(props.data?.label ?? groupId.value))
const slotType = computed(() => String(props.data?.slotType ?? 'memory'))
const collapsed = computed(() => Boolean(props.data?.collapsed))
const accent = computed(
  () => SLOT_TYPE_GROUP_COLORS[slotType.value] ?? SLOT_TYPE_GROUP_COLORS.memory,
)

function onToggle() {
  actions?.onToggleGroupCollapse(groupId.value)
}
</script>

<template>
  <div
    class="agn-group"
    :class="{ 'agn-group--collapsed': collapsed, 'agn-group--selected': selected }"
    :style="{ '--agn-group-accent': accent }"
  >
    <button type="button" class="agn-group__header" @click.stop="onToggle">
      <span class="agn-group__chevron">{{ collapsed ? "▸" : "▾" }}</span>
      <span class="agn-group__title">{{ label }}</span>
      <span class="agn-group__type">{{ slotType }}</span>
    </button>
  </div>
</template>

<style scoped>
.agn-group {
  width: 100%;
  height: 100%;
  box-sizing: border-box;
  border: 2px solid var(--agn-group-accent, #6d9a7d);
  border-radius: 10px;
  background: color-mix(in srgb, var(--agn-group-accent) 8%, #252526);
  pointer-events: none;
}
.agn-group__header {
  pointer-events: auto;
  display: flex;
  align-items: center;
  gap: 6px;
  width: 100%;
  margin: 0;
  padding: 6px 10px;
  border: none;
  border-bottom: 1px solid color-mix(in srgb, var(--agn-group-accent) 40%, transparent);
  border-radius: 8px 8px 0 0;
  background: color-mix(in srgb, var(--agn-group-accent) 18%, #2d2d30);
  color: #d4d4d4;
  font-size: 12px;
  cursor: pointer;
  text-align: left;
}
.agn-group__chevron {
  opacity: 0.85;
  font-size: 11px;
}
.agn-group__title {
  font-weight: 600;
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.agn-group__type {
  font-size: 10px;
  opacity: 0.7;
  text-transform: uppercase;
}
.agn-group--collapsed {
  opacity: 0.92;
}
</style>
