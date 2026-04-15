import { onBeforeUnmount, watch } from "vue";
import { useRoleStore } from "../stores/roleStore";
import type { PackUiTheme } from "../utils/tauri-api";
import { hostEventBus } from "../lib/hostEventBus";

/**
 * 将角色包 `ui.json` → `theme` 映射到 Fluent / oclive CSS 变量；切换角色或清空字段时移除内联覆盖以回退内置主题。
 */
export function usePackUiTheme(): void {
  const roleStore = useRoleStore();
  let applied: string[] = [];

  function clearApplied(): void {
    const root = document.documentElement;
    for (const k of applied) {
      root.style.removeProperty(k);
    }
    applied = [];
  }

  function applyTheme(t: PackUiTheme | undefined): void {
    clearApplied();
    const root = document.documentElement;
    const push = (key: string, val: string): void => {
      root.style.setProperty(key, val);
      applied.push(key);
    };
    const pc = t?.primaryColor?.trim();
    if (pc) {
      push("--fluent-accent", pc);
      push("--accent", pc);
      push("--text-accent", pc);
      hostEventBus.emitBuiltin("theme:changed", { primaryColor: pc });
    }
    const bg = t?.backgroundColor?.trim();
    if (bg) {
      push("--fluent-bg-page", bg);
      push("--bg-page", bg);
      push("--shell-page-bg", bg);
    }
    const ff = t?.fontFamily?.trim();
    if (ff) {
      push("--font-ui", `${ff}, system-ui, sans-serif`);
    }
  }

  watch(
    () => ({
      roleId: roleStore.currentRoleId,
      theme: roleStore.roleInfo.packUiConfig?.theme,
    }),
    () => applyTheme(roleStore.roleInfo.packUiConfig?.theme ?? {}),
    { deep: true, immediate: true },
  );

  onBeforeUnmount(() => {
    clearApplied();
  });
}
