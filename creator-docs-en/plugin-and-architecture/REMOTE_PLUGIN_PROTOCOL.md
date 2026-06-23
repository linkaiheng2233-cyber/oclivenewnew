# Remote plugin protocol (host ↔ HTTP sidecar) — full reference

**Implementation status**: the host implements an **HTTP POST + JSON‑RPC 2.0** client under `kernel/crates/oclive_kernel_host/src/infrastructure/remote_plugin/`. When a pack sets a subsystem to `remote` and env URLs are set, requests go to the sidecar; on **network errors, non‑2xx HTTP, JSON‑RPC `error`, or result deserialization failure**, the host **falls back to built‑in implementations** and logs (`target: oclive_plugin`) — chat usually continues.

[中文](../../creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md)

---

## 1. Transport

### 1.1 URL & HTTP

- **Method**: **`POST`** to the **full URL** from env (may include path), e.g. `http://127.0.0.1:8765/rpc`.  
- **`Content-Type`**: host sends **`application/json`**.  
- **Headers** (fixed):  
  - `x-oclive-remote-protocol: oclive-remote-jsonrpc-v1`  
  - `x-oclive-client-version: <host version>` (e.g. `0.2.0`)  
- **Auth**: if `OCLIVE_REMOTE_PLUGIN_TOKEN` / `OCLIVE_REMOTE_LLM_TOKEN` are set, host adds **`Authorization: Bearer <token>`**.  
- **Timeouts**: `OCLIVE_REMOTE_PLUGIN_TIMEOUT_MS` (default 8000 ms) and `OCLIVE_REMOTE_LLM_TIMEOUT_MS` (default 120000 ms), clamped in host `config.rs`.

### 1.2 JSON‑RPC 2.0 request (host → sidecar)

One JSON object per HTTP body:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "memory.rank",
  "params": { }
}
```

| Field | Meaning |
|-------|---------|
| `jsonrpc` | literal `"2.0"` |
| `id` | positive int (host increments); echo in response |
| `method` | name from §4 below |
| `params` | object; per‑method fields in §4 |

### 1.3 JSON‑RPC 2.0 success response

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": { }
}
```

`result` shape is per method; the host accepts **`result` as an object** with expected fields, or **`result` as a string** for a few methods.

