# Semantic slots, multi-appearance, and hotkeys

[中文](../../creator-docs/role-pack/SLOTS_AND_HOTKEYS.md)

## Official slot names (embedded in `ui_slots`)

The host recognizes these **10** semantic slots (`manifest.json` → `ui_slots[].slot`):

| Slot name | Typical mount |
|-----------|---------------|
| `chat_toolbar` | Toolbar above chat input |
| `settings.panel` | Settings → plugin extensions |
| `role.detail` | Left role detail area |
| `sidebar` | Sidebar extension |
| `chat.header` | Top of chat column |
| `settings.plugins` | Inside plugin manager panel |
| `settings.advanced` | Settings dialog “General” extension |
| `overlay.floating` | Bottom-right floating layer on main UI |
| `launcher.palette` | Inside hotkey help overlay |
| `debug.dock` | Debug panel |

## Multi-appearance (`appearance_id`)

One plugin may declare multiple `ui_slots` entries for the **same `slot`**, each with a unique **`appearance_id`** (empty string = “default” single appearance; at most one empty id per slot).

- User choice stored in `plugin_state` → `slot_appearance`: `plugin_id` → `slot` → `appearance_id`.
- Pack `ui.json` / `author.suggested_ui` may set default **plugin id → appearance_id** per slot (`slots.<key>.appearance`; pack editor may auto-fill first variant).

## Global hotkeys

Config in app data **`hotkey_bindings.json`** (beside `plugin_state.json`). Each binding has **`enabled`**: when `false`, no system global hotkey is registered.

Action types:

- **`openPluginSlot`:** `plugin_id`, `slot`, optional `appearance_id` — opens bootstrap page (overlay iframe).
- **`openLauncherList`:** opens simple plugin catalog list.

Creators may document **suggested hotkeys** in docs or market listings; they **do not** reserve system keys — users enable in settings.

## Market metadata (optional)

Listing description JSON may include **`uiSlotVariants`** (similar to host catalog API shape): `{ slot, appearanceId, label }[]` for “multi-appearance” discovery; not required.
