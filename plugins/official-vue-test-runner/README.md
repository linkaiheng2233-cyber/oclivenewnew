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
| `run_test` | `{ cwd: string, specPath?: string, runAll?: boolean, timeoutMs?: number }` | Runs `vitest run` with JSON reporter; returns summary + failures. |

## Discovery

The kernel scans each container such as repo-level `plugins/`: every **subdirectory** that contains a valid `manifest.json` is registered under the manifest **`id`** field (the folder name may differ, e.g. `plugins/official-vue-test-runner/`).

## License

MIT (same family as oclive examples).
