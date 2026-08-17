# RFC: Reply Mode — side-channel capability enhancement

[中文](../../creator-docs/rfc/RFC_REPLY_MODE.md)

| Metadata | Value |
|----------|-------|
| Status | **Draft v1** (design confirmed and locally regression-tested; real-machine UX and separator hit-rate still require manual verification) |
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
| `separator` | string | `"+++"` | Protocol marker between segments; matches a line whose trimmed content equals it exactly (or equals it plus trailing punctuation only) |
| `delays_ms` | uint[] | `[0, 0]` | Visual delay before each segment; first must be 0; short arrays pad with 0, long arrays truncate |
| `streaming` | `"live"` \| `"batch"` | `live` | Frontend splits live during streaming, or shows all segments after generation |
| `fallback_leads` | string[] | `[]` | Natural burst lead-in phrases for degradation splitting on weak local models (e.g. `——`, `而且`); used only when the model never emits the separator protocol; never injected into prompts or exposed to the frontend |

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
你的回复必须分成 N 段。每一段写完后必须换行，单独输出一行分隔符（这一行只有分隔符本身，不允许添加任何文字或标点）：
<separator>
然后再换行写下一段。绝对不允许把各段连成一段，绝对不允许省略分隔符这一行。
```

Any pack enabling the mode gets the protocol automatically; changing `separator` never requires editing persona files. Persona text may describe the "two-burst" tone and rhythm and may say "separate them with the system-provided separator", but must not carry the separator value itself.

## 4. Pipeline order

```text
pre / build_prompt
  → host appends 【输出格式要求】 per reply_mode
  → six-slot LLM generation
  → emo/adult parsing
  → ordinary co-present turns split and strip protocol markers early
  → emotion policy / profile evolution / short_term_memory consume marker-free text
  → reply_post_process polish
  → reply_mode authoritatively splits the final display text again
  → persist chat (one assistant message + segment metadata)
  → SendMessageResponse
  → frontend renders segments / voice reads in order
```

Final presentation splitting still runs after `reply_post_process`: the polisher sees the complete text and the user sees only the final segmented result. In parallel, the kernel derives marker-free semantic text before emotion policy, profile evolution, and short-term-memory consumers run, so `+++` cannot leak through an earlier persistence stage into durable state or next-turn context.

## 5. Split semantics (pure function)

Input `raw`, `separator`, `segments`, `fallback_leads`:

1. Normalize `\r\n` to `\n`.
2. A line is a boundary when its trimmed content exactly equals `separator`, equals the separator followed only by trailing punctuation (`。` / `.` / `!` etc.), or ends with the separator right after a sentence-terminal character (the marker is stripped); other content (including `C+++`, `a +++ b`, `+++abc`) is not a boundary.
3. Trim each segment; drop empty segments.
4. Segments beyond `segments` merge into the last one.
5. When no separator boundary exists, a degradation chain applies (weak local models often omit the protocol): split on blank-line paragraphs first; if still one paragraph, split before a pack-declared `fallback_leads` phrase sitting right after a sentence-terminal punctuation or a line start; otherwise keep one segment.
6. No boundary and no degradation hit → one segment (the whole reply); this is the natural degradation for "the second burst was late" and similar cases.
7. Empty `raw` → empty list.

The live frontend mirrors only primary protocol boundaries (standalone marker lines, marker-plus-trailing-punctuation lines, and markers immediately following sentence-terminal punctuation). Blank-paragraph and `fallback_leads` degradation remains authoritative on the backend after the complete reply is available, avoiding premature live splits of ordinary prose.

## 6. Storage and DTO

**Storage**: still one assistant message.

- `chat_messages.content`: the full reply with separators stripped, segments joined by a newline — used by search, export, memory and the next-turn context.
- `chat_messages.metadata.reply_segments`: `["first burst", "second burst"]`; `reply_segment_delays_ms`: `[0, 300]`. The former rebuilds history bubbles and the latter preserves the turn's presentation-rhythm snapshot.
- `short_term_memory.bot_reply`: ordinary co-present turns use marker-free semantic text before any atomic persistence; adult and remote branches do not enable `reply_mode`.
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

- `streaming = "live"`: the first bubble streams normally; once the accumulated text contains a valid separator boundary, strip the marker and reveal subsequent bubbles in order (up to 8), honoring `delays_ms`.
- `streaming = "batch"`: the transport may still use SSE, but no assistant bubble appears before the complete response; the frontend then renders from authoritative `reply_presentation` with per-segment delays.
- History loading: expand `chat_messages.metadata.reply_segments` into bubbles keyed by base message id plus segment index.
- Narration: split into segments first, then run the existing dialogue/narration split per segment; dialogue stays in its bubble, narration aggregates into the turn's narration strip.
- Voice: ordinary single replies retain low-latency stream speech. With `reply_mode`, raw SSE chunks are never spoken; TTS waits for the authoritative host response and reads only the complete separator-free text, preventing both spoken `+++` and full-reply replay after a prefix mismatch.
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

- [x] Ordinary packs without `reply_mode` retain the existing streaming bubble and low-latency stream voice path
- [x] `burst` + `+++` output splits correctly and the separator never appears in the response, chat log, or short-term memory
- [x] Missing separator, overflow segments, empty segments, CRLF and custom/full-width separators have pure-function unit tests
- [x] One assistant message persists and `metadata.reply_segments` restores any 2–8 sibling bubbles
- [x] `live`, `batch`, and blocking fallback converge on final `reply_presentation`; three-segment live ordering and history reload are tested
- [ ] Manual real-machine confirmation remains for undo / regenerate / delete as one turn (storage is still one assistant row)
- [x] Reply-mode stream voice is suppressed and `message:sent.reply` carries only host-cleaned text; ordinary stream voice remains enabled
- [ ] Manual real-machine confirmation remains for narration aggregation when multiple segments all contain narration
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
