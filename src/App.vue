<script setup lang="ts">
import { computed, defineAsyncComponent, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import HelpHint from "./components/HelpHint.vue";
import ChatComposer from "./components/ChatComposer.vue";
import HotkeyHost from "./components/HotkeyHost.vue";
import ChatMessageList from "./components/ChatMessageList.vue";
import RoleSelector from "./components/RoleSelector.vue";
import Toast from "./components/Toast.vue";
import { useChatStore } from "./stores/chatStore";
import { useDebugStore } from "./stores/debugStore";
import { useRoleStore } from "./stores/roleStore";
import { useUiStore } from "./stores/uiStore";
import { usePluginStore } from "./stores/pluginStore";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { buildRelationDropdownOptions } from "./utils/relationOptions";
import { useAppToast } from "./composables/useAppToast";
import { useOcliveAppearance } from "./composables/useOcliveAppearance";
import { useNarrativeScene } from "./composables/useNarrativeScene";
import { useSceneDestination } from "./composables/useSceneDestination";
import { usePackUiTheme } from "./composables/useTheme";
import { usePluginManagerWindow } from "./composables/usePluginManagerWindow";
import { hostEventBus } from "./lib/hostEventBus";
import { chordModifierKeyDown, isMacLikePlatform } from "./lib/shortcutDisplay";
import {
  cancelChatGeneration,
  consumePendingProtocolInstalls,
  importRolePack,
  installPluginFromGit,
  loadRole,
  OCLIVE_DEFAULT_RELATION_SENTINEL,
  peekRolePack,
  revealRolePackFolder,
  parseApiErrorCode,
  setErrorReporter,
  setRemoteLifeEnabled,
  setRoleInteractionMode,
  setUserRelation,
  toPureChatPlainErrorMessage,
  type JumpTimeResponse,
} from "./utils/tauri-api";

const AutonomousSceneNotice = defineAsyncComponent(() => import("./components/AutonomousSceneNotice.vue"));
const RoleDetailView = defineAsyncComponent(() => import("./views/RoleDetailView.vue"));
const PluginSidebarSlots = defineAsyncComponent(() => import("./components/PluginSidebarSlots.vue"));
const RoleplayAsidePanel = defineAsyncComponent(() => import("./components/RoleplayAsidePanel.vue"));
const PluginManagerPanel = defineAsyncComponent(() => import("./views/PluginManagerPanel.vue"));
const PluginManagerV2Panel = defineAsyncComponent(() => import("./views/PluginManagerV2Panel.vue"));
const LocalModelManagerPanel = defineAsyncComponent(() => import("./views/LocalModelManagerPanel.vue"));
const PureChatModelSheet = defineAsyncComponent(() => import("./views/PureChatModelSheet.vue"));
const PluginMarketPanel = defineAsyncComponent(() => import("./views/PluginMarketPanel.vue"));
const PluginMarketV2Panel = defineAsyncComponent(() => import("./views/PluginMarketV2Panel.vue"));
const SettingsView = defineAsyncComponent(() => import("./views/SettingsView.vue"));
const DebugPanel = defineAsyncComponent(() => import("./components/DebugPanel.vue"));
const SceneTravelBars = defineAsyncComponent(() => import("./components/SceneTravelBars.vue"));
const TopBarSceneModeDialog = defineAsyncComponent(() => import("./components/TopBarSceneModeDialog.vue"));
const ShortcutHelp = defineAsyncComponent(() => import("./components/ShortcutHelp.vue"));
const VirtualTimeBar = defineAsyncComponent(() => import("./components/VirtualTimeBar.vue"));
const RolePackBar = defineAsyncComponent(() => import("./components/RolePackBar.vue"));
const ImportProgressModal = defineAsyncComponent(() => import("./components/ImportProgressModal.vue"));

const roleStore = useRoleStore();
usePackUiTheme();
const chatStore = useChatStore();
const debugStore = useDebugStore();
const uiStore = useUiStore();
const pluginStore = usePluginStore();
const { toast, showToast } = useAppToast();
const { t } = useI18n();
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
const startupStatus = ref(String(t("app.startup.loadingRoleAndPlugins")));
const startupReady = ref(false);

/** 角色回复结束后，若本句含位移意图且有多场景，显示目的地条 */
const postReplySceneBarVisible = ref(false);
const postReplySceneSelectedId = ref("");
/** 邀请同行语义：选目的地后同行或仅叙事 */
const togetherTravelBarVisible = ref(false);
const togetherTravelSelectedId = ref("");
/** 顶栏改场景：叙事独行 / 同行 */
const topBarSceneDialogVisible = ref(false);
const pendingTopBarSceneId = ref("");
const quickActionTravelEvent = "com.oclive.mumu.quick-actions:travel";
const settingsSetRemoteLifeEvent = "com.oclive.mumu.settings-panel:set_remote_life";
const settingsSetInteractionModeEvent =
  "com.oclive.mumu.settings-panel:set_interaction_mode";
const settingsCycleThemeEvent = "com.oclive.mumu.settings-panel:cycle_theme";
const settingsResetLayoutEvent = "com.oclive.mumu.settings-panel:request_reset_layout";
const settingsResetLayoutResultEvent = "com.oclive.mumu.settings-panel:reset_layout_result";
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
  if (!roleStore.interactionImmersive) return;
  const modKey = isMacLikePlatform() ? "Meta" : "Control";
  if (e.key !== modKey || e.repeat) {
    return;
  }
  clearCtrlLongPressTimer();
  ctrlLongPressTimer = window.setTimeout(() => {
    ctrlLongPressTimer = null;
    if (!roleStore.interactionImmersive) return;
    shortcutHelpOpen.value = true;
  }, 1000);
}

