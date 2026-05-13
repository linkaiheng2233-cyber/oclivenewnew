# official-vue-test-runner

Directory plugin (`com.oclive.official_vue_test_runner`) that exposes a JSON-RPC sidecar for running **Vitest** against an oclive workspace.

## Layout

- `manifest.json` — `process` launches `node rpc_server.mjs`; `shell.bridge.invoke` declares `rpc:invoke` and `process:spawn` for install-time permission consent.
- `rpc_server.mjs` — HTTP JSON-RPC on a random localhost port; prints `OCLIVE_READY <url>` on stdout for the host to bind.

## RPC methods

| Method | Params | Description |
|--------|--------|-------------|
| `echo.ping` | `{ text?: string }` | Smoke test. |
| `health` | `{ cwd?: string }` | Runs `npx vitest --version` in `cwd` (default plugin cwd). |
| `list_test_files` | `{ root?: string }` | Recursively lists `*.spec.ts` / `*.test.ts` under `root`. |
| `run_test` | `{ cwd: string, specPath?: string, runAll?: boolean, timeoutMs?: number }` | Runs `vitest run` with JSON reporter; returns **legacy** `summary` / `failures` plus **`structured`** (通过率、定位、耗时 headline). |
| `get_history` | `{ limit?: number }` | Returns up to **20** (default) or **limit** (max 100) recent run summaries from `test_history.json` beside `rpc_server.mjs`. |
| `clear_history` | `{}` | Clears the local history file. |

## OOCP helpers for Vitest (`test_utils/oocp_mock.ts`)

Use these when a Vue/TS test needs **wire-shaped** OOCP JSON (request / response envelopes) without running the kernel.

From a spec one folder under `src/` (e.g. `src/foo.spec.ts`), import with:

```ts
import { describe, expect, it } from "vitest";
import {
  createTestSession,
  mockOocpRequest,
  mockOocpResponse,
} from "../../plugins/official-vue-test-runner/test_utils/oocp_mock";

describe("OOCP stubs", () => {
  it("builds session.create + chat.send_message shapes", () => {
    const s = createTestSession({ roleId: "demo.role" });
    expect(s.sessionCreateRequest.method).toBe("session.create");
    expect(s.sessionCreateResponse.result.session_ns).toBe(s.sessionNs);

    const send = mockOocpRequest("chat.send_message", {
      ...s.chatSendMessageParams,
      user_message: "ping",
    });
    expect(send.params.session_ns).toBe(s.sessionNs);

    const reply = mockOocpResponse(send.id, {
      reply: "pong",
      bot_emotion: "neutral",
      portrait_emotion: "neutral",
    });
    expect(reply.result.reply).toBe("pong");
  });
});
```

Deeper files under `src/` need more `../` until the import reaches the repo root.

Types and field names follow **`crates/oclive_core/src/oocp/mod.rs`** and **[`creator-docs/oocp/OOCP_SPEC_v0_1.md`](../../creator-docs/oocp/OOCP_SPEC_v0_1.md)** (`role_id`, `session_ns`, etc.).

## Discovery

The kernel scans each container such as repo-level `plugins/`: every **subdirectory** that contains a valid `manifest.json` is registered under the manifest **`id`** field (the folder name may differ, e.g. `plugins/official-vue-test-runner/`).

## License

MIT (same family as oclive examples).
