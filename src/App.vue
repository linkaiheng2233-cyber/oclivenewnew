<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import AutonomousSceneNotice from "./components/AutonomousSceneNotice.vue";
import CharacterInfo from "./components/CharacterInfo.vue";
import ChatInput from "./components/ChatInput.vue";
import ChatMessageList from "./components/ChatMessageList.vue";
import DebugPanel from "./components/DebugPanel.vue";
import RoleSelector from "./components/RoleSelector.vue";
import SceneTravelBars from "./components/SceneTravelBars.vue";
import TopBarSceneModeDialog from "./components/TopBarSceneModeDialog.vue";
import Toast from "./components/Toast.vue";
import VirtualTimeBar from "./components/VirtualTimeBar.vue";
import { useChatStore } from "./stores/chatStore";
import { useDebugStore } from "./stores/debugStore";
import { useRoleStore } from "./stores/roleStore";
import { useUiStore } from "./stores/uiStore";
import { buildRelationDropdownOptions } from "./utils/relationOptions";
import { useAppToast } from "./composables/useAppToast";
import { useNarrativeScene } from "./composables/useNarrativeScene";
import { useSceneDestination } from "./composables/useSceneDestination";
import {
  loadRole,
  OCLIVE_DEFAULT_RELATION_SENTINEL,
  setErrorReporter,
  setRoleInteractionMode,
  setUserRelation,
  type JumpTimeResponse,
} from "./utils/tauri-api";

const roleStore = useRoleStore();
const chatStore = useChatStore();
const debugStore = useDebugStore();
const uiStore = useUiStore();
const { toast, showToast } = useAppToast();
const { applyResolvedNarrativeScene } = useNarrativeScene();
const {
  sceneTransition,
  applySceneDestination,
  sceneLabelForId,
  characterSceneLabel,
} = useSceneDestination(showToast);

const chatListRef = ref<InstanceType<typeof ChatMessageList> | null>(null);
const roleSwitching = ref(false);

/** 人设回复结束后，若本句含位移意图且有多场景，显示目的地条 */
const postReplySceneBarVisible = ref(false);
const postReplySceneSelectedId = ref("");
/** 邀请同行语义：选目的地后同行或仅叙事 */
const togetherTravelBarVisible = ref(false);
const togetherTravelSelectedId = ref("");
/** 顶栏改场景：叙事独行 / 同行 */
const topBarSceneDialogVisible = ref(false);
const pendingTopBarSceneId = ref("");
/** 虚拟时间跳转触发 autonomous_scene 规则时，左下角系统提示 */
const autonomousSceneNotice = ref<{
  visible: boolean;
  fromLabel: string;
  toLabel: string;
}>({ visible: false, fromLabel: "", toLabel: "" });

/** 宽屏左右分栏；窄屏改为上下堆叠，立绘用 stack 布局更易读 */
const wideSplitLayout = ref(typeof window !== "undefined" && window.innerWidth > 720);
function refreshSplitLayout(): void {
  wideSplitLayout.value = typeof window !== "undefined" && window.innerWidth > 720;
}

const relationOptions = computed(() =>
  buildRelationDropdownOptions(
    roleStore.roleInfo.userRelations ?? [],
    roleStore.roleInfo.defaultRelation,
  ),
);

/** 顶栏：全部场景选项（展示名） */
const allSceneOptions = computed(() => {
  const labels = roleStore.roleInfo.sceneLabels ?? [];
  const scenes = roleStore.roleInfo.scenes ?? [];
  if (labels.length > 0) {
    return labels.map((s) => ({ id: s.id, label: s.label }));
  }
  return scenes.map((id) => ({ id, label: id }));
});

/** 除当前叙事场景外可切换的目的地（位移条） */
const sceneDestinationOptions = computed(() => {
  const cur = uiStore.sceneId;
  return allSceneOptions.value.filter((s) => s.id !== cur);
});

const messages = computed(() =>
  chatStore.messagesForRoleScene(roleStore.currentRoleId, uiStore.sceneId),
);
const sceneHistorySplitIndex = computed(() =>
  chatStore.sceneHistorySplitForRoleScene(roleStore.currentRoleId, uiStore.sceneId),
);
const roleName = computed(() => roleStore.roleInfo.name || "沐沐");
const emotion = computed(() => roleStore.roleInfo.currentEmotion || "neutral");

