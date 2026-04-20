import termsRaw from "../assets/i18n/pluginTerms.json";

type TermMap = Record<string, string>;

const terms = termsRaw as TermMap;

export function usePluginTerm() {
  function term(key: string): string {
    const v = terms[key];
    if (typeof v === "string" && v.trim().length > 0) {
      return v;
    }
    return key;
  }

  return { term };
}
