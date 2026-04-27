export const OFFICIAL_UI_SLOTS: readonly string[] = [
  "chat_toolbar",
  "settings.panel",
  "role.detail",
  "sidebar",
  "chat.header",
  "settings.plugins",
  "settings.advanced",
  "overlay.floating",
  "launcher.palette",
  "debug.dock",
] as const;

export function isOfficialUiSlot(slot: string): boolean {
  return OFFICIAL_UI_SLOTS.includes(slot);
}

