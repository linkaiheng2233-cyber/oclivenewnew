import { describe, it, expect } from "vitest";
import enUS from "../i18n/locales/en-US";
import zhCN from "../i18n/locales/zh-CN";

/** Leaf key paths for nested message objects (arrays skipped). */
function flattenMessageKeys(obj: unknown, prefix = ""): Set<string> {
  const out = new Set<string>();
  if (obj === null || typeof obj !== "object") {
    if (prefix) out.add(prefix);
    return out;
  }
  if (Array.isArray(obj)) {
    return out;
  }
  for (const k of Object.keys(obj as Record<string, unknown>)) {
    const path = prefix ? `${prefix}.${k}` : k;
    const v = (obj as Record<string, unknown>)[k];
    if (v !== null && typeof v === "object" && !Array.isArray(v)) {
      for (const p of flattenMessageKeys(v, path)) out.add(p);
    } else {
      out.add(path);
    }
  }
  return out;
}

describe("i18n locale parity (zh-CN vs en-US)", () => {
  it("has the same key tree in both catalogs (no missing en-US leaves)", () => {
    const zhKeys = flattenMessageKeys(zhCN);
    const enKeys = flattenMessageKeys(enUS);
    const missingInEn = [...zhKeys].filter((k) => !enKeys.has(k)).sort();
    expect(missingInEn, `Missing in en-US: ${missingInEn.join(", ")}`).toEqual([]);
  });

  it("has the same key tree in both catalogs (no missing zh-CN leaves)", () => {
    const zhKeys = flattenMessageKeys(zhCN);
    const enKeys = flattenMessageKeys(enUS);
    const missingInZh = [...enKeys].filter((k) => !zhKeys.has(k)).sort();
    expect(missingInZh, `Missing in zh-CN: ${missingInZh.join(", ")}`).toEqual([]);
  });
});
