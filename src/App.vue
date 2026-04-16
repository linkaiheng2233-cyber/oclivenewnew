<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import AutonomousSceneNotice from "./components/AutonomousSceneNotice.vue";
import HelpHint from "./components/HelpHint.vue";
import RoleDetailView from "./views/RoleDetailView.vue";
import ChatInput from "./components/ChatInput.vue";
import ChatPluginToolbarSlots from "./components/ChatPluginToolbarSlots.vue";
import PluginChatHeaderSlots from "./components/PluginChatHeaderSlots.vue";
import PluginSidebarSlots from "./components/PluginSidebarSlots.vue";
import PluginManagerPanel from "./views/PluginManagerPanel.vue";
import SettingsView from "./views/SettingsView.vue";
import ChatMessageList from "./components/ChatMessageList.vue";
import DebugPanel from "./components/DebugPanel.vue";
import RoleSelector from "./components/RoleSelector.vue";
import SceneTravelBars from "./components/SceneTravelBars.vue";
import TopBarSceneModeDialog from "./components/TopBarSceneModeDialog.vue";
import ShortcutHelp from "./components/ShortcutHelp.vue";
import Toast from "./components/Toast.vue";
import VirtualTimeBar from "./components/VirtualTimeBar.vue";
import { useChatStore } from "./stores/chatStore";
import { useDebugStore } from "./stores/debugStore";
import { useRoleStore } from "./stores/roleStore";
import { useUiStore } from "./stores/uiStore";
import { usePluginStore } from "./stores/pluginStore";
import { listen } from "@tauri-apps/api/event";
import { buildRelationDropdownOptions } from "./utils/relationOptions";
import { useAppToast } from "./composables/useAppToast";
import { useOcliveAppearance } from "./composables/useOcliveAppearance";
import { useNarrativeScene } from "./composables/useNarrativeScene";
import { useSceneDestination } from "./composables/useSceneDestination";
import { usePackUiTheme } from "./composables/useTheme";
import { hostEventBus } from "./lib/hostEventBus";
import {
  loadRole,
  OCLIVE_DEFAULT_RELATION_SENTINEL,
  setErrorReporter,
  setRoleInteractionMode,
  setUserRelation,
  type JumpTimeResponse,
} from "./utils/tauri-api";

const roleStore = useRoleStore();
usePackUiTheme();
const chatStore = useChatStore();
const debugStore = useDebugStore();
const uiStore = useUiStore();
const pluginStore = usePluginStore();
const { toast, showToast } = useAppToast();
const { themeCycleLabel, cycleTheme, bumpScale, scaleLabel } = useOcliveAppearance();
const { applyResolvedNarrativeScene } = useNarrativeScene();
const {
  sceneTransition,
  applySceneDestination,
  sceneLabelForId,
  characterSceneLabel,
} = useSceneDestination(showToast);

const chatListRef = ref<InstanceType<typeof ChatMessageList> | null>(null);
const roleSwitching = ref(false);

/** 角色回复结束后，若本句含位移意图且有多场景，显示目的地条 */
const postReplySceneBarVisible = ref(false);
const postReplySceneSelectedId = ref("");
/** 邀请同行语义：选目的地后同行或仅叙事 */
const togetherTravelBarVisible = ref(false);
const togetherTravelSelectedId = ref("");
/** 顶栏改场景：叙事独行 / 同行 */
const topBarSceneDialogVisible = ref(false);
const pendingTopBarSceneId = ref("");
const quickActionSendEvent = "com.oclive.mumu.quick-actions:send_phrase";
const quickActionTravelEvent = "com.oclive.mumu.quick-actions:travel";
/** 虚拟时间跳转触发 autonomous_scene 规则时，左下角系统提示 */
const autonomousSceneNotice = ref<{
  visible: boolean;
  fromLabel: string;
  toLabel: string;
}>({ visible: false, fromLabel: "", toLabel: "" });

const shortcutHelpOpen = ref(false);
let ctrlLongPressTimer: ReturnType<typeof setTimeout> | null = null;

