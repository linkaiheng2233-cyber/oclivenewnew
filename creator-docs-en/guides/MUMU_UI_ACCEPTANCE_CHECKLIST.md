# mumu front-end modules — release acceptance checklist

Quick pre-release checks that **`distros/chat-pro/roles/mumu/ui.json`** recommended layout and directory-plugin slots behave as expected.

[中文](../../handoff/distros/MUMU_UI_ACCEPTANCE_CHECKLIST.md)

---

## 1) Default modules and slots

- `chat.header` → `com.oclive.mumu.chat-header-status`
- `chat_toolbar` → `com.oclive.mumu.quick-actions`
- `role.detail` → `com.oclive.mumu.role-detail-card`
- `sidebar` → `com.oclive.mumu.sidebar-glance`
- `settings.panel` → `com.oclive.mumu.settings-panel`

**Steps**

1. After launch, press `Ctrl+Shift+F` to open Plugin Manager.
2. Click **Reset to role-pack recommendations** once; confirm all five modules appear in the correct slot lists.
3. Save, restart the app, and confirm the layout is unchanged.

---

## 2) Key interactions

- **`chat_toolbar`**: quick phrases send immediately; scene buttons trigger “past for me only / travel together” as designed.
- **`sidebar`**: “suggested next line” fills the input only; it does not auto-send.
- **`settings.panel`**: “restore defaults” shows a confirmation dialog; after confirm, success/failure feedback appears.
- **`role.detail`**: after a role switch, card fields update on `role:switched`.

---

## 3) Fallbacks and errors

1. Enable **force iframe mode** in Plugin Manager; confirm all five modules still show basic content.
2. If a module fails to load, an error message should appear without breaking the main chat flow.
3. After turning off force iframe mode, Vue components should render normally again.

---

## 4) Release notes

- Before exporting a role pack, confirm **`distros/chat-pro/roles/mumu/ui.json`** `slots` includes the five plugin IDs above.
- If you change module copy or visuals, update the matching notes in **`creator-docs/FAQ.md`** (Chinese hub) or keep **`creator-docs-en/FAQ.md`** in sync where applicable.
