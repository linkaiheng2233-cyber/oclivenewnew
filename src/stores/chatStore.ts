import { defineStore } from "pinia";
import {
  sendMessage,
  type PresenceMode,
  type SendMessageResponse,
} from "../utils/tauri-api";
import { presentationFromSendResponse } from "../utils/replyPresentation";
import { getRelationUpgradeMessage } from "../utils/relation";
import { useDebugStore } from "./debugStore";
import { useRoleStore } from "./roleStore";
import { useUiStore } from "./uiStore";
import { hostEventBus } from "../lib/hostEventBus";

export type ChatMessage = {
  id: string;
  role: "user" | "assistant" | "system";
  content: string;
  timestamp: number;
  /** assistant：本回合 bot 情绪（小写）；user 通常不传 */
  emotion?: string;
  /** assistant：异地模式（用于样式） */
  presenceVariant?: PresenceMode;
  /** 主 LLM 失败时的备用短回复（与后端 `reply_is_fallback` 一致） */
  replyIsFallback?: boolean;
};

type RoleSceneMessageMap = Record<string, Record<string, ChatMessage[]>>;

/** 与后端短期对话 FIFO 策略对齐（每角色最多保留条数） */
const MAX_MESSAGES_PER_CONVERSATION = 500;

/** 进入某场景时，该桶内已有消息条数；索引小于该值的视为「历史」折叠区（按角色×场景） */
export type SceneHistorySplitIndex = Record<string, Record<string, number>>;

