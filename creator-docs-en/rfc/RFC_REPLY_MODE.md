# RFC: Reply Mode — side-channel capability enhancement

[中文](../../creator-docs/rfc/RFC_REPLY_MODE.md)

| Metadata | Value |
|----------|-------|
| Status | **Draft v1** (design confirmed, implementation complete; real-machine separator hit-rate pending manual verification) |
| Audience | Kernel / frontend / pack editor / role pack authors |
| Prerequisites | [RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md](../../creator-docs/rfc/RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md) · [MODULE_MAP_AND_HANDOFF.md](../../handoff/MODULE_MAP_AND_HANDOFF.md) · [ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md) |
| Authoritative Chinese name | **回复模式** |
| Authoritative English name | **Reply Mode** (registry id `reply_mode`) |

## 0. English summary

`reply_mode` is a **side-channel capability enhancement module** that lets a role pack declare how one LLM generation is presented as one or more assistant message segments. v1 ships `single` (default) and `burst` (N segments split on a configurable line-only separator, with optional per-segment display delays).

It is **not** a six-slot backend and **not** a numbered facility submodule. It hooks into the Stable turn chain at `post_llm`, after `reply_post_process`, and only transforms reply presentation. The separator protocol is injected into the generated prompt by the host, so role persona text does not hard-code protocol details.

## 1. Positioning and classification

| Category | Uses six slots? | Numbered facility? | Wiring |
|----------|-----------------|--------------------|--------|
| Modules 1–6 | Yes | — | `PluginHost` → `process_message` |
| Facilities ①–④ | No | Yes | Inside `turn_pipeline` orchestration |
| **`reply_mode`** | **No** | **No** | `turn_pipeline/post.rs` · after `post_llm` · own resolver |

Classification mirrors `reply_post_process`: both are "after LLM output, before the user sees it" reply transforms, but with different responsibilities. `reply_post_process` polishes text; `reply_mode` owns segmentation and presentation rhythm. It therefore gets a new side-channel id instead of reusing or absorbing the post-process channel.

**Default behavior**: unset or `mode = "single"` is byte-for-byte identical to today — one assistant message, one bubble.

## 2. Config schema (role pack `config.json`)

```json
{
  "reply_mode": {
    "mode": "burst",
    "segments": 2,
    "separator": "+++",
    "delays_ms": [0, 300],
    "streaming": "live"
  }
}
```

| Field | Type | Default | Notes |
|-------|------|---------|-------|
| `mode` | `"single"` \| `"burst"` | `single` | v1 supports two modes |
| `segments` | uint | `2` | Expected segment count; 1 equals single; `burst` capped at 8 |
| `separator` | string | `"+++"` | Protocol marker between segments; matches only a line whose trimmed content equals it exactly |
| `delays_ms` | uint[] | `[0, 0]` | Visual delay before each segment; first must be 0; short arrays pad with 0, long arrays truncate |
| `streaming` | `"live"` \| `"batch"` | `live` | Frontend splits live during streaming, or shows all segments after generation |

**Separator validation**:

- Non-empty after trimming
- No `\r` or `\n`
- At most 16 Unicode characters
- Not pure whitespace

On validation failure the pack falls back to `single` with a role-load diagnostic — no silent separator guessing.

## 3. Output protocol and prompt injection

The separator protocol is **injected by the host**, never written into the persona. When the pack enables `reply_mode`, the host appends while assembling the prompt:

```text
【输出格式要求】
本次回复需要分成 N 段。每段之间，单独输出一行分隔符：
<separator>
分隔符前后不要添加任何文字、标点或解释。
```

Any pack enabling the mode gets the protocol automatically; changing `separator` never requires editing persona files. Persona text may describe the "two-burst" tone and rhythm and may say "separate them with the system-provided separator", but must not carry the separator value itself.

## 4. Pipeline order

```text
pre / build_prompt
  → host appends 【输出格式要求】 per reply_mode
  → six-slot LLM generation
  → emo/adult parsing
  → reply_post_process polish
  → reply_mode split + strip separator       ← new channel
  → persist (one assistant message + segment metadata)
  → SendMessageResponse
  → frontend renders segments / voice reads in order
```

`reply_mode` runs after `reply_post_process`: the polisher sees the un-split full text, the splitter sees the final user text — the two channels stay decoupled.

## 5. Split semantics (pure function)

Input `raw`, `separator`, `segments`:

