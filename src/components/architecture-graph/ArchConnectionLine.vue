<script setup lang="ts">
import { BaseEdge, getBezierPath } from "@vue-flow/core";
import { computed } from "vue";
import { BACKEND_COLORS } from "../../lib/graphEditorTheme";

const props = defineProps({
  sourceX: { type: Number, required: true },
  sourceY: { type: Number, required: true },
  targetX: { type: Number, required: true },
  targetY: { type: Number, required: true },
  sourcePosition: { type: String, required: true },
  targetPosition: { type: String, required: true },
  connectionStatus: { type: String, default: null },
});

const path = computed(() => getBezierPath(props));

const stroke = computed(() => {
  if (props.connectionStatus === "invalid") return "#c45c5c";
  return BACKEND_COLORS.builtin.stroke;
});
</script>

<script lang="ts">
export default { inheritAttrs: false };
</script>

<template>
  <BaseEdge
    :path="path[0]"
    :style="{
      stroke,
      strokeWidth: 2,
      opacity: connectionStatus === 'invalid' ? 0.85 : 0.95,
    }"
  />
</template>
