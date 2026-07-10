# Facility pack · visual stage (EN summary)

> Full checklist (ZH): [`human-docs/modules/facilities/visual-stage.md`](../../human-docs/modules/facilities/visual-stage.md)  
> RFC SSOT: [RFC_VISUAL_PRESENTATION](../../creator-docs/rfc/RFC_VISUAL_PRESENTATION.md) · [MODULE_MAP §10 facility ④](../../handoff/MODULE_MAP_AND_HANDOFF.md)

**You plug in**: Input `visual_state_id` (from facility ③ portrait) · output `performance_directive` → host UI frame loop · **off** by default · **no** AI image pick.

**Do**: Vue frame loop · directive consumption · Chat Pro `distros/chat-pro` UI presentation · graceful degrade when facility off.

**Don't**: Call LLM for image pick in visual layer · write to six-slot `plugin_backends` · push penetration into `process_message` order.

**Read next**: [portrait](portrait.md) · [frontend-chat-pro](../surfaces/frontend-chat-pro.md) · [RFC_PORTRAIT_FACILITY](../../creator-docs/rfc/RFC_PORTRAIT_FACILITY.md).