export const useChatStore = defineStore(
  "chat",
  {
    state: () => ({
      messageMap: {} as RoleSceneMessageMap,
      isLoading: false,
      sceneHistorySplitIndex: {} as SceneHistorySplitIndex,
    }),
    getters: {
      /**
       * 指定角色×场景的消息列表（不读其它 store；调用方传入 currentRoleId / sceneId）。
       * 旧版 messageMap[roleId] 为数组时只读返回；写入路径（addMessage / clearMessages 等）会迁入分桶结构。
       */
      messagesForRoleScene: (state) => {
        return (roleId: string, sceneId: string): ChatMessage[] => {
          const sid = sceneId || "default";
          const roleBucket = (state.messageMap as unknown as Record<
            string,
            unknown
          >)[roleId];
          if (Array.isArray(roleBucket)) {
            return roleBucket as ChatMessage[];
          }
          const sceneBucket = (roleBucket as Record<string, ChatMessage[]> | undefined)?.[
            sid
          ];
          return sceneBucket ?? [];
        };
      },
      /** 指定角色×场景下「本次进入场景前」已有消息条数（用于折叠历史） */
      sceneHistorySplitForRoleScene: (state) => {
        return (roleId: string, sceneId: string): number => {
          const sid = sceneId || "default";
          return state.sceneHistorySplitIndex[roleId]?.[sid] ?? 0;
        };
      },
    },
    actions: {
      /** 将旧版 messageMap[roleId] 为数组的结构迁入当前 sceneId 桶（与 messages getter 一致） */
      ensureLegacyMigrated(roleId: string) {
        const uiStore = useUiStore();
        const roleBucket = (this.messageMap as unknown as Record<string, unknown>)[
          roleId
        ];
        if (Array.isArray(roleBucket)) {
          const legacy = roleBucket as ChatMessage[];
          (this.messageMap as unknown as Record<string, Record<string, ChatMessage[]>>)[
            roleId
          ] = { [uiStore.sceneId || "default"]: legacy };
        }
      },

      getMessageCountForRoleScene(roleId: string, sceneId: string): number {
        this.ensureLegacyMigrated(roleId);
        const sid = sceneId || "default";
        const roleMap = (this.messageMap as RoleSceneMessageMap)[roleId];
        return roleMap?.[sid]?.length ?? 0;
      },

      /**
       * 统一改场景入口：更新 uiStore.sceneId，并在跨场景时记录历史折叠分割点。
       * 初始化/换角/导入等同步场景请传 skipHistorySplit，避免误折叠。
       */
      applySceneChange(
        nextSceneId: string,
        options?: { skipHistorySplit?: boolean },
      ) {
        const uiStore = useUiStore();
        const roleStore = useRoleStore();
        const prev = uiStore.sceneId;
        const next = nextSceneId || "default";
        if (prev !== next && !options?.skipHistorySplit) {
          const roleId = roleStore.currentRoleId;
          this.ensureLegacyMigrated(roleId);
          if (!this.sceneHistorySplitIndex[roleId]) {
            this.sceneHistorySplitIndex[roleId] = {};
          }
          const count = this.getMessageCountForRoleScene(roleId, next);
          this.sceneHistorySplitIndex[roleId][next] = count;
        }
        uiStore.setScene(next);
      },

      /** 系统消息（如关系升级提示、场景欢迎语等） */
      addSystemMessage(content: string, sceneId?: string) {
        const roleStore = useRoleStore();
        const uiStore = useUiStore();
        const sid = sceneId ?? uiStore.sceneId ?? "default";
        const ts = Date.now();
        const message: ChatMessage = {
          id: `sys-${ts}-${Math.random().toString(36).slice(2, 9)}`,
          role: "system",
          content,
          timestamp: ts,
        };
        this.addMessage(roleStore.currentRoleId, sid, message);
      },

      /** 助手消息（沐沐的回复/独白等） */
      addAssistantMessage(
        content: string,
        emotion?: string,
        sceneId?: string,
        presenceVariant?: PresenceMode,
        replyIsFallback?: boolean,
      ) {
        const roleStore = useRoleStore();
        const uiStore = useUiStore();
        const sid = sceneId ?? uiStore.sceneId ?? "default";
        const ts = Date.now();
        const message: ChatMessage = {
          id: `a-${ts}-${Math.random().toString(36).slice(2, 9)}`,
          role: "assistant",
          content,
          timestamp: ts,
          emotion,
          presenceVariant,
          replyIsFallback,
        };
        this.addMessage(roleStore.currentRoleId, sid, message);
      },

      /** 用户消息（用户发送的内容） */
      addUserMessage(content: string, sceneId?: string) {
        const roleStore = useRoleStore();
        const uiStore = useUiStore();
        const sid = sceneId ?? uiStore.sceneId ?? "default";
        const ts = Date.now();
        const message: ChatMessage = {
          id: `u-${ts}-${Math.random().toString(36).slice(2, 9)}`,
          role: "user",
          content,
          timestamp: ts,
        };
        this.addMessage(roleStore.currentRoleId, sid, message);
      },

      addMessage(roleId: string, sceneId: string, msg: ChatMessage) {
        const sid = sceneId || "default";
        const roleBucket = (this.messageMap as unknown as Record<
          string,
          unknown
        >)[roleId];
        // 兼容旧版本：messageMap[roleId] 曾经是 ChatMessage[]
        if (Array.isArray(roleBucket)) {
          const legacy = roleBucket as ChatMessage[];
          (this.messageMap as unknown as Record<string, Record<string, ChatMessage[]>>)[
            roleId
          ] = { [sid]: legacy };
        }
        const current =
          (this.messageMap as unknown as Record<string, Record<string, ChatMessage[]>>)[
            roleId
          ]?.[sid] ?? [];
        const next = [...current, msg];
        if (!this.messageMap[roleId]) (this.messageMap as any)[roleId] = {};
        (this.messageMap as any)[roleId][sid] =
          next.length > MAX_MESSAGES_PER_CONVERSATION
            ? next.slice(-MAX_MESSAGES_PER_CONVERSATION)
            : next;
      },
      clearMessages(roleId: string, sceneId: string) {
        const sid = sceneId || "default";
        const roleBucket = (this.messageMap as unknown as Record<
          string,
          unknown
        >)[roleId];
        if (Array.isArray(roleBucket)) {
          const legacy = roleBucket as ChatMessage[];
          (this.messageMap as unknown as Record<string, Record<string, ChatMessage[]>>)[
            roleId
          ] = { [sid]: legacy };
        }
        if (!this.messageMap[roleId]) (this.messageMap as any)[roleId] = {};
        (this.messageMap as any)[roleId][sid] = [];
      },
      async sendMessage(content: string, sceneId: string): Promise<SendMessageResponse> {
        const roleStore = useRoleStore();
        const roleId = roleStore.currentRoleId;
        const sid = sceneId || "default";
        this.addUserMessage(content, sid);
        this.isLoading = true;
        const relationBefore = roleStore.roleInfo.relationState;
        try {
          const res = await sendMessage({
            role_id: roleId,
            user_message: content,
            scene_id: sid || null,
          });
          const pres = presentationFromSendResponse(res);
          this.addAssistantMessage(
            pres.replyText,
            pres.assistantEmotionLabel,
            sid,
            pres.presenceVariant,
            pres.replyIsFallback,
          );
          useDebugStore().recordKnowledgeFromSend(res);
          roleStore.updateLocalAfterMessage(
            pres.assistantEmotionLabel,
            res.favorability_current,
          );
          if (res.relation_state) {
            const tip = getRelationUpgradeMessage(
              res.relation_state,
              relationBefore,
            );
            if (tip) {
              this.addSystemMessage(tip, sid);
            }
            roleStore.updateRelationState(res.relation_state);
          }
          hostEventBus.emit("message:sent", {
            message: content,
            reply: pres.replyText,
          });
          return res;
        } finally {
          this.isLoading = false;
        }
      },
    },
    persist: true,
  },
);
