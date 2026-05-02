import type { InjectionKey } from "vue";
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";

const CLOUD_LLM_TRUST_CAP_I18N_KEYS = [
  "settings.cloudLlmTrust.caps.net",
  "settings.cloudLlmTrust.caps.secret",
  "settings.cloudLlmTrust.caps.perm",
  "settings.cloudLlmTrust.caps.local",
] as const;

/** 与 TrustConsentModal / 原生 confirm 共用同一段只读说明正文（纯文本）。 */
export function buildCloudLlmTrustPlainText(t: (key: string) => string): string {
  const caps = CLOUD_LLM_TRUST_CAP_I18N_KEYS.map((k) => `• ${String(t(k))}`);
  return [
    String(t("settings.cloudLlmTrust.modal.subtitle")),
    "",
    String(t("settings.cloudLlmTrust.modal.trustSummaryTitle")),
    String(t("settings.cloudLlmTrust.modal.trustSummaryBody")),
    "",
    String(t("settings.cloudLlmTrust.modal.hint")),
    "",
    ...caps,
  ].join("\n");
}

/** 云端 LLM 风险说明弹层：与设置页 / 本地模型页共用文案与能力列表。 */
export function useCloudLlmTrustModal() {
  const { t } = useI18n();
  const visible = ref(false);

  const capabilities = computed(() =>
    CLOUD_LLM_TRUST_CAP_I18N_KEYS.map((k) => String(t(k))),
  );

  const modalTitle = computed(() => String(t("settings.cloudLlmTrust.modal.title")));
  const modalSubtitle = computed(() => String(t("settings.cloudLlmTrust.modal.subtitle")));
  const trustSummaryTitle = computed(() => String(t("settings.cloudLlmTrust.modal.trustSummaryTitle")));
  const trustSummaryBody = computed(() => String(t("settings.cloudLlmTrust.modal.trustSummaryBody")));
  const modalHint = computed(() => String(t("settings.cloudLlmTrust.modal.hint")));
  const confirmLabel = computed(() => String(t("settings.cloudLlmTrust.modal.allow")));

  function open(): void {
    visible.value = true;
  }

  function close(): void {
    visible.value = false;
  }

  return {
    visible,
    capabilities,
    modalTitle,
    modalSubtitle,
    trustSummaryTitle,
    trustSummaryBody,
    modalHint,
    confirmLabel,
    open,
    close,
  };
}

export type CloudLlmTrustModalApi = ReturnType<typeof useCloudLlmTrustModal>;

/** 本地模型面板向子组件注入同一套「云端能力说明」状态（避免 Teleport 与外层遮罩命中错乱）。 */
export const cloudLlmTrustModalKey: InjectionKey<CloudLlmTrustModalApi> = Symbol("cloudLlmTrustModal");

/**
 * 仅由 LocalModelManagerPanel 提供：在 Tauri 内用系统 confirm 展示说明（规避嵌套 WebView 模态点击问题）；
 * 非 Tauri 时由提供者回退为打开 Vue TrustConsentModal。
 */
export const cloudLlmTrustReadmeOpenerKey: InjectionKey<() => Promise<void>> = Symbol("cloudLlmTrustReadmeOpener");
