/**
 * 读取宿主当前「有效深浅色」与界面缩放（与 `html[data-theme]`、`--oclive-ui-scale` 一致）。
 * 供插件 `oclive.getAppearance()` 与内置事件载荷使用。
 */
export function readHostAppearance(): {
  effectiveTheme: "light" | "dark";
  scale: number;
} {
  const dt = document.documentElement.getAttribute("data-theme");
  const effectiveTheme: "light" | "dark" = dt === "dark" ? "dark" : "light";
  const raw = getComputedStyle(document.documentElement)
    .getPropertyValue("--oclive-ui-scale")
    .trim();
  const scale = Number.parseFloat(raw) || 1;
  return { effectiveTheme, scale };
}