function clearCtrlLongPressTimer(): void {
  if (ctrlLongPressTimer != null) {
    window.clearTimeout(ctrlLongPressTimer);
    ctrlLongPressTimer = null;
  }
}

function onCtrlHoldHintKeydown(e: KeyboardEvent): void {
  if (e.key !== "Control" || e.repeat) {
    return;
  }
  clearCtrlLongPressTimer();
  ctrlLongPressTimer = window.setTimeout(() => {
    ctrlLongPressTimer = null;
    shortcutHelpOpen.value = true;
  }, 1000);
}

function onCtrlHoldHintKeyup(e: KeyboardEvent): void {
  if (e.key === "Control") {
    clearCtrlLongPressTimer();
  }
}

/** 宽屏左右分栏；窄屏改为上下堆叠，立绘用 stack 布局更易读 */
const wideSplitLayout = ref(typeof window !== "undefined" && window.innerWidth > 720);
function refreshSplitLayout(): void {
  wideSplitLayout.value = typeof window !== "undefined" && window.innerWidth > 720;
}

let splitLayoutResizeRaf = 0;
function scheduleRefreshSplitLayout(): void {
  if (splitLayoutResizeRaf !== 0) return;
  splitLayoutResizeRaf = requestAnimationFrame(() => {
    splitLayoutResizeRaf = 0;
    refreshSplitLayout();
  });
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

const topMoreOpen = ref(false);
const settingsViewOpen = ref(false);
const topBarRef = ref<HTMLElement | null>(null);
let morePanelClickListenTimer: ReturnType<typeof setTimeout> | null = null;

function toggleTopMore(e: Event) {
  e.stopPropagation();
  topMoreOpen.value = !topMoreOpen.value;
}

function onDocumentClickCloseMore(e: MouseEvent) {
  if (!topMoreOpen.value) return;
  const el = topBarRef.value;
  if (el && !el.contains(e.target as Node)) topMoreOpen.value = false;
}
const sceneHistorySplitIndex = computed(() =>
  chatStore.sceneHistorySplitForRoleScene(roleStore.currentRoleId, uiStore.sceneId),
);

/** 角色包 `ui.json` → layout；空字段视为 left / bottom */
const packLayoutResolved = computed(() => {
  const l = roleStore.roleInfo.packUiConfig?.layout ?? {
    sidebar: "",
    chatInput: "",
  };
  const sidebar = l.sidebar === "right" ? "right" : "left";
  const chatInput = l.chatInput === "top" ? "top" : "bottom";
  return { sidebar, chatInput };
});
const sidebarRight = computed(() => packLayoutResolved.value.sidebar === "right");
const chatInputTop = computed(() => packLayoutResolved.value.chatInput === "top");
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
    await pluginStore.refresh();
    await roleStore.refreshRoleInfo();
    hostEventBus.emitBuiltin("role:switched", { roleId: roleStore.currentRoleId });
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

function onPluginQuickActionSend(payload: unknown): void {
  if (chatStore.isLoading) return;
  const text = (payload as { text?: string } | null)?.text;
  const content = typeof text === "string" ? text.trim() : "";
  if (!content) return;
  void onSend({ content });
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

function onPluginQuickActionTravel(payload: unknown): void {
  const sceneId = (payload as { sceneId?: string } | null)?.sceneId;
  const togetherRaw = (payload as { together?: boolean } | null)?.together;
  const id = typeof sceneId === "string" ? sceneId.trim() : "";
  if (!id) return;
  if (!allSceneOptions.value.some((s) => s.id === id)) return;
  const together = togetherRaw === true;
  void applySceneDestination(id, together);
}

async function onSwitchRole(nextRoleId: string) {
  try {
    roleSwitching.value = true;
    await roleStore.switchRole(nextRoleId);
    await pluginStore.syncDirectoryPluginBootstrap();
    hostEventBus.emitBuiltin("role:switched", { roleId: nextRoleId });
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
    await pluginStore.refresh();
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
  if (e.key === "Escape") {
    if (shortcutHelpOpen.value) {
      e.preventDefault();
      shortcutHelpOpen.value = false;
      return;
    }
    if (pluginStore.panelVisible) {
      e.preventDefault();
      pluginStore.closePanel();
      return;
    }
    if (settingsViewOpen.value) {
      e.preventDefault();
      settingsViewOpen.value = false;
      return;
    }
    if (topMoreOpen.value) {
      e.preventDefault();
      topMoreOpen.value = false;
      return;
    }
    if (debugStore.visible) {
      e.preventDefault();
      debugStore.toggle();
      return;
    }
  }
  if (e.ctrlKey && e.shiftKey && e.key.toLowerCase() === "f") {
    e.preventDefault();
    void pluginStore.openPanel();
    return;
  }
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

let unlistenPluginFs: (() => void) | undefined;

onMounted(() => {
  setErrorReporter((err) => {
    showToast("error", err.message);
  });
  hostEventBus.on(quickActionSendEvent, onPluginQuickActionSend);
  hostEventBus.on(quickActionTravelEvent, onPluginQuickActionTravel);
  window.addEventListener("keydown", onHotkey);
  window.addEventListener("keydown", onCtrlHoldHintKeydown);
  window.addEventListener("keyup", onCtrlHoldHintKeyup);
  window.addEventListener("resize", scheduleRefreshSplitLayout);
  refreshSplitLayout();
  initialize();
  void listen("plugin:changed", () => {
    void pluginStore.onPluginFilesChanged().then(() => {
      showToast("success", "检测到插件变更，已自动刷新");
    });
  }).then((u) => {
    unlistenPluginFs = u;
  });
});

watch(topMoreOpen, (open) => {
  if (morePanelClickListenTimer != null) {
    clearTimeout(morePanelClickListenTimer);
    morePanelClickListenTimer = null;
  }
  document.removeEventListener("click", onDocumentClickCloseMore);
  if (open) {
    nextTick(() => {
      morePanelClickListenTimer = setTimeout(() => {
        morePanelClickListenTimer = null;
        document.addEventListener("click", onDocumentClickCloseMore);
      }, 0);
    });
  }
});

onBeforeUnmount(() => {
  if (morePanelClickListenTimer != null) clearTimeout(morePanelClickListenTimer);
  document.removeEventListener("click", onDocumentClickCloseMore);
  if (splitLayoutResizeRaf !== 0) {
    cancelAnimationFrame(splitLayoutResizeRaf);
    splitLayoutResizeRaf = 0;
  }
  setErrorReporter(null);
  window.removeEventListener("keydown", onHotkey);
  hostEventBus.off(quickActionSendEvent, onPluginQuickActionSend);
  hostEventBus.off(quickActionTravelEvent, onPluginQuickActionTravel);
  window.removeEventListener("keydown", onCtrlHoldHintKeydown);
  window.removeEventListener("keyup", onCtrlHoldHintKeyup);
  window.removeEventListener("resize", scheduleRefreshSplitLayout);
  clearCtrlLongPressTimer();
  unlistenPluginFs?.();
});
</script>

<template>
  <main class="layout">
    <div class="app-frame">
    <!-- 对齐 oclive-new：顶栏角色 + 时间/场景 -->
    <header ref="topBarRef" class="top-bar">
      <div class="top-bar-row">
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
        <button
          type="button"
          class="shortcut-help-btn"
          title="快捷键说明"
          aria-label="快捷键说明"
          @click="shortcutHelpOpen = true"
        >
          ?
        </button>
        <button
          type="button"
          class="more-toggle"
          :aria-expanded="topMoreOpen"
          aria-controls="top-more-panel"
          @click="toggleTopMore"
        >
          {{ topMoreOpen ? "收起" : "更多" }}
        </button>
      </div>

      <div
        v-show="topMoreOpen"
        id="top-more-panel"
        class="top-more-panel"
        role="region"
        aria-label="更多功能"
        @click.stop
      >
        <div class="more-grid">
          <div class="more-tile more-tile--xs">
            <div class="more-tile-head">
              <span class="more-label">互动模式</span>
              <HelpHint
                :paragraphs="[
                  '沉浸：启用虚拟时间、叙事场景、日程推断与位移相关能力。',
                  '纯聊：只保留对话，隐藏场景与时间条，适合日常闲聊。',
                ]"
              />
            </div>
            <div class="more-tile-body">
              <select
                id="interaction-mode"
                class="interaction-mode-select more-select more-select--fill"
                :value="roleStore.roleInfo.interactionMode"
                @change="onInteractionModeChange"
              >
                <option value="immersive">沉浸</option>
                <option value="pure_chat">纯聊</option>
              </select>
            </div>
          </div>

          <div class="more-tile more-tile--sm">
            <div class="more-tile-head">
              <span class="more-label">身份</span>
              <HelpHint text="与角色相处时的关系身份（如朋友、恋人等），影响对话与关系数值；与包内「核心性格档案」不同，后者写在 core_personality.txt。" />
            </div>
            <div class="more-tile-body more-tile-body--selector">
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
          </div>

          <div class="more-tile more-tile--lg">
            <div class="more-tile-head">
              <span class="more-label">界面</span>
              <HelpHint
                :paragraphs="[
                  '字号 A− / A+ 与编写器、启动器使用同一套档位，会保存在本机。',
                  '主题为浅色 / 深色 / 跟随系统，亦会记住。',
                ]"
              />
            </div>
            <div class="more-tile-body">
              <div class="top-bar-appearance" role="toolbar" aria-label="外观与字号">
                <div class="appearance-scale" aria-label="界面大小">
                  <button
                    type="button"
                    class="appearance-icon-btn"
                    title="缩小"
                    aria-label="缩小界面"
                    @click="bumpScale(-1)"
                  >
                    A−
                  </button>
                  <span
                    class="appearance-scale-value"
                    :title="'相对默认字号：' + scaleLabel"
                  >{{ scaleLabel }}</span>
                  <button
                    type="button"
                    class="appearance-icon-btn"
                    title="放大"
                    aria-label="放大界面"
                    @click="bumpScale(1)"
                  >
                    A+
                  </button>
                </div>
                <button
                  type="button"
                  class="appearance-theme-btn"
                  :title="'主题：' + themeCycleLabel + '（点击切换）'"
                  @click="cycleTheme"
                >
                  {{
                    themeCycleLabel === "跟随系统"
                      ? "◐"
                      : themeCycleLabel === "深色"
                        ? "🌙"
                        : "☀️"
                  }}
                  {{ themeCycleLabel }}
                </button>
              </div>
            </div>
          </div>

          <div class="more-tile more-tile--action">
            <div class="more-tile-head">
              <span class="more-label">设置</span>
              <HelpHint text="应用内设置：外观与交互说明见「界面」瓦片；插件扩展页可嵌入目录插件 settings.panel 配置页。" />
            </div>
            <div class="more-tile-body">
              <button
                type="button"
                class="more-debug-btn more-debug-btn--fill"
                @click="settingsViewOpen = true"
              >
                打开设置
              </button>
            </div>
          </div>

          <div class="more-tile more-tile--action">
            <div class="more-tile-head">
              <span class="more-label">调试</span>
              <HelpHint
                text="开发者与排错用：好感、记忆、策略重载等。Ctrl+Shift+D 可开关调试窗；顶栏「更多」展开时按 Esc 先收起本栏。"
              />
            </div>
            <div class="more-tile-body">
              <button type="button" class="more-debug-btn more-debug-btn--fill" @click="debugStore.toggle">
                打开调试面板
              </button>
            </div>
          </div>

          <template v-if="roleStore.interactionImmersive">
            <div class="more-tile more-tile--third">
              <div class="more-tile-head more-tile-head--tight">
                <span class="more-label">虚拟时间</span>
                <HelpHint
                  :paragraphs="[
                    '故事内的时间，与真实时钟独立。点击时间可打开滚轮调整。',
                    '可用快捷按钮推进时间；部分角色包会在跳转后触发场景或独白。',
                  ]"
                />
              </div>
              <div class="more-tile-body more-tile-body--row">
                <VirtualTimeBar
                  compact
                  class="more-vtime"
                  :role-id="roleStore.currentRoleId"
                  @notify="(p) => showToast(p.type, p.message)"
                  @refreshed="roleStore.refreshRoleInfo"
                  @jump-complete="onVirtualTimeJumpComplete"
                />
              </div>
            </div>

            <div v-if="allSceneOptions.length > 0" class="more-tile more-tile--third">
              <div class="more-tile-head more-tile-head--tight">
                <span class="more-label">叙事场景</span>
                <HelpHint
                  text="你当前叙事的场景；与角色包中的场景配置一致。切换后可能触发历史记录折叠分界。"
                />
              </div>
              <div class="more-tile-body more-tile-body--scene more-tile-body--scene-inline">
                <select
                  id="top-scene-select"
                  class="scene-select more-select more-select--fill"
                  :value="uiStore.sceneId"
                  @change="onTopBarSceneChange($event)"
                >
                  <option v-for="s in allSceneOptions" :key="s.id" :value="s.id">
                    {{ s.label }}
                  </option>
                </select>
                <span class="scene-row-hint scene-row-hint--tile">角色在：{{ characterSceneLabel() }}</span>
              </div>
            </div>
          </template>
        </div>
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
      <div
        class="split-row"
        :class="{
          'split-row--narrow': !wideSplitLayout,
          'split-row--sidebar-right': sidebarRight,
        }"
      >
        <aside class="left-pane">
          <RoleDetailView
            class="character-block"
            :layout="wideSplitLayout ? 'sidebar' : 'stack'"
            :role-id="roleStore.currentRoleId"
            :name="roleName"
            :emotion="emotion"
            :bootstrap-epoch="pluginStore.bootstrapEpoch"
          />
          <PluginSidebarSlots :bootstrap-epoch="pluginStore.bootstrapEpoch" />
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
        <div class="right-pane" :class="{ 'right-pane--input-top': chatInputTop }">
          <PluginChatHeaderSlots :bootstrap-epoch="pluginStore.bootstrapEpoch" />
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
            <ChatPluginToolbarSlots :bootstrap-epoch="pluginStore.bootstrapEpoch" />
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
    <ShortcutHelp v-model="shortcutHelpOpen" />

    <PluginManagerPanel />

    <SettingsView :visible="settingsViewOpen" @close="settingsViewOpen = false" />
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
  background: var(--shell-page-bg);
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
  box-shadow: var(--shadow-app), var(--frame-inset-highlight);
  overflow: hidden;
}
.top-bar {
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  align-items: stretch;
  gap: 0;
  padding: 10px 14px 12px;
  background: color-mix(in srgb, var(--bg-secondary) 92%, var(--rail-accent-runtime-bg) 8%);
  border-bottom: 1px solid var(--border-light);
  border-left: 3px solid var(--rail-accent-runtime);
  box-shadow: 0 1px 0 color-mix(in srgb, var(--accent) 12%, transparent);
}
.top-bar-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}
.shortcut-help-btn {
  flex-shrink: 0;
  width: 32px;
  height: 32px;
  padding: 0;
  border-radius: var(--radius-btn);
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-secondary);
  font-size: 14px;
  font-weight: 700;
  font-family: var(--font-ui);
  line-height: 1;
  cursor: pointer;
  transition: var(--control-transition);
}
.shortcut-help-btn:hover {
  border-color: color-mix(in srgb, var(--border-light) 70%, var(--text-secondary) 30%);
  color: var(--text-accent);
}
.shortcut-help-btn:focus {
  outline: none;
}
.shortcut-help-btn:focus-visible {
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--focus-ring-color) 35%, transparent);
}
.more-toggle {
  flex-shrink: 0;
  padding: 6px 14px;
  border-radius: var(--radius-btn);
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-primary);
  font-size: 12px;
  font-weight: 600;
  font-family: var(--font-ui);
  cursor: pointer;
  transition: var(--control-transition);
}
.more-toggle:hover {
  border-color: color-mix(in srgb, var(--border-light) 70%, var(--text-secondary) 30%);
  color: var(--text-accent);
}
.more-toggle:focus {
  outline: none;
}
.more-toggle:focus-visible {
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--focus-ring-color) 35%, transparent);
}
.top-more-panel {
  margin-top: 10px;
  padding-top: 12px;
  border-top: 1px solid var(--border-light);
}
.top-more-panel .interaction-mode-select,
.top-more-panel .scene-select {
  font-size: 13px;
  padding: 6px 10px;
  line-height: 1.4;
}
.top-more-panel .appearance-icon-btn,
.top-more-panel .appearance-theme-btn {
  font-size: 13px;
  min-height: 30px;
}
.top-more-panel .more-debug-btn {
  font-size: 13px;
  padding: 8px 12px;
}
.more-grid {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-start;
  align-items: flex-start;
  align-content: flex-start;
  gap: 12px 16px;
}
.more-tile {
  box-sizing: border-box;
  min-width: 0;
  padding: 12px 14px;
  border-radius: var(--radius-btn);
  border: 1px solid var(--border-light);
  background: color-mix(in srgb, var(--bg-elevated) 72%, transparent);
  display: flex;
  flex-direction: column;
  gap: 10px;
  box-shadow: var(--shadow-sm);
}
/* 按功能自然占地：不强行 flex-grow 拉满整行，宽裕时右侧留白 */
.more-tile--xs {
  flex: 0 0 auto;
  width: min(12rem, 100%);
}
.more-tile--sm {
  flex: 0 0 auto;
  width: min(17rem, 100%);
}
.more-tile--lg {
  flex: 0 0 auto;
  width: min(22rem, 100%);
}
.more-tile--action {
  flex: 0 0 auto;
  width: min(13rem, 100%);
}
/* 虚拟时间、叙事场景：约一行三分之一宽，不拉满；窄屏仍单列满宽 */
.more-tile--third {
  flex: 0 0 calc((100% - 32px) / 3);
  width: calc((100% - 32px) / 3);
  max-width: calc((100% - 32px) / 3);
  min-width: 0;
  padding: 12px 14px;
  gap: 10px;
  box-sizing: border-box;
}
.more-tile-head--tight {
  justify-content: flex-start;
  align-items: center;
  flex-wrap: wrap;
  gap: 6px 8px;
}
.more-tile-head--tight .more-label {
  padding-top: 0;
}
@media (max-width: 560px) {
  .more-tile--xs,
  .more-tile--sm,
  .more-tile--lg,
  .more-tile--action {
    width: 100%;
  }
  .more-tile--third {
    flex: 1 1 100%;
    width: 100%;
    max-width: 100%;
  }
}
.more-tile-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 8px;
}
.more-label {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
  line-height: 1.45;
  padding-top: 2px;
}
.more-tile-body {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.more-tile-body--row {
  flex-direction: row;
  flex-wrap: wrap;
  align-items: center;
}
.more-tile-body--scene {
  display: grid;
  grid-template-columns: minmax(0, 1.2fr) minmax(0, 1fr);
  gap: 8px 12px;
  align-items: center;
}
.more-tile-body--scene-inline {
  display: flex;
  flex-direction: row;
  flex-wrap: wrap;
  align-items: flex-start;
  gap: 8px 12px;
}
.more-tile-body--scene-inline .more-select--fill,
.more-tile-body--scene-inline .scene-select {
  flex: 0 1 14rem;
  min-width: min(12rem, 100%);
  max-width: 100%;
}
@media (max-width: 520px) {
  .more-tile-body--scene {
    grid-template-columns: 1fr;
  }
}
.more-tile-body--selector :deep(.selector-row--topbar) {
  width: 100%;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
}
.more-tile-body--selector :deep(.select) {
  min-width: 0;
  flex: 1 1 8rem;
  max-width: 100%;
}
.more-select--fill {
  width: 100%;
  max-width: none;
  box-sizing: border-box;
}
.more-vtime {
  flex: 1 1 12rem;
  min-width: 0;
  width: 100%;
}
.scene-row-hint--tile {
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.5;
  min-width: min(12rem, 100%);
  flex: 1 1 12rem;
  max-width: 100%;
}
.more-tile--third :deep(.vtime--compact) {
  gap: 6px;
  flex-wrap: wrap;
}
.more-tile--third :deep(.vtime--compact .time-display) {
  max-width: 100%;
  padding: 5px 8px;
  font-size: 12px;
}
.more-tile--third :deep(.vtime--compact .label-icon) {
  font-size: 14px;
}
.more-debug-btn {
  padding: 8px 12px;
  border-radius: var(--radius-btn);
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-secondary);
  font-size: 12px;
  font-family: var(--font-ui);
  cursor: pointer;
  transition: var(--control-transition);
}
.more-debug-btn--fill {
  width: 100%;
  box-sizing: border-box;
}
.more-debug-btn:hover {
  color: var(--text-primary);
  border-color: var(--border-focus);
}
.top-bar-appearance {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
  width: 100%;
}
.top-more-panel .top-bar-appearance {
  margin-left: 0;
}
.appearance-scale {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 6px;
  border-radius: var(--radius-btn);
  border: 1px solid var(--border-light);
  background: color-mix(in srgb, var(--bg-elevated) 88%, transparent);
  box-shadow: var(--shadow-sm), var(--frame-inset-highlight);
}
.appearance-scale-value {
  min-width: 2.6rem;
  text-align: center;
  font-size: 11px;
  font-weight: 600;
  color: var(--text-secondary);
  font-variant-numeric: tabular-nums;
}
.appearance-icon-btn,
.appearance-theme-btn {
  padding: 4px 8px;
  min-height: 28px;
  border-radius: var(--radius-btn);
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-primary);
  cursor: pointer;
  font-size: 12px;
  font-weight: 500;
  font-family: var(--font-ui);
  transition: var(--control-transition);
}
.appearance-icon-btn:hover,
.appearance-theme-btn:hover {
  border-color: var(--accent);
  color: var(--text-accent);
}
.appearance-icon-btn:focus,
.appearance-theme-btn:focus {
  outline: none;
}
.appearance-icon-btn:focus-visible,
.appearance-theme-btn:focus-visible {
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--focus-ring-color) 35%, transparent);
}
.appearance-theme-btn {
  white-space: nowrap;
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
.interaction-mode-select:focus {
  outline: none;
}
.interaction-mode-select:focus-visible {
  border-color: var(--accent);
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--focus-ring-color) 35%, transparent);
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
.scene-select:focus {
  outline: none;
}
.scene-select:focus-visible {
  border-color: var(--accent);
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--focus-ring-color) 35%, transparent);
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
/* 宽屏：立绘在右；窄屏：对话在上、立绘在下 */
.split-row--sidebar-right:not(.split-row--narrow) {
  flex-direction: row-reverse;
}
.split-row--sidebar-right:not(.split-row--narrow) .left-pane {
  border-right: none;
  border-left: 1px solid var(--border-light);
  box-shadow: inset 1px 0 0 color-mix(in srgb, var(--border-light) 65%, transparent);
}
.split-row--sidebar-right.split-row--narrow {
  flex-direction: column-reverse;
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
  background: color-mix(in srgb, var(--bg-secondary) 96%, var(--accent) 4%);
  box-shadow: inset -1px 0 0 color-mix(in srgb, var(--border-light) 65%, transparent);
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
.right-pane--input-top {
  flex-direction: column-reverse;
}
/* 聊天记录仅在右侧栏滚动；底部多留空，避免气泡+阴影被输入区视觉上压住 */
.chat-scroll-wrap {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 12px 18px max(52px, calc(32px + env(safe-area-inset-bottom, 0px)));
  scroll-padding-bottom: 44px;
  background: var(--bg-primary);
  -webkit-overflow-scrolling: touch;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.04);
}
.input-area {
  flex-shrink: 0;
  position: relative;
  z-index: 1;
  border-top: 1px solid var(--border-light);
  background: var(--bg-primary);
  /* 略收阴影，减少「盖住最后一泡」的错觉 */
  box-shadow: 0 -2px 14px color-mix(in srgb, var(--text-primary) 8%, transparent);
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
