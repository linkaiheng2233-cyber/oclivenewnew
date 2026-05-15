# REGRESSION — Plugin Manager V2 & Complex Emotion

Manual checklist for **Plugin Manager V2** and **Complex Emotion** related changes.

[中文](../../creator-docs/guides/REGRESSION_COMPLEX_EMOTION_QA.md)

## Before you start

- Run the latest build.
- Confirm **`Ctrl+Shift+F`** works in your environment.
- Note whether a session/app restart is required for each step.

## Checklist

### 1) Feature flag default OFF

- Settings: **“Enable new plugin manager (V2 preview)”** is **off** by default.
- **`Ctrl+Shift+F`** opens legacy **`PluginManagerPanel`**.
- Legacy tabs (list, backends, slots) behave normally.

### 2) With flag ON, route to V2

- Turn **V2 preview** on in settings.
- **`Ctrl+Shift+F`** opens **`PluginManagerV2`**.
- **`Ctrl+Shift+F`** or **`Esc`** closes it reliably.

### 3) Left rail filters & badges

- Click: All / LLM / Emotion / Complex Emotion / Built‑in / Remote / Directory / Needs setup.
- Center cards update immediately.
- Left badge counts match the filtered center list.

### 4) Live search

- Type `llm`, `emotion`, `complex`, `remote`, … in the search box.
- Center list filters live; no extra submit.
- Clearing search restores the full list for the current filter.

### 5) Card selection & right detail

- Selecting a card shows title, description, templated form.
- Templates render: `endpoint-config` / `provider-selector` / `slot-selector` / `switch-toggle`.
- Switching cards refreshes the right pane.

### 6) Save & toasts

- Change LLM or Emotion config → **Apply**.
- Success toast; **effective** backend values in role info update.
- If a restart/session is required, the copy must say so explicitly.

### 7) Right pane collapse

- **Collapse** narrows the detail strip; **Expand** restores full detail.

### 8) “Open advanced (V1)” shortcut

- V2 header **open advanced** closes V2 and opens **`PluginManagerPanel`**.
- **`Ctrl+Shift+F`** afterwards follows the V1/V2 toggle state.

### 9) Complex Emotion read‑only

- Complex Emotion card is **status/read‑only** — no accidental env editing.
- Refresh updates “remote sidecar detected” state.
- Copy must clarify **Complex Emotion ≠ emotion backend dropdown**.

## Result template

- Verdict: pass / fail
- If fail:
  - [ ] Item id + repro
  - [ ] Expected
  - [ ] Actual
  - [ ] Screenshot / logs (if any)