/** 对齐 oclive-new 底部状态栏心形 */
const statusHeart = computed(() => {
  const f = roleStore.roleInfo.favorability;
  if (f >= 60) return "💖";
  if (f >= 30) return "💕";
  return "🤍";
});

async function onInteractionModeChange(ev: Event) {
  const v = (ev.target as HTMLSelectElement).value as "immersive" | "pure_chat";
  try {
    const info = await setRoleInteractionMode(roleStore.currentRoleId, v);
    roleStore.applyRoleInfo(info);
    if (v === "pure_chat") {
      postReplySceneBarVisible.value = false;
      postReplySceneSelectedId.value = "";
      togetherTravelBarVisible.value = false;
      togetherTravelSelectedId.value = "";
      topBarSceneDialogVisible.value = false;
      pendingTopBarSceneId.value = "";
      autonomousSceneNotice.value = {
        visible: false,
        fromLabel: "",
        toLabel: "",
      };
    }
  } catch (err) {
    showToast("error", err instanceof Error ? err.message : String(err));
  }
}

async function initialize() {
  try {
    await roleStore.loadRoles();
    await loadRole(roleStore.currentRoleId);
    await roleStore.refreshRoleInfo();
    applyResolvedNarrativeScene();
    await debugStore.loadDebugData();
  } catch (err) {
    showToast("error", err instanceof Error ? err.message : String(err));
  }
}

async function onSend(payload: { content: string }) {
  postReplySceneBarVisible.value = false;
  postReplySceneSelectedId.value = "";
  togetherTravelBarVisible.value = false;
  togetherTravelSelectedId.value = "";
  const userText = payload.content;
  try {
    const res = await chatStore.sendMessage(userText, uiStore.sceneId);
    await roleStore.refreshRoleInfo();
    applyResolvedNarrativeScene();
    await debugStore.loadDebugData();
    if (res.reply_is_fallback) {
      showToast("info", "本次为备用回复（模型未返回正文时自动生成）");
    }
    const offerTogether = res.offer_together_travel ?? false;
    const offerPicker = res.offer_destination_picker ?? false;
    // 问卷：邀请同行条优先于「仅选目的地」条（与后端 movement_ui_flags 一致）
    if (offerTogether && sceneDestinationOptions.value.length > 0) {
      togetherTravelBarVisible.value = true;
    } else if (offerPicker && sceneDestinationOptions.value.length > 0) {
      postReplySceneBarVisible.value = true;
    }
  } catch (err) {
    showToast("error", err instanceof Error ? err.message : String(err));
  }
}

async function confirmPostReplyScene(together: boolean) {
  const id = postReplySceneSelectedId.value.trim();
  postReplySceneBarVisible.value = false;
  postReplySceneSelectedId.value = "";
  await applySceneDestination(id, together);
}

function dismissPostReplySceneBar() {
  postReplySceneBarVisible.value = false;
  postReplySceneSelectedId.value = "";
}

async function confirmTogetherTravel(together: boolean) {
  const id = togetherTravelSelectedId.value.trim();
  togetherTravelBarVisible.value = false;
  togetherTravelSelectedId.value = "";
  await applySceneDestination(id, together);
}

function dismissTogetherTravelBar() {
  togetherTravelBarVisible.value = false;
  togetherTravelSelectedId.value = "";
}

function onTopBarSceneChange(ev: Event) {
  const sel = ev.target as HTMLSelectElement;
  const next = sel.value;
  if (next === uiStore.sceneId) return;
  pendingTopBarSceneId.value = next;
  topBarSceneDialogVisible.value = true;
  sel.value = uiStore.sceneId;
}

function dismissTopBarSceneDialog() {
  topBarSceneDialogVisible.value = false;
  pendingTopBarSceneId.value = "";
}

async function confirmTopBarScene(together: boolean) {
  const id = pendingTopBarSceneId.value.trim();
  topBarSceneDialogVisible.value = false;
  pendingTopBarSceneId.value = "";
  await applySceneDestination(id, together);
}

async function onSwitchRole(nextRoleId: string) {
  try {
    roleSwitching.value = true;
    await roleStore.switchRole(nextRoleId);
    applyResolvedNarrativeScene();
    await debugStore.loadDebugData();
    showToast("success", `已切换角色: ${nextRoleId}`);
  } catch (err) {
    showToast("error", err instanceof Error ? err.message : String(err));
  } finally {
    window.setTimeout(() => {
      roleSwitching.value = false;
    }, 220);
  }
}