function onCtrlHoldHintKeyup(e: KeyboardEvent): void {
  const modKey = isMacLikePlatform() ? "Meta" : "Control";
  if (e.key === modKey) {
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

/** 本场景最近一条助手消息拆出的旁白/内心（供左侧叙事区，与主气泡对白分离） */
const latestRoleplayAside = computed(() => {
  const list = messages.value;
  for (let i = list.length - 1; i >= 0; i--) {
    const m = list[i];
    if (m.role === "assistant") {
      const a = m.aside?.trim();
      if (a) return a;
    }
  }
  return "";
});

const topMoreOpen = ref(false);
const settingsViewOpen = ref(false);
const localModelManagerOpen = ref(false);
const pureChatModelSheetOpen = ref(false);

const {
  pluginManagerV2Open,
  pluginMarketV2Open,
  openPluginManagerPanel,
  openPluginMarketPanel,
  openPluginManagerV2Preview,
  pluginManagerMoreBtnLabel,
  settingsEntryMoreHelp,
} = usePluginManagerWindow({
  closeMoreMenu: () => {
    topMoreOpen.value = false;
  },
  closeSettingsView: () => {
    settingsViewOpen.value = false;
  },
});

const topBarRef = ref<HTMLElement | null>(null);
let morePanelClickListenTimer: ReturnType<typeof setTimeout> | null = null;

function toggleTopMore(e: Event) {
  e.stopPropagation();
  topMoreOpen.value = !topMoreOpen.value;
}

function openShortcutHelp(): void {
  if (!roleStore.interactionImmersive) return;
  shortcutHelpOpen.value = true;
  topMoreOpen.value = false;
}

function openSettingsView(): void {
  settingsViewOpen.value = true;
  topMoreOpen.value = false;
}

async function onRevealRolePackFolder(): Promise<void> {
  const rid = (roleStore.currentRoleId ?? "").trim();
  if (!rid) {
    showToast("error", String(t("app.topBar.tiles.settingsEntry.revealRolePackNoRole")));
    return;
  }
  try {
    await revealRolePackFolder(rid);
    topMoreOpen.value = false;
    showToast("success", String(t("app.topBar.tiles.settingsEntry.revealRolePackOk")));
  } catch (e) {
    showToast("error", e instanceof Error ? e.message : String(e));
  }
}

/** 切到纯聊时收起依赖沉浸/插件栈的浮层，避免与纯聊路径叠在一起 */
function closePanelsForPureChatMode(): void {
  shortcutHelpOpen.value = false;
  settingsViewOpen.value = false;
  localModelManagerOpen.value = false;
  pureChatModelSheetOpen.value = false;
  pluginManagerV2Open.value = false;
  pluginMarketV2Open.value = false;
  pluginStore.closePanel();
  pluginStore.closeMarketPanel();
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
const roleName = computed(() => roleStore.roleInfo.name || String(t("app.defaults.roleName")));
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
      closePanelsForPureChatMode();
    }
  } catch (err) {
    showToast("error", err instanceof Error ? err.message : String(err));
  }
}

