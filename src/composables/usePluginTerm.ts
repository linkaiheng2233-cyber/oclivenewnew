import { useI18n } from "vue-i18n";

export function usePluginTerm() {
  const { t } = useI18n();
  function term(key: string): string {
    return t(`pluginTerms.${key}`);
  }
  return { term };
}