async function onChangeRelation(nextRelation: string) {
  try {
    const perScene = roleStore.roleInfo.identityBinding === "per_scene";
    if (nextRelation === OCLIVE_DEFAULT_RELATION_SENTINEL) {
      if (perScene) {
        await roleStore.setManifestDefaultIdentity(uiStore.sceneId);
      } else {
        await roleStore.setManifestDefaultIdentity();
      }
    } else if (perScene) {
      await roleStore.setSceneUserRelation(uiStore.sceneId, nextRelation);
    } else {
      const info = await setUserRelation(roleStore.currentRoleId, nextRelation);
      roleStore.applyRoleInfo(info);
    }
    const relationName =
      relationOptions.value.find((r) => r.id === nextRelation)?.name ?? nextRelation;
    const scopeLabel = perScene ? "当前场景身份" : "身份";
    showToast("success", `已设置${scopeLabel}：${relationName}`);
  } catch (err) {
    showToast("error", err instanceof Error ? err.message : String(err));
  }
}

async function onPackImported(roleId: string) {
  try {
    roleStore.currentRoleId = roleId;
    await loadRole(roleId);
    await roleStore.refreshRoleInfo();
    await roleStore.loadRoles();
    applyResolvedNarrativeScene();
    await debugStore.loadDebugData();
  } catch (err) {
    showToast("error", err instanceof Error ? err.message : String(err));
  }
}

function onVirtualTimeJumpComplete(res: JumpTimeResponse): void {
  if (res.autonomous_scene_from && res.autonomous_scene_to) {
    autonomousSceneNotice.value = {
      visible: true,
      fromLabel: sceneLabelForId(res.autonomous_scene_from),
      toLabel: sceneLabelForId(res.autonomous_scene_to),
    };
  }
}

function dismissAutonomousSceneNotice(): void {
  autonomousSceneNotice.value = { visible: false, fromLabel: "", toLabel: "" };
}

async function onReloadPolicy() {
  try {
    const msg = await debugStore.reloadPolicy();
    showToast("success", msg);
  } catch (err) {
    showToast("error", err instanceof Error ? err.message : String(err));
  }
}

function onHotkey(e: KeyboardEvent) {
  if (e.ctrlKey && e.shiftKey && e.key.toLowerCase() === "d") {
    e.preventDefault();
    debugStore.toggle();
  }
}

watch(
  messages,
  async () => {
    await nextTick();
    chatListRef.value?.scrollToBottom?.();
  },
  { flush: "post" },
);

watch(
  () => debugStore.visible,
  (v) => {
    if (v) void debugStore.loadDebugData();
  },
);

onMounted(() => {
  setErrorReporter((err) => {
    showToast("error", err.message);
  });
  window.addEventListener("keydown", onHotkey);
  window.addEventListener("resize", refreshSplitLayout);
  refreshSplitLayout();
  initialize();
});

onBeforeUnmount(() => {
  setErrorReporter(null);
  window.removeEventListener("keydown", onHotkey);
  window.removeEventListener("resize", refreshSplitLayout);
});
</script>

