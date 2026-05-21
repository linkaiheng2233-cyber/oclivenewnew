<script setup lang="ts">
import { BaseEdge, getBezierPath } from "@vue-flow/core";
import { computed } from "vue";
import { BACKEND_COLORS, edgeDash, type BackendKind } from "../../lib/graphEditorTheme";

const props = defineProps({
  sourceX: { type: Number, required: true },
  sourceY: { type: Number, required: true },
  targetX: { type: Number, required: true },
  targetY: { type: Number, required: true },
  sourcePosition: { type: String, required: true },
  targetPosition: { type: String, required: true },
  data: { type: Object, default: () => ({}) },
  selected: { type: Boolean, default: false },
});

const path = computed(() => getBezierPath(props));

const kind = computed(() => (props.data?.kind as BackendKind) ?? "builtin");

const stroke = computed(() => BACKEND_COLORS[kind.value].stroke);

const dash = computed(() => {
  const d = edgeDash(kind.value);
  return d === "none" ? undefined : d;
});
</script>

<script lang="ts">
export default { inheritAttrs: false };
</script>

<template>
  <BaseEdge
    :path="path[0]"
    :style="{
      stroke: stroke,
      strokeWidth: selected ? 2.25 : 1.75,
      strokeDasharray: dash,
      opacity: 0.88,
    }"
    :marker-end="undefined"
  />
</template>
