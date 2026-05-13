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

## Unified test output (`run_test.structured`)

`run_test` returns a **`structured`** object aligned with **[`creator-docs/testing/TEST_OUTPUT_SCHEMA.md`](../../creator-docs/testing/TEST_OUTPUT_SCHEMA.md)** (`schemaVersion: 1`, `kind: "oclive.unit_test_run.v1"`). Hosts should prefer `structured.summary` / `structured.failures` / `structured.suites` for UI.

## Adaptation guide (other runners: Jest / Mocha / Playwright)

1. **Fork the sidecar**: keep the JSON-RPC HTTP bootstrap (`OCLIVE_READY …`) and method names your host expects (`health`, `run_test`, …).  
2. **Swap the CLI** inside `run_test`: replace `npx vitest …` with `npx jest` / `npx mocha` / `npx playwright test`, then map that reporter JSON into the **same `structured` shape** (see schema doc).  
3. **Permissions**: keep `rpc:invoke`; add `process:spawn` only if your runner shells out (same as Vitest path).  
4. **OOCP mocks**: `test_utils/oocp_mock.ts` is **Vitest-agnostic** (plain TS types + objects); reuse in Jest or copy the JSON shapes to another language.  
5. **End-to-end checklist** for a new stack: see **[`creator-docs/ADAPTING_TEST_PLUGIN.md`](../../creator-docs/ADAPTING_TEST_PLUGIN.md)**.

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
