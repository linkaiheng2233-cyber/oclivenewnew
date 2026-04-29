# Startup Regression + Repackaging Checklist

## Scope
- `oclivenewnew`
- `oclive-launcher`
- `oclive-pack-editor`

## Startup Regression (Cold + Warm)
- [ ] Cold start (first launch after reboot) records: app-open to first interactive UI.
- [ ] Warm start (second launch) records: app-open to first interactive UI.
- [ ] Confirm startup status feedback is visible during initialization (no blank waiting period).
- [ ] Confirm no duplicate initial role info fetch in `oclivenewnew` startup.
- [ ] Confirm launcher first-launch diagnose runs after initial UI render (non-blocking).
- [ ] Confirm launcher startup loads config/announcements/version checks in parallel.
- [ ] Confirm pack-editor non-active pages are not mounted at first paint.
- [ ] Confirm feedback workspace ping/refresh only runs when entering feedback page.

## Core Flow Regression
- [ ] `oclivenewnew`: role switch, send message, import role pack.
- [ ] `oclive-launcher`: launch runtime, launch editor, stop processes, view logs.
- [ ] `oclive-pack-editor`: import pack, edit simple/advanced, export zip/folder, open chat panel.
- [ ] Directory plugin bootstrap still loads shell + UI slots correctly.
- [ ] Role list remains correct (including dev-only visibility behavior).

## Build / Compile Gates
- [x] `oclivenewnew`: `npm run build`
- [x] `oclivenewnew`: `cargo check --manifest-path src-tauri/Cargo.toml`
- [x] `oclive-launcher`: `npm run build`
- [x] `oclive-pack-editor`: `npm run build`

## Repackaging Steps
- [ ] Update release notes with startup/perf and UX changes.
- [ ] Build release artifacts for all three apps.
- [ ] Smoke install/launch each packaged app on a clean machine profile.
- [ ] Verify bundled resources (if any) and startup behavior in packaged mode.
- [ ] Verify no debug-only content shipped in release package.
- [ ] Final sign-off: startup timings + core flows + packaging integrity.