### 1.4 JSON‑RPC error response

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32601,
    "message": "Method not found"
  }
}
```

Any `error` object is treated as failure → built‑in fallback.

#### Recommended error codes (product)

The host logs `error.code` / `error.message` / `error.data` as‑is; sidecars should prefer:

| code | name | meaning |
|------|------|---------|
| `-32700` | `parse_error` | body not valid JSON |
| `-32600` | `invalid_request` | malformed JSON‑RPC envelope |
| `-32601` | `method_not_found` | unknown method |
| `-32602` | `invalid_params` | missing/wrong types |
| `-32603` | `internal_error` | sidecar internal |
| `-32010` | `plugin_timeout` | upstream (model/retrieval) timeout |
| `-32011` | `auth_failed` | bad token / permission |
| `-32012` | `rate_limited` | throttled |
| `-32013` | `upstream_unavailable` | dependency down |

### 1.5 HTTP status

Prefer **HTTP 200** with machine‑readable errors in JSON‑RPC `error`. **4xx/5xx** is treated like transport failure → fallback.

---

## 2. Environment variables (host)

| Variable | Role |
|----------|------|
| `OCLIVE_REMOTE_PLUGIN_URL` | When set, **one** HTTP endpoint for **memory / emotion / event / prompt** remote; distinguished by `method` |
| `OCLIVE_REMOTE_PLUGIN_TIMEOUT_MS` | optional; default `8000` ms |
| `OCLIVE_REMOTE_PLUGIN_TOKEN` | optional Bearer |
| `OCLIVE_REMOTE_LLM_URL` | When set **and** pack `plugin_backends.llm = remote`, **LLM** calls use this URL |
| `OCLIVE_REMOTE_LLM_TIMEOUT_MS` | optional; default `120000` |
| `OCLIVE_REMOTE_LLM_TOKEN` | optional Bearer |

If the URL for a subsystem is missing, the host uses **built‑in placeholders** even when the pack says `remote`, and may log a warning once.

---

## 2. Security & product boundaries (current implementation)

| Topic | Notes |
|-------|--------|
| **Protocol version** | Header **`x-oclive-remote-protocol: oclive-remote-jsonrpc-v1`** labels this JSON‑RPC shape; sidecars may reject unknown hosts. |
| **Timeouts & codes** | See §1.1 / §1.4; timeouts and non‑2xx → **fallback**, so remote outages do not crash chat. |
| **HTTP sidecar (today)** | Host only **POSTs** to user‑configured URLs; **no** auto‑download of arbitrary binaries from packs. Put secrets in **env**, not in committed packs. |
| **Future: local exe sidecars** | If added, document separately: path declaration, first‑run **user consent**, sandboxing, signing — until then, this HTTP model is authoritative. |

---

## 3. Rust enum JSON shapes (sidecars must read)

The host uses **serde default externally tagged enums** — **not** bare strings.

### 3.1 `EventType` (`event.estimate` → `result.event_type`)

Variants: `Quarrel`, `Apology`, `Praise`, `Complaint`, `Confession`, `Joke`, `Ignore`.

**Correct** (`Ignore`):

```json
"event_type": { "Ignore": null }
```

**Wrong** (deserialization fails → builtin):

```json
"event_type": "Ignore"
```

### 3.2 `Emotion` (`event.estimate` → `params.user_emotion`)

Variants: `Happy`, `Sad`, `Angry`, `Neutral`, `Excited`, `Confused`, `Shy`.

Example:

```json
"user_emotion": { "Neutral": null }
```

### 3.3 `EmotionResult` (`emotion.analyze` → `result`)

Flat object, seven `f64` fields:

```json
{
  "joy": 0.0,
  "sadness": 0.0,
  "anger": 0.0,
  "fear": 0.0,
  "surprise": 0.0,
  "disgust": 0.0,
  "neutral": 1.0
}
```

### 3.4 `PersonalitySource` (`event.estimate` / `prompt.build_prompt`)

Orchestration enum; in JSON this is a **string** (not the §3.1 object style):

- `"vector"` — classic seven‑dim evolution path.  
- `"profile"` — core + mutable archives drive behavior; seven dims are mostly a summarized view.

Both **`event.estimate`** and **`prompt.build_prompt`** include top‑level **`personality_source`** in `params` (aligned with `role.evolution_config` / pack `evolution`). Sidecars may read only this field.

---

## 4. Methods & examples

`params` below = JSON‑RPC **`params` object**.

### 4.1 `memory.rank`

**params**

| field | type | notes |
|-------|------|-------|
| `memories` | array | `Memory` objects (`id`, `role_id`, `content`, `importance`, `weight`, `created_at` ISO8601, optional `scene_id`) |
| `user_query` | string | current user line |
| `scene_id` | string or `null` | current scene |
| `limit` | int | max memories |

**result**

| field | type | notes |
|-------|------|-------|
| `ordered_ids` | string[] | ordered `Memory.id`; unknown ids skipped; ids not listed keep **original array order** at tail until `limit` |

**Request example**

```json
{
  "jsonrpc": "2.0",
  "id": 10,
  "method": "memory.rank",
  "params": {
    "memories": [
      {
        "id": "m1",
        "role_id": "demo",
        "content": "last time we talked about weather",
        "importance": 0.8,
        "weight": 1.0,
        "created_at": "2026-04-01T12:00:00Z",
        "scene_id": "home"
      }
    ],
    "user_query": "going out today?",
    "scene_id": "home",
    "limit": 8
  }
}
```

**Response example**

```json
{
  "jsonrpc": "2.0",
  "id": 10,
  "result": {
    "ordered_ids": ["m1"]
  }
}
```

### 4.2 `emotion.analyze`

**params**

| field | type |
|-------|------|
| `text` | string |

**result**: **`EmotionResult`** (§3.3).

### 4.3 `event.estimate`

**params**

| field | type |
|-------|------|
| `ollama_model` | string |
| `user_message` | string |
| `user_emotion` | `Emotion` (§3.2) |
| `personality` | `PersonalityVector` (seven `f64`) |
| `personality_source` | `"vector"` \| `"profile"` (§3.4) |
| `recent_turns` | `[[user, bot], ...]` strings |
| `recent_events` | `Event[]` (`event_type` §3.1; `user_emotion`/`bot_emotion` strings) |
| `knowledge_augment` | object or `null` |

**result**: **`EventImpactEstimate`**

| field | type |
|-------|------|
| `event_type` | `EventType` (§3.1) |
| `impact_factor` | number |
| `confidence` | number (0–1) |

**Example result**

```json
{
  "event_type": { "Ignore": null },
  "impact_factor": 0.0,
  "confidence": 0.5
}
```

### 4.4 `prompt.build_prompt`

**params**: flat object aligned with serialized `PromptInput`, including:

- `role` (large)  
- **`personality_source`** (§3.4)  
- `personality`, `memories`, `user_input`, `user_emotion`, relation fields, previews, …  
- `event_type`, `impact_factor`  
- `scene_label`, `scene_detail`, `topic_hint_line`, `life_context_line`, …  

**result**

- object with `"prompt": "<string>"`, **or**  
- **`result` itself a string** → whole prompt

### 4.5 `prompt.top_topic_hint`

**params**: `role`, `scene_id`  
**result**: `{ "hint": "..." }` / `null`, or raw string.

### 4.6 `llm.generate` / `llm.generate_tag`

**params**: `model`, `prompt` (strings)  
**result**: `{ "text": "..." }` or raw string.  
`generate_tag` — low temperature short outputs (sprites, travel intent, …).

---

## 5. Security & ops

- URLs and tokens belong in **deployment env**, not shared packs.  
- Sidecars should bound body size, connections, and redact logs.  
- **Do not** silently download and run untrusted binaries from the network.

---

## 6. Versioning

- **v1** in this repo is authoritative; prefer **ignoring unknown keys** on both sides when evolving. Today the host deserializes fixed structs — **unknown shapes fall back to builtin**.  
- Non‑HTTP child sidecars are out of scope for the current HTTP client (see historical notes).

---

## 7. Related docs

- [CREATOR_PLUGIN_ARCHITECTURE.md](CREATOR_PLUGIN_ARCHITECTURE.md) — creator overview & bring‑up  
- [PLUGIN_V1.md](PLUGIN_V1.md) — `plugin_backends` enums  
- [../getting-started/DOCUMENTATION_INDEX.md](../getting-started/DOCUMENTATION_INDEX.md) — hub