1. Normalize `\r\n` to `\n`.
2. A line is a boundary when its trimmed content exactly equals `separator`, or equals the separator followed only by trailing punctuation (`。` / `.` / `!` etc.); other content (including `C+++`, `a +++ b`, `+++abc`) is not a boundary.
3. Trim each segment; drop empty segments.
4. Segments beyond `segments` merge into the last one.
5. No boundary → one segment (the whole reply); this is the natural degradation for "the second burst was late" and similar cases.
6. Empty `raw` → empty list.

## 6. Storage and DTO

**Storage**: still one assistant message.

- `chat_messages.content`: the full reply with separators stripped, segments joined by a newline — used by search, export, memory and the next-turn context.
- `chat_messages.metadata.reply_segments`: `[{ "text": "...", "delay_ms": 0 }, ...]` — lets frontend history loading rebuild bubbles.
- `message_count`, undo, regenerate, delete still treat the turn as one unit; no new table.

**DTO**:

- `SendMessageResponse.reply`: stays compatible — the full reply with separators stripped.
- New optional `reply_presentation`:

```json
{
  "segments": ["first burst", "second burst"],
  "delays_ms": [0, 300]
}
```

- `RoleInfo` exposes read-only `pack_reply_mode` (`mode` / `segments` / `separator` / `delays_ms` / `streaming`) so the frontend knows the separator before the first streamed token arrives.

## 7. Frontend

- `streaming = "live"`: the first bubble streams normally; when the accumulated text contains a standalone separator line, strip the marker and open the next bubble, honoring `delays_ms`.
- `streaming = "batch"`: render all segments at once after the full response, still honoring per-segment delays.
- History loading: expand `chat_messages.metadata.reply_segments` into bubbles keyed by base message id plus segment index.
- Narration: split into segments first, then run the existing dialogue/narration split per segment; dialogue stays in its bubble, narration aggregates into the turn's narration strip.
- Voice: read the full separator-stripped reply; the first-segment delay only affects visual bubbles, not reading.
- When streaming falls back to the blocking `/chat` path, render from `reply_presentation` — same behavior.

## 8. Degradation and non-goals

- Agent quick replies, degraded short replies, remote life tracks and adult staged beats stay single-segment; they do not go through `reply_mode`.
- Missing separator or a single segment gracefully degrades to one reply.
- Separator validation failure falls back to `single`.
- v1 adds no six-slot type, no facility number, no database table.
- v1 does not change the `short_term_memory` shape; memory stores the separator-stripped full reply.

## 9. Extension: user-defined reply modes

`separator`, `segments`, `delays_ms` and `streaming` are all configurable per role pack; `+++` is only the default. Future semantics (branch replies, multi-role chorus, …):

1. Extend the `mode` enum and the matching host strategy;
2. For third-party implementations, add a directory plugin backend with `provides: ["reply_mode"]` per [RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md](../../creator-docs/rfc/RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md), keeping built-in `burst` as the default.

## 10. Acceptance

- [x] Packs without `reply_mode` behave exactly as today
- [x] `burst` + `+++` output splits correctly and the separator never appears in bubbles or memory
- [x] Missing separator, overflow segments, empty segments, CRLF and custom/full-width separators have pure-function unit tests
- [x] One assistant message persists and `metadata.reply_segments` round-trips
- [x] Streaming and blocking paths both render two bubbles; history reload matches
- [x] Undo / regenerate / delete treat the turn as one unit
- [x] Narration aggregates into the narration strip; voice reads the joined text
- [x] `RoleInfo.pack_reply_mode` is read-only passthrough; frontend never hard-codes role ids
- [x] MODULE_MAP §11 and the RFC_SIDE_CHANNEL registry list `reply_mode`

## 11. Reference anchors

| Topic | Path |
|-------|------|
| post anchor | `kernel/crates/oclive_kernel_host/src/domain/chat_engine/turn_pipeline/post/post_llm.rs` |
| Reply post-process side channel | `kernel/crates/oclive_kernel_host/src/domain/reply_post_processor.rs` |
| Role pack config model | `kernel/crates/oclive_kernel_types/src/models/role_pack_config.rs` |
| Chat persistence | `kernel/crates/oclive_kernel_host/src/infrastructure/chat_storage/chat_messages.rs` |
| Frontend send | `distros/shared/src/stores/chatStoreSend.ts` |
| Frontend history load | `distros/shared/src/stores/chatStoreLoad.ts` |
| Narration split | `distros/shared/src/utils/roleplayReplySplit.ts` |
