import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";

/** 云端 LLM 风险说明弹层：与设置页 / 本地模型页共用文案与能力列表。 */
export function useCloudLlmTrustModal() {
  const { t } = useI18n();
  const visible = ref(false);

  const capabilities = computed(() => [
    String(t("settings.cloudLlmTrust.caps.net")),
    String(t("settings.cloudLlmTrust.caps.secret")),
    String(t("settings.cloudLlmTrust.caps.perm")),
    String(t("settings.cloudLlmTrust.caps.local")),
  ]);

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
