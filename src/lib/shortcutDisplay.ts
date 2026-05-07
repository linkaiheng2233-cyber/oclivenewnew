/**
 * 平台相关快捷键展示与键盘事件判断（macOS 用 ⌘/Meta，其它平台用 Ctrl）。
 */

export function isMacLikePlatform(): boolean {
  if (typeof navigator === "undefined") return false;
  return /Mac|iPhone|iPad|iPod/i.test(navigator.userAgent);
}

/** 界面文案中的主修饰键（表格、说明文字） */
export function shortcutPrimarySymbolForDisplay(): string {
  return isMacLikePlatform() ? "⌘" : "Ctrl";
}

/** 与 `shortcutPrimarySymbolForDisplay` 对应的键盘事件主修饰键是否按下 */
export function chordModifierKeyDown(e: KeyboardEvent): boolean {
  if (isMacLikePlatform()) {
    return e.metaKey;
  }
  return e.ctrlKey;
}

/** 用于快捷键表一行的「主修饰键 + Shift + 字母」 */
export function formatChordModShift(letter: string): string {
  const m = shortcutPrimarySymbolForDisplay();
  const L = letter.length === 1 ? letter.toUpperCase() : letter;
  return `${m} + Shift + ${L}`;
}

/**
 * 将文案中的字面量 `{m}` 替换为当前平台主修饰键（供 i18n merge 时调用；避免各处手动传参）。
 */
export function applyShortcutModTokens<T>(input: T): T {
  const mod = shortcutPrimarySymbolForDisplay();
  const walk = (v: unknown): unknown => {
    if (typeof v === "string") return v.split("{m}").join(mod);
    if (Array.isArray(v)) return v.map(walk);
    if (v && typeof v === "object") {
      const o = v as Record<string, unknown>;
      const out: Record<string, unknown> = {};
      for (const [k, val] of Object.entries(o)) {
        out[k] = walk(val);
      }
      return out;
    }
    return v;
  };
  return walk(input) as T;
}