async function onPluginSetRemoteLife(payload: unknown): Promise<void> {
  const enabledRaw = (payload as { enabled?: boolean } | null)?.enabled;
  if (typeof enabledRaw !== "boolean") return;
  try {
    const info = await setRemoteLifeEnabled(roleStore.currentRoleId, enabledRaw);
    roleStore.applyRoleInfo(info);
    showToast(
      "success",
      String(t(enabledRaw ? "app.toasts.remoteLifeEnabled" : "app.toasts.remoteLifeDisabled")),
    );
  } catch (err) {
    showToast("error", err instanceof Error ? err.message : String(err));
  }
}

async function onPluginSetInteractionMode(payload: unknown): Promise<void> {
  const mode = (payload as { mode?: string } | null)?.mode;
  if (mode !== "immersive" && mode !== "pure_chat") return;
  try {
    const info = await setRoleInteractionMode(roleStore.currentRoleId, mode);
    roleStore.applyRoleInfo(info);
    if (mode === "pure_chat") {
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
      closePanelsForPureChatMode();
    }
    showToast(
      "success",
      String(
        t("app.toasts.interactionModeSwitched", {
          mode: mode === "immersive" ? t("app.interactionMode.immersive") : t("app.interactionMode.pureChat"),
        }),
      ),
    );
  } catch (err) {
    showToast("error", err instanceof Error ? err.message : String(err));
  }
}

function onPluginCycleTheme(): void {
  cycleTheme();
}

async function onPluginResetLayout(): Promise<void> {
  try {
    await pluginStore.resetToRolePackDefault();
    const message = String(t("app.toasts.layoutResetOk"));
    hostEventBus.emit(settingsResetLayoutResultEvent, { ok: true, message });
    showToast("success", message);
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    hostEventBus.emit(settingsResetLayoutResultEvent, {
      ok: false,
      message: String(t("app.toasts.layoutResetFailed", { message })),
    });
    showToast("error", message);
  }
}

async function initialize() {
  try {
    startupStatus.value = String(t("app.startup.scanningRolePacks"));
    await roleStore.loadRoles();
    if (!roleStore.currentRoleId.trim()) {
      showToast(
        "error",
        String(t("app.startup.noRolesFound")),
      );
      return;
    }
    startupStatus.value = String(t("app.startup.loadingRoleData"));
    await loadRole(roleStore.currentRoleId);
    startupStatus.value = String(t("app.startup.initializingPlugins"));
    await pluginStore.refresh();
    hostEventBus.emitBuiltin("role:switched", { roleId: roleStore.currentRoleId });
    applyResolvedNarrativeScene();
    startupReady.value = true;
  } catch (err) {
    startupStatus.value = String(t("app.startup.failed"));
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
    if (!res) {
      return;
    }
    await roleStore.refreshRoleInfo();
    applyResolvedNarrativeScene();
    if (debugStore.visible) {
      await debugStore.loadDebugData();
    }
    if (res.reply_is_fallback) {
      showToast("info", String(t("app.toasts.fallbackReply")));
    }
    const offerTogether = res.offer_together_travel ?? false;
    const offerPicker = res.offer_destination_picker ?? false;
    // 纯聊不展示叙事位移条；沉浸下才响应后端位移问卷
    if (!roleStore.interactionPureChat) {
      // 问卷：邀请同行条优先于「仅选目的地」条（与后端 movement_ui_flags 一致）
      if (offerTogether && sceneDestinationOptions.value.length > 0) {
        togetherTravelBarVisible.value = true;
      } else if (offerPicker && sceneDestinationOptions.value.length > 0) {
        postReplySceneBarVisible.value = true;
      }
    }
  } catch (err) {
    if (parseApiErrorCode(err) === "CHAT_GENERATION_CANCELLED") {
      showToast("info", String(t("app.toasts.chatStopped")));
      return;
    }
    if (roleStore.interactionPureChat) {
      showToast("error", toPureChatPlainErrorMessage(err));
    } else {
      showToast("error", err instanceof Error ? err.message : String(err));
    }
  }
}

