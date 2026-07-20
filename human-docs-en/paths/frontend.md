# Frontend contributor path

> **Audience**: Engineers changing Vue / Pinia / Tauri `invoke`.  
> **Time**: ~1 day to onboard.  
> **Chinese SSOT**: [`human-docs/paths/frontend.md`](../../human-docs/paths/frontend.md)
> **Next**: [modules/surfaces/](../modules/surfaces/) · [05 debugging](05_DEBUGGING.md)

---

## Suggested order

1. [02 thirty-minute start](02_THIRTY_MINUTE_START.md) — `npm run tauri:dev` + `npm run check`
2. [03 glossary](03_GLOSSARY.md) — **`reply`** not `response`; invoke **camelCase**
3. [04 engineering rules summary](04_ENGINEERING_RULES_SUMMARY.md) — §3, §4, §7 (full ZH: [04](../../human-docs/04_ENGINEERING_RULES.md))
4. **Surface packs** (~30–45 min):
   - Chat Pro UI → [modules/surfaces/frontend-chat-pro.md](../modules/surfaces/frontend-chat-pro.md)
   - New invoke → [modules/surfaces/tauri-invoke.md](../modules/surfaces/tauri-invoke.md)
5. Read `distros/shared/src/api/` wrappers and `distros/shared/src/stores/chatStore.ts`

---

## Key paths

| Task | Path |
|------|------|
| Send message | `distros/shared/src/api/chat.ts` → `send_message` |
| Chat state | `distros/shared/src/stores/chatStore.ts` |
| Plugin manager | `Ctrl+Shift+F` → `SimplePluginManagerPanel.vue` |
| Model manager | `Ctrl+Shift+M` → `ModelManagerPanel.vue` |
| All Tauri commands | `distros/desktop-tauri/src/lib.rs` `generate_handler!` |

---

## Testing

| Scenario | Command |
|----------|---------|
| Unit | `npm run test:unit` |
| Build | `npm run build` |
| E2E (Linux CI aligned) | `npm run test:e2e:preview` |

---

## Deep links

- [modules/ picker](../modules/README.md)
- [NAMING §8 frontend mapping](../../creator-docs/NAMING_CONVENTIONS.md)
- [COMPATIBILITY](../../creator-docs/COMPATIBILITY.md)
