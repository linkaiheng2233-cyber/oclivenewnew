import type { DirectoryPluginSlots, PluginBackends } from "./tauri-api";

const SLOT_KEYS: (keyof DirectoryPluginSlots)[] = [
  "memory",
  "emotion",
  "event",
  "prompt",
  "llm",
];

/** 将 `directory_plugins` 槽位格式化为单行调试文本；全空返回 `none`。 */
export function formatDirectoryPluginSlots(
  slots: DirectoryPluginSlots | undefined | null,
): string {
  if (!slots) return "none";
  const parts: string[] = [];
  for (const k of SLOT_KEYS) {
    const raw = slots[k];
    const v = typeof raw === "string" ? raw.trim() : "";
    if (v) parts.push(`${k}=${v}`);
  }
  return parts.length ? parts.join(", ") : "none";
}

/** 任一模块使用 `directory` 或槽位非空时用于决定是否展示「目录插件」行。 */
export function usesDirectoryPlugins(pb: PluginBackends): boolean {
  if (
    pb.memory === "directory" ||
    pb.emotion === "directory" ||
    pb.event === "directory" ||
    pb.prompt === "directory" ||
    pb.llm === "directory"
  ) {
    return true;
  }
  return formatDirectoryPluginSlots(pb.directory_plugins) !== "none";
}