async function onClearStuckSending() {
  const hadLoading = chatStore.isLoading;
  try {
    await cancelChatGeneration();
  } catch {
    /* 无活动生成或非 Tauri 环境 */
  }
  chatStore.invalidateActiveSend();
  if (hadLoading) {
    chatStore.removeLastUserBubble(
      roleStore.currentRoleId,
      uiStore.sceneId || "default",
    );
  }
  showToast("info", String(t("app.toasts.waitCleared")));
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
  if (roleStore.interactionPureChat) return;
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
    if (debugStore.visible) {
      await debugStore.loadDebugData();
    }
    showToast("success", String(t("app.toasts.roleSwitched", { id: nextRoleId })));
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
    const scopeLabel = perScene ? t("app.toasts.identityScope.scene") : t("app.toasts.identityScope.global");
    showToast("success", String(t("app.toasts.identitySet", { scope: scopeLabel, name: relationName })));
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
    if (debugStore.visible) {
      await debugStore.loadDebugData();
    }
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
    if (pluginMarketV2Open.value) {
      e.preventDefault();
      pluginMarketV2Open.value = false;
      return;
    }
    if (pluginManagerV2Open.value) {
      e.preventDefault();
      pluginManagerV2Open.value = false;
      return;
    }
    if (shortcutHelpOpen.value) {
      e.preventDefault();
      shortcutHelpOpen.value = false;
      return;
    }
    if (pluginStore.marketPanelVisible) {
      e.preventDefault();
      pluginStore.closeMarketPanel();
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
  if (chordModifierKeyDown(e) && e.shiftKey && e.key.toLowerCase() === "d") {
    e.preventDefault();
    debugStore.toggle();
    return;
  }
  if (roleStore.interactionPureChat) {
    if (chordModifierKeyDown(e) && e.shiftKey) {
      const k = e.key.toLowerCase();
      if (k === "f" || k === "a" || k === "s") e.preventDefault();
    }
    return;
  }
  if (chordModifierKeyDown(e) && e.shiftKey && e.key.toLowerCase() === "f") {
    e.preventDefault();
    openPluginManagerPanel();
    return;
  }
  if (chordModifierKeyDown(e) && e.shiftKey && e.key.toLowerCase() === "a") {
    e.preventDefault();
    openPluginMarketPanel();
    return;
  }
  if (chordModifierKeyDown(e) && e.shiftKey && e.key.toLowerCase() === "s") {
    e.preventDefault();
    openSettingsView();
    return;
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
let unlistenProtocolInstall: (() => void) | undefined;
let unlistenFileDrop: UnlistenFn | undefined;

const dropImportOpen = ref(false);
const dropImportPercent = ref(0);
const dropImportMessage = ref("");

function pickFirstRolePackArchivePath(paths: string[]): string | null {
  for (const raw of paths) {
    const p = (raw ?? "").trim();
    if (!p) continue;
    const low = p.toLowerCase();
    if (low.endsWith(".ocpak") || low.endsWith(".zip")) {
      return p;
    }
  }
  return null;
}

async function withDropImportProgress<T>(fn: () => Promise<T>): Promise<T> {
  dropImportOpen.value = true;
  dropImportPercent.value = 0;
  dropImportMessage.value = String(t("rolePackBar.progress.preparing"));
  let unlistenPr: UnlistenFn | undefined;
  unlistenPr = await listen<{ percent: number; message: string }>("import_progress", (e) => {
    dropImportPercent.value = e.payload.percent;
    dropImportMessage.value = e.payload.message;
  });
  try {
    return await fn();
  } finally {
    unlistenPr?.();
    dropImportOpen.value = false;
  }
}

async function handleTauriFileDrop(paths: string[]): Promise<void> {
  const path = pickFirstRolePackArchivePath(paths);
  if (!path) {
    showToast("info", String(t("app.fileDrop.ignoredNonPack")));
    return;
  }
  try {
    const peek = await peekRolePack(path);
    const exists = roleStore.roles.some((r) => r.id === peek.id);
    if (exists) {
      const ok = window.confirm(
        String(
          t("app.fileDrop.confirmOverwrite", {
            id: peek.id,
            name: peek.name,
            version: peek.version,
          }),
        ),
      );
      if (!ok) return;
      const roleId = await withDropImportProgress(() => importRolePack(path, true));
      showToast("success", String(t("app.fileDrop.imported", { id: roleId })));
      const dlMsg = peek.creator_message_to_downloader?.trim();
      if (dlMsg) showToast("info", dlMsg);
      await onPackImported(roleId);
      return;
    }
    const roleId = await withDropImportProgress(() => importRolePack(path, false));
    showToast("success", String(t("app.fileDrop.imported", { id: roleId })));
    const dlMsgNew = peek.creator_message_to_downloader?.trim();
    if (dlMsgNew) showToast("info", dlMsgNew);
    await onPackImported(roleId);
  } catch (e) {
    const msg =
      roleStore.interactionPureChat && e instanceof Error
        ? toPureChatPlainErrorMessage(e)
        : e instanceof Error
          ? e.message
          : String(e);
    showToast("error", msg);
  }
}

async function runPendingProtocolInstallsFromQueue(): Promise<void> {
  try {
    const pending = await consumePendingProtocolInstalls();
    for (const p of pending) {
      const git = p.gitUrl?.trim();
      if (!git) continue;
      try {
        const r = await installPluginFromGit(git);
        showToast("success", String(t("app.toasts.pluginInstalledFromUrl", { id: r.installedPluginId })));
        await pluginStore.refresh();
        if (!roleStore.interactionPureChat) openPluginManagerPanel();
      } catch (e) {
        showToast("error", e instanceof Error ? e.message : String(e));
      }
    }
  } catch (e) {
    console.warn("consume_pending_protocol_installs", e);
  }
}

onMounted(() => {
  chatStore.clearStuckSendingState();
  setErrorReporter((err) => {
    showToast("error", err.message);
  });
  hostEventBus.on(quickActionTravelEvent, onPluginQuickActionTravel);
  hostEventBus.on(settingsSetRemoteLifeEvent, onPluginSetRemoteLife);
  hostEventBus.on(settingsSetInteractionModeEvent, onPluginSetInteractionMode);
  hostEventBus.on(settingsCycleThemeEvent, onPluginCycleTheme);
  hostEventBus.on(settingsResetLayoutEvent, onPluginResetLayout);
  window.addEventListener("keydown", onHotkey);
  window.addEventListener("keydown", onCtrlHoldHintKeydown);
  window.addEventListener("keyup", onCtrlHoldHintKeyup);
  window.addEventListener("resize", scheduleRefreshSplitLayout);
  refreshSplitLayout();
  initialize();
  void listen("plugin:changed", () => {
    void pluginStore.onPluginFilesChanged().then(() => {
      showToast("success", String(t("app.toasts.pluginsAutoRefreshed")));
    });
  }).then((u) => {
    unlistenPluginFs = u;
  });

  void listen("protocol:pending_install", () => {
    void runPendingProtocolInstallsFromQueue();
  }).then((u) => {
    unlistenProtocolInstall = u;
  });

  void runPendingProtocolInstallsFromQueue();

  void listen<string[]>("tauri://file-drop", (e) => {
    const payload = e.payload;
    const paths = Array.isArray(payload) ? payload : payload ? [String(payload)] : [];
    if (paths.length === 0) return;
    void handleTauriFileDrop(paths);
  }).then((u) => {
    unlistenFileDrop = u;
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
  hostEventBus.off(quickActionTravelEvent, onPluginQuickActionTravel);
  hostEventBus.off(settingsSetRemoteLifeEvent, onPluginSetRemoteLife);
  hostEventBus.off(settingsSetInteractionModeEvent, onPluginSetInteractionMode);
  hostEventBus.off(settingsCycleThemeEvent, onPluginCycleTheme);
  hostEventBus.off(settingsResetLayoutEvent, onPluginResetLayout);
  window.removeEventListener("keydown", onCtrlHoldHintKeydown);
  window.removeEventListener("keyup", onCtrlHoldHintKeyup);
  window.removeEventListener("resize", scheduleRefreshSplitLayout);
  clearCtrlLongPressTimer();
  unlistenPluginFs?.();
  unlistenProtocolInstall?.();
  unlistenFileDrop?.();
});
</script>

<template>
  <main class="layout">
    <div class="app-frame">
    <!-- 对齐 oclive-new：顶栏角色 + 时间/场景 -->
    <header ref="topBarRef" class="top-bar">
      <div v-if="!startupReady" class="startup-status" role="status" aria-live="polite">
        {{ startupStatus }}
      </div>
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
          class="more-toggle"
          :aria-expanded="topMoreOpen"
          aria-controls="top-more-panel"
          @click="toggleTopMore"
        >
          {{ topMoreOpen ? t("app.topBar.more.collapse") : t("app.topBar.more.open") }}
        </button>
      </div>

      <div
        v-show="topMoreOpen"
        id="top-more-panel"
        class="top-more-panel"
        role="region"
        :aria-label="String(t('app.topBar.more.regionLabel'))"
        @click.stop
      >
        <div class="more-grid">
          <div class="more-tile more-tile--xs">
            <div class="more-tile-head">
              <span class="more-label">{{ t("app.topBar.tiles.interactionMode.title") }}</span>
              <HelpHint
                :paragraphs="(t('app.topBar.tiles.interactionMode.hint') as any)"
              />
            </div>
            <div class="more-tile-body">
              <select
                id="interaction-mode"
                class="interaction-mode-select more-select more-select--fill"
                :value="roleStore.roleInfo.interactionMode"
                @change="onInteractionModeChange"
              >
                <option value="immersive">{{ t("app.topBar.tiles.interactionMode.immersive") }}</option>
                <option value="pure_chat">{{ t("app.topBar.tiles.interactionMode.pureChat") }}</option>
              </select>
            </div>
          </div>

          <div class="more-tile more-tile--sm">
            <div class="more-tile-head">
              <span class="more-label">{{ t("app.topBar.tiles.identity.title") }}</span>
              <HelpHint :text="String(t('app.topBar.tiles.identity.hint'))" />
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
              <span class="more-label">{{ t("app.topBar.tiles.appearance.title") }}</span>
              <HelpHint
                :paragraphs="(t('app.topBar.tiles.appearance.hint') as any)"
              />
            </div>
            <div class="more-tile-body">
              <div class="top-bar-appearance" role="toolbar" :aria-label="String(t('app.topBar.tiles.appearance.toolbarLabel'))">
                <div class="appearance-scale" :aria-label="String(t('app.topBar.tiles.appearance.scaleLabel'))">
                  <button
                    type="button"
                    class="appearance-icon-btn"
                    :title="String(t('app.topBar.tiles.appearance.shrink'))"
                    :aria-label="String(t('app.topBar.tiles.appearance.shrinkAria'))"
                    @click="bumpScale(-1)"
                  >
                    A−
                  </button>
                  <span
                    class="appearance-scale-value"
                    :title="String(t('app.topBar.tiles.appearance.relativeScaleTitle', { label: scaleLabel }))"
                  >{{ scaleLabel }}</span>
                  <button
                    type="button"
                    class="appearance-icon-btn"
                    :title="String(t('app.topBar.tiles.appearance.enlarge'))"
                    :aria-label="String(t('app.topBar.tiles.appearance.enlargeAria'))"
                    @click="bumpScale(1)"
                  >
                    A+
                  </button>
                </div>
                <button
                  type="button"
                  class="appearance-theme-btn"
                  :title="String(t('app.topBar.tiles.appearance.themeTitle', { label: themeCycleLabel }))"
                  @click="cycleTheme"
                >
                  {{
                    themeCycleLabel === String(t("app.topBar.tiles.appearance.themeSystem"))
                      ? "◐"
                      : themeCycleLabel === String(t("app.topBar.tiles.appearance.themeDark"))
                        ? "🌙"
                        : "☀️"
                  }}
                  {{ themeCycleLabel }}
                </button>
              </div>
            </div>
          </div>

          <div v-if="roleStore.interactionPureChat" class="more-tile more-tile--action">
            <div class="more-tile-head">
              <span class="more-label">{{ t("app.topBar.tiles.pureChatModels.title") }}</span>
              <HelpHint :text="String(t('app.topBar.tiles.pureChatModels.hint'))" />
            </div>
            <div class="more-tile-body">
              <button
                type="button"
                class="more-debug-btn more-debug-btn--fill"
                @click="
                  pureChatModelSheetOpen = true;
                  topMoreOpen = false;
                "
              >
                {{ t("app.topBar.tiles.pureChatModels.openSheet") }}
              </button>
            </div>
          </div>

          <div v-if="!roleStore.interactionPureChat" class="more-tile more-tile--action settings-entry-tile">
            <div class="more-tile-head">
              <span class="more-label">{{ t("app.topBar.tiles.settingsEntry.title") }}</span>
              <HelpHint :text="settingsEntryMoreHelp" />
            </div>
            <div class="more-tile-body settings-entry-actions" role="group" :aria-label="String(t('app.topBar.tiles.settingsEntry.groupLabel'))">
              <button type="button" class="more-debug-btn more-debug-btn--fill settings-entry-btn" @click="openShortcutHelp">
                {{ t("app.topBar.tiles.settingsEntry.shortcutHelp") }}
              </button>
              <button
                type="button"
                class="more-debug-btn more-debug-btn--fill settings-entry-btn settings-entry-btn--primary settings-gear-btn"
                @click="openSettingsView"
              >
                {{ t("app.topBar.tiles.settingsEntry.settings") }}
              </button>
              <button
                type="button"
                class="more-debug-btn more-debug-btn--fill settings-entry-btn"
                @click="
                  localModelManagerOpen = true;
                  topMoreOpen = false;
                "
              >
                {{ t("app.topBar.tiles.settingsEntry.localModels") }}
              </button>
              <button
                type="button"
                class="more-debug-btn more-debug-btn--fill settings-entry-btn"
                :title="String(t('app.topBar.tiles.settingsEntry.revealRolePackHint'))"
                @click="void onRevealRolePackFolder()"
              >
                {{ t("app.topBar.tiles.settingsEntry.revealRolePackFolder") }}
              </button>
              <button
                type="button"
                class="more-debug-btn more-debug-btn--fill settings-entry-btn"
                @click="openPluginManagerPanel"
              >
                {{ pluginManagerMoreBtnLabel }}
              </button>
              <button
                type="button"
                class="more-debug-btn more-debug-btn--fill settings-entry-btn"
                @click="openPluginMarketPanel"
              >
                {{ t("app.topBar.tiles.settingsEntry.pluginMarket") }}
              </button>
            </div>
          </div>

          <div v-if="!roleStore.interactionPureChat" class="more-tile more-tile--action">
            <div class="more-tile-head">
              <span class="more-label">{{ t("app.topBar.tiles.rolePackShare.title") }}</span>
              <HelpHint
                :paragraphs="(t('app.topBar.tiles.rolePackShare.hint') as any)"
              />
            </div>
            <div class="more-tile-body">
              <RolePackBar
                @notify="(p) => showToast(p.type, p.message)"
                @imported="onPackImported"
              />
            </div>
          </div>

          <div class="more-tile more-tile--action">
            <div class="more-tile-head">
              <span class="more-label">{{ t("app.topBar.tiles.debug.title") }}</span>
              <HelpHint
                :text="
                  String(
                    roleStore.interactionPureChat
                      ? t('app.topBar.tiles.debug.hintPureChat')
                      : t('app.topBar.tiles.debug.hint'),
                  )
                "
              />
            </div>
            <div class="more-tile-body">
              <button type="button" class="more-debug-btn more-debug-btn--fill" @click="debugStore.toggle">
                {{ t("app.topBar.tiles.debug.openPanel") }}
              </button>
            </div>
          </div>

          <template v-if="roleStore.interactionImmersive">
            <div class="more-tile more-tile--third">
              <div class="more-tile-head more-tile-head--tight">
                <span class="more-label">{{ t("app.topBar.tiles.virtualTime.title") }}</span>
                <HelpHint
                  :paragraphs="(t('app.topBar.tiles.virtualTime.hint') as any)"
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
                <span class="more-label">{{ t("app.topBar.tiles.narrativeScene.title") }}</span>
                <HelpHint
                  :text="String(t('app.topBar.tiles.narrativeScene.help'))"
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
                <span class="scene-row-hint scene-row-hint--tile">
                  {{ t("app.topBar.tiles.narrativeScene.characterAt") }}：{{ characterSceneLabel() }}
                </span>
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
      {{ t("app.sceneTravel.travelingTo", { label: sceneTransition.label }) }}
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
          />
          <RoleplayAsidePanel :text="latestRoleplayAside" />
          <PluginSidebarSlots
            v-if="pluginStore.hasSidebarEmbeds"
            :bootstrap-epoch="pluginStore.bootstrapEpoch"
          />
          <div class="left-pane-status" :aria-label="String(t('app.status.favorabilityAria'))">
            {{ t("app.status.favorabilityLabel") }} {{ Math.round(roleStore.roleInfo.favorability) }} {{ statusHeart }}
          </div>
          <div
            v-if="roleStore.interactionImmersive && roleStore.roleInfo.currentLife?.label"
            class="left-pane-life"
            :aria-label="String(t('app.status.lifeAria'))"
          >
            {{ t("app.status.lifeNow") }}：{{ roleStore.roleInfo.currentLife?.label }}
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
          <div
            v-if="roleStore.interactionPureChat && messages.length === 0"
            class="pure-chat-assist"
            role="region"
            :aria-label="String(t('app.pureChatAssist.aria'))"
          >
            <p class="pure-chat-assist-lead">{{ t("app.pureChatAssist.lead") }}</p>
            <div class="pure-chat-assist-actions">
              <button type="button" class="pure-chat-assist-btn pure-chat-assist-btn--primary" @click="debugStore.toggle">
                {{ t("app.pureChatAssist.openDebug") }}
              </button>
            </div>
          </div>
          <div class="chat-scroll-wrap chat-list">
            <transition name="fade">
              <ChatMessageList
                ref="chatListRef"
                :key="`${roleStore.currentRoleId}-${uiStore.sceneId}`"
                :messages="messages"
                :history-split-index="sceneHistorySplitIndex"
                :loading="chatStore.isLoading"
                :role-switching="roleSwitching"
                @clear-stuck-loading="onClearStuckSending"
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
            <ChatComposer
              :loading="chatStore.isLoading"
              @send="onSend"
              @open-settings="openSettingsView"
              @clear-stuck-loading="onClearStuckSending"
            />
          </section>
        </div>
      </div>
    </div>

    <DebugPanel
      v-if="debugStore.visible"
      :visible="true"
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
    <ShortcutHelp
      v-if="roleStore.interactionImmersive"
      v-model="shortcutHelpOpen"
      :bootstrap-epoch="pluginStore.bootstrapEpoch"
    />

    <PluginMarketPanel v-if="pluginStore.marketPanelVisible" />
    <PluginManagerPanel v-if="pluginStore.panelVisible" />
    <PluginManagerV2Panel
      v-if="pluginManagerV2Open"
      :visible="true"
      @close="pluginManagerV2Open = false"
      @open-v1="
        pluginManagerV2Open = false;
        void pluginStore.openPanel('plugins');
      "
    />
    <LocalModelManagerPanel
      v-if="localModelManagerOpen"
      :visible="true"
      @close="localModelManagerOpen = false"
    />
    <PureChatModelSheet
      :visible="pureChatModelSheetOpen"
      @close="pureChatModelSheetOpen = false"
      @open-settings="
        pureChatModelSheetOpen = false;
        openSettingsView();
      "
    />
    <ImportProgressModal
      v-if="dropImportOpen"
      :open="dropImportOpen"
      :percent="dropImportPercent"
      :message="dropImportMessage"
    />
    <PluginMarketV2Panel
      v-if="pluginMarketV2Open"
      :visible="true"
      @close="pluginMarketV2Open = false"
    />

    <SettingsView
      v-if="settingsViewOpen"
      :visible="settingsViewOpen"
      @close="settingsViewOpen = false"
      @open-plugin-v2="openPluginManagerV2Preview"
    />

    <HotkeyHost />
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
.startup-status {
  margin-bottom: 8px;
  padding: 6px 10px;
  border: 1px solid var(--border-light);
  border-radius: var(--radius-btn);
  background: color-mix(in srgb, var(--bg-elevated) 80%, transparent);
  font-size: 12px;
  color: var(--text-secondary);
}
.settings-entry-tile {
  min-width: min(24rem, 100%);
}
.settings-entry-actions {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 8px;
}
.settings-entry-btn {
  min-height: 34px;
  font-size: 12px;
  font-weight: 600;
}
.settings-entry-btn--primary {
  border-color: color-mix(in srgb, var(--accent) 48%, var(--border-light) 52%);
  color: var(--text-accent);
  background: color-mix(in srgb, var(--bg-elevated) 75%, var(--accent-soft) 25%);
}
.settings-gear-btn {
  justify-content: center;
}
@media (max-width: 680px) {
  .settings-entry-actions {
    grid-template-columns: 1fr;
  }
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
.pure-chat-assist {
  flex-shrink: 0;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 10px 14px;
  padding: 10px 14px;
  border-bottom: 1px solid var(--border-light);
  background: color-mix(in srgb, var(--bg-secondary) 92%, var(--accent) 8%);
}
.pure-chat-assist-lead {
  margin: 0;
  flex: 1 1 220px;
  font-size: 12px;
  line-height: 1.5;
  color: var(--text-secondary);
}
.pure-chat-assist-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  flex-shrink: 0;
}
.pure-chat-assist-btn {
  border-radius: 8px;
  padding: 6px 12px;
  font-size: 12px;
  border: 1px solid var(--border-light);
  background: transparent;
  color: inherit;
  cursor: pointer;
}
.pure-chat-assist-btn:hover {
  background: var(--bg-hover, rgba(255, 255, 255, 0.06));
}
.pure-chat-assist-btn--primary {
  background: color-mix(in srgb, var(--accent) 22%, transparent);
  border-color: color-mix(in srgb, var(--accent) 45%, var(--border-light));
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