<template>
  <main class="layout">
    <div class="app-frame">
    <!-- 对齐 oclive-new：顶栏角色 + 时间/场景 -->
    <header class="top-bar">
      <RoleSelector
        variant="topbar"
        :sections="['role']"
        :current-role-id="roleStore.currentRoleId"
        :current-relation="roleStore.relationSelectValue"
        :roles="roleStore.roles"
        :relations="relationOptions"
        :loading="chatStore.isLoading"
        @change-role="onSwitchRole"
        @change-relation="onChangeRelation"
      />
      <div class="interaction-mode-wrap">
        <label class="interaction-mode-label" for="interaction-mode">模式</label>
        <select
          id="interaction-mode"
          class="interaction-mode-select"
          :value="roleStore.roleInfo.interactionMode"
          title="沉浸：虚拟时间、场景、日程与异地心声；纯聊：仅对话，隐藏上述能力"
          @change="onInteractionModeChange"
        >
          <option value="immersive">沉浸</option>
          <option value="pure_chat">纯聊</option>
        </select>
      </div>
      <div class="time-section">
        <template v-if="roleStore.interactionImmersive">
        <VirtualTimeBar
          compact
          :role-id="roleStore.currentRoleId"
          @notify="(p) => showToast(p.type, p.message)"
          @refreshed="roleStore.refreshRoleInfo"
          @jump-complete="onVirtualTimeJumpComplete"
        />
        <div v-if="allSceneOptions.length > 0" class="scene-row">
          <label class="scene-row-label" for="top-scene-select">叙事</label>
          <select
            id="top-scene-select"
            class="scene-select"
            :value="uiStore.sceneId"
            @change="onTopBarSceneChange($event)"
          >
            <option
              v-for="s in allSceneOptions"
              :key="s.id"
              :value="s.id"
            >
              {{ s.label }}
            </option>
          </select>
          <span class="scene-row-hint">角色在：{{ characterSceneLabel() }}</span>
        </div>
        </template>
        <RoleSelector
          variant="topbar"
          :sections="['relation']"
          :current-role-id="roleStore.currentRoleId"
          :current-relation="roleStore.relationSelectValue"
          :roles="roleStore.roles"
          :relations="relationOptions"
          :loading="chatStore.isLoading"
          @change-role="onSwitchRole"
          @change-relation="onChangeRelation"
        />
      </div>
    </header>

    <div
      v-if="roleStore.interactionImmersive && sceneTransition.visible"
      class="scene-transition-overlay"
      role="status"
      aria-live="polite"
    >
      正在前往「{{ sceneTransition.label }}」…
    </div>

    <TopBarSceneModeDialog
      v-if="roleStore.interactionImmersive"
      :visible="topBarSceneDialogVisible"
      :pending-scene-label="sceneLabelForId(pendingTopBarSceneId)"
      @confirm="confirmTopBarScene"
      @dismiss="dismissTopBarSceneDialog"
    />

    <div class="main-content">
      <div class="split-row" :class="{ 'split-row--narrow': !wideSplitLayout }">
        <aside class="left-pane">
          <CharacterInfo
            class="character-block"
            :layout="wideSplitLayout ? 'sidebar' : 'stack'"
            :role-id="roleStore.currentRoleId"
            :name="roleName"
            :emotion="emotion"
          />
          <div class="left-pane-status" aria-label="好感度">
            好感度 {{ Math.round(roleStore.roleInfo.favorability) }} {{ statusHeart }}
          </div>
          <div
            v-if="roleStore.interactionImmersive && roleStore.roleInfo.currentLife?.label"
            class="left-pane-life"
            aria-label="日程推断"
          >
            此刻：{{ roleStore.roleInfo.currentLife?.label }}
          </div>
          <AutonomousSceneNotice
            v-if="roleStore.interactionImmersive"
            :visible="autonomousSceneNotice.visible"
            :from-label="autonomousSceneNotice.fromLabel"
            :to-label="autonomousSceneNotice.toLabel"
            @dismiss="dismissAutonomousSceneNotice"
          />
        </aside>
        <div class="right-pane">
          <div class="chat-scroll-wrap chat-list">
            <transition name="fade">
              <ChatMessageList
                ref="chatListRef"
                :key="`${roleStore.currentRoleId}-${uiStore.sceneId}`"
                :messages="messages"
                :history-split-index="sceneHistorySplitIndex"
                :loading="chatStore.isLoading"
                :role-switching="roleSwitching"
              />
            </transition>
          </div>
          <section class="input-area">
            <SceneTravelBars
              v-if="roleStore.interactionImmersive"
              :together-visible="togetherTravelBarVisible"
              :post-reply-visible="postReplySceneBarVisible"
              :destination-options="sceneDestinationOptions"
              :together-selected-id="togetherTravelSelectedId"
              :post-reply-selected-id="postReplySceneSelectedId"
              @update:together-selected-id="togetherTravelSelectedId = $event"
              @update:post-reply-selected-id="postReplySceneSelectedId = $event"
              @confirm-together="confirmTogetherTravel"
              @dismiss-together="dismissTogetherTravelBar"
              @confirm-post-reply="confirmPostReplyScene"
              @dismiss-post-reply="dismissPostReplySceneBar"
            />
            <ChatInput :loading="chatStore.isLoading" @send="onSend" />
          </section>
        </div>
      </div>
    </div>

    <DebugPanel
      :visible="debugStore.visible"
      :loading="chatStore.isLoading"
      :favorability="roleStore.roleInfo.favorability"
      :personality="roleStore.roleInfo.personality ?? []"
      :events="debugStore.events"
      :memories="debugStore.memories"
      @reload="onReloadPolicy"
      @refresh="debugStore.loadDebugData"
      @close="debugStore.toggle"
      @notify="(p) => showToast(p.type, p.message)"
      @imported="onPackImported"
    />

    <Toast :show="toast.show" :type="toast.type" :message="toast.message" />
    </div>
  </main>
