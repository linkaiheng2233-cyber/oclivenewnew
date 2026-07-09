# Creator golden path (V4 outline · ≤2 pages)

[中文](../../creator-docs/getting-started/CREATOR_GOLDEN_PATH.md)

**Status:** Wave 5 expansion (after Theater mode 2 product gate); step-by-step screenshots planned for next minor.  
**Audience:** Beginner creators — a **talkable role pack in 30 minutes**, **without** `slot_registry` / blueprint orchestration.

---

## 0. Prerequisites (5 minutes)

- Install OCLive desktop or open [oclive-pack-editor](https://github.com) (sister-repo pack editor)
- Local Ollama or a cloud API key (Theater demo can run zero-config)
- Clone/download the official sample pack `distros/chat-pro/roles/mumu` as reference (not a product ceiling)

## 1. Initialize the role pack (5 minutes)

1. Pack editor: **New role pack** → fill `manifest.json` (`id`, `name`, `version`)
2. Choose the **v2 minimal template** (`pipeline.ocblueprint` + `settings.json` with default six-slot builtin)
3. Save under `distros/chat-pro/roles/{your_id}/`

**Do not change on this path:** multi-instance `slot_registry`, `groups`, or Experimental core.

## 2. Identity and personality (10 minutes)

| File | Purpose |
|------|---------|
| `prompts/system.md` | One-line role + tone examples |
| `config.json` → `reply_quality_anchor` | Optional: replace the default reply anchor (cannot replace guardrails) |
| `user_identities/` (optional) | One default `.md` + `index.json` |

**Acceptance:** Pack editor prompt preview shows system / role / user layers; no blueprint step fields.

## 3. Local trial run (5 minutes)

### Desktop (general)

```powershell
$env:OCLIVE_ROLES_DIR = "path\to\roles"
npm run tauri:dev
```

- Select the new role → send “hello”
- **Ctrl+Shift+F** opens plugin management (`SimplePluginManagerPanel`); **Ctrl+Shift+M** opens model management
- Settings → Model management: confirm the LLM backend is reachable

### Theater distro (`distro_id=theater`)

```powershell
npm run dev:theater
```

- Breakfast scene · dual-role comparison · 3 poke chips (see [theater/DEVELOPMENT_ROADMAP.md](../../handoff/theater/DEVELOPMENT_ROADMAP.md) §4)
- Automated smoke: `npm run test:unit` → `distros/theater/src/theater.acceptance.test.ts` (9 tests)

### Theater mode 2 · script outline (optional +10 minutes)

> Product gate passed (friend cohort); entry: **More → Write script outline**. Not on the chat main chain.

1. Top bar **More** → **Write script outline**
2. Write 2–5 sentences about what happens in this scene (e.g. supermarket milk scramble, forgot wallet)
3. Pick character cards, then **Generate and play**
4. On failure, canned dialogue fallback; retry with a stronger model via **Ctrl+Shift+M**

Contract: [`handoff/theater/MODE2_RFC.md`](../../handoff/theater/MODE2_RFC.md)

## 4. Distribution and next steps (5 minutes)

- `oclive-cli` / pack editor: **pack** → `.oclive-plugin` or full zip
- Doc index: [ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md) · [DOCUMENTATION_INDEX.md](./DOCUMENTATION_INDEX.md)

**Advanced (outside the 30-minute path):** directory plugins, remote slots, `distro.oclive.toml`, Agent/MCP.

---

## Related debt items

- **V4 full edition:** After mode 2 is demo-ready, expand this outline into step-by-step screenshots (including outline mode).
- **Expert routing demo:** `mumu` pack `blueprint/includes/expert_routing.json` (`enabled: false`; not invoked by default).
