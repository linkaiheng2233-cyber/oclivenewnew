<script setup lang="ts">
import { onMounted, watch } from "vue";
import DebugPanel from "../DebugPanel.vue";
import { useAppToast } from "../../composables/useAppToast";
import { useChatStore } from "../../stores/chatStore";
import { useDebugStore } from "../../stores/debugStore";
import { useRoleStore } from "../../stores/roleStore";

const props = defineProps<{
  active: boolean;
}>();

const emit = defineEmits<{
  imported: [roleId: string];
  reloadPolicy: [];
}>();

const chatStore = useChatStore();
const roleStore = useRoleStore();
const debugStore = useDebugStore();
const { showToast } = useAppToast();

async function refreshIfActive(): Promise<void> {
  if (!props.active) return;
  await debugStore.loadDebugData();
}

onMounted(() => {
  void refreshIfActive();
});

watch(
  () => [props.active, roleStore.currentRoleId] as const,
  () => {
    void refreshIfActive();
  },
);

function onNotify(p: { type: "success" | "error" | "info" | "warning"; message: string }): void {
  showToast(p.type, p.message);
}
</script>

<template>
  <div v-show="active" class="sde">
    <DebugPanel
      embedded
      :visible="active"
      :loading="chatStore.isLoading"
      :favorability="roleStore.roleInfo.favorability"
      :personality="roleStore.roleInfo.personality ?? []"
      :events="debugStore.events"
      :memories="debugStore.memories"
      @reload="emit('reloadPolicy')"
      @refresh="debugStore.loadDebugData"
      @close="() => {}"
      @notify="onNotify"
      @imported="emit('imported', $event)"
    />
  </div>
</template>

<style scoped>
.sde {
  min-height: 120px;
}
.sde :deep(.debug--embedded .title) {
  margin-bottom: 10px;
  padding-bottom: 8px;
}
</style>