</template>

<style scoped>
/* 占满视口：宽度随窗口拉伸，避免两侧大块留白 */
.layout {
  flex: 1;
  min-height: 0;
  width: 100%;
  display: flex;
  justify-content: stretch;
  align-items: stretch;
  padding: 6px 8px;
  background: var(--bg-page);
  box-sizing: border-box;
  overflow: hidden;
}
/* 单卡外壳：圆角与阴影保留，横向铺满可用区域 */
.app-frame {
  width: 100%;
  max-width: 100%;
  height: 100%;
  max-height: 100%;
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: var(--bg-primary);
  border-radius: var(--radius-app);
  border: 1px solid var(--border-light);
  box-shadow: var(--shadow-app);
  overflow: hidden;
}
.top-bar {
  flex-shrink: 0;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-light);
  flex-wrap: wrap;
}
.interaction-mode-wrap {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}
.interaction-mode-label {
  font-size: 12px;
  color: var(--text-secondary);
  white-space: nowrap;
}
.interaction-mode-select {
  min-width: 88px;
  border: 1px solid var(--border-light);
  border-radius: var(--radius-btn);
  padding: 4px 8px;
  font-size: 12px;
  color: var(--text-primary);
  background: var(--bg-elevated);
}
.time-section {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
  font-size: 12px;
  color: var(--text-secondary);
}
.scene-row {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}
.scene-row-label {
  color: var(--text-secondary);
  font-weight: 600;
  white-space: nowrap;
}
.scene-select {
  min-width: 120px;
  max-width: 200px;
  border: 1px solid var(--border-light);
  border-radius: var(--radius-btn);
  padding: 4px 8px;
  font-size: 12px;
  color: var(--text-primary);
  background: var(--bg-elevated);
}
.scene-row-hint {
  font-size: 11px;
  opacity: 0.9;
  max-width: 140px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.main-content {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--bg-primary);
}
/* 左：立绘 + 好感；右：历史 + 输入（历史区域显著变宽） */
.split-row {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: row;
  align-items: stretch;
  overflow: hidden;
}
.split-row--narrow {
  flex-direction: column;
}
.left-pane {
  flex: 0 0 clamp(248px, 28vw, 360px);
  max-width: 40%;
  min-width: 220px;
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow-x: hidden;
  overflow-y: auto;
  border-right: 1px solid var(--border-light);
  background: var(--bg-secondary);
}
.split-row--narrow .left-pane {
  flex: 0 0 auto;
  width: 100%;
  max-width: none;
  min-width: 0;
  border-right: none;
  border-bottom: 1px solid var(--border-light);
  max-height: min(52vh, 520px);
}
.character-block {
  flex-shrink: 0;
}
.left-pane-status {
  flex-shrink: 0;
  margin-top: auto;
  padding: 10px 12px 14px;
  font-size: 12px;
  color: var(--text-secondary);
  text-align: center;
  border-top: 1px solid var(--border-light);
  background: var(--bg-status);
}
.left-pane-life {
  flex-shrink: 0;
  padding: 0 12px 12px;
  font-size: 12px;
  color: var(--text-secondary);
  text-align: center;
  line-height: 1.4;
}
.right-pane {
  flex: 1;
  min-width: 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--bg-primary);
}
/* 聊天记录仅在右侧栏滚动 */
.chat-scroll-wrap {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 12px 18px 16px;
  background: var(--bg-primary);
  -webkit-overflow-scrolling: touch;
}
.input-area {
  flex-shrink: 0;
  border-top: 1px solid var(--border-light);
  background: var(--bg-primary);
}
.fade-enter-active,
.fade-leave-active {
  transition: opacity 220ms ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0.82;
}
.scene-transition-overlay {
  flex-shrink: 0;
  width: 100%;
  padding: 10px 16px;
  text-align: center;
  font-size: 14px;
  color: var(--text-primary);
  background: color-mix(in srgb, var(--bg-secondary) 88%, transparent);
  border-bottom: 1px solid var(--border-light);
  box-shadow: var(--shadow-sm);
  z-index: 2;
}
</style>
