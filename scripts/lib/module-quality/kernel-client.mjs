import { spawn, spawnSync } from "node:child_process";
import { createServer as createNetServer } from "node:net";
import { join } from "node:path";

import { assert, fail } from "./contracts.mjs";

export function freePort() {
  return new Promise((resolvePort, reject) => {
    const server = createNetServer();
    server.once("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const address = server.address();
      const port = typeof address === "object" && address ? address.port : 0;
      server.close((error) => (error ? reject(error) : resolvePort(port)));
    });
  });
}

export function listen(server, port) {
  return new Promise((resolveListen, reject) => {
    server.once("error", reject);
    server.listen(port, "127.0.0.1", resolveListen);
  });
}

export function closeServer(server) {
  return new Promise((resolveClose) => server.close(() => resolveClose()));
}

export function spawnKernel({
  binary,
  port,
  rolesRoot,
  appDataRoot,
  sidecarUrl,
  apiToken,
  repoRoot,
}) {
  return spawn(binary, ["--api", "--port", String(port)], {
    cwd: repoRoot,
    env: {
      ...process.env,
      OCLIVE_APP_DATA: appDataRoot,
      OCLIVE_API_TOKEN: apiToken,
      OCLIVE_API_USE_TEMP_APP_DATA: "0",
      OCLIVE_USE_CANONICAL_APP_DATA: "1",
      OCLIVE_LLM_BACKEND: "remote",
      OCLIVE_LLM_CLOUD_API_STYLE: "oclive_jsonrpc",
      OCLIVE_REMOTE_LLM_URL: sidecarUrl,
      OCLIVE_REMOTE_PLUGIN_URL: sidecarUrl,
      OCLIVE_REMOTE_FALLBACK_TO_BUILTIN: "0",
      OCLIVE_ROLES_DIR: rolesRoot,
      OCLIVE_SKIP_HIGH_RISK_GRANTS: "1",
      OCLIVE_SKIP_STARTUP_HEALTH: "1",
    },
    stdio: ["ignore", "pipe", "pipe"],
    windowsHide: true,
  });
}

export async function stopProcessTree(child) {
  if (!child || child.exitCode !== null || child.signalCode !== null) return;
  const exited = new Promise((resolveExit) => child.once("exit", resolveExit));
  if (process.platform === "win32") {
    spawnSync("taskkill", ["/PID", String(child.pid), "/T", "/F"], {
      windowsHide: true,
      stdio: "ignore",
    });
  } else {
    child.kill("SIGTERM");
  }
  await Promise.race([
    exited,
    new Promise((resolveWait) => setTimeout(resolveWait, 3_000)),
  ]);
}

export async function waitForKernel(baseUrl, child, stderr) {
  for (let attempt = 0; attempt < 100; attempt += 1) {
    if (child.exitCode !== null) {
      fail(`kernel exited before readiness (${child.exitCode})\n${stderr()}`);
    }
    try {
      const response = await fetch(`${baseUrl}/health`, {
        signal: AbortSignal.timeout(1_000),
      });
      if (response.ok) return;
    } catch {
      // Continue while migrations and the HTTP listener initialize.
    }
    await new Promise((resolveWait) => setTimeout(resolveWait, 150));
  }
  fail(`kernel health timeout\n${stderr()}`);
}

export async function configureRemoteLlm(
  baseUrl,
  apiToken,
  sidecarUrl,
  firstCase,
) {
  const response = await fetch(`${baseUrl}/llm/user_settings`, {
    method: "POST",
    headers: {
      "content-type": "application/json",
      "x-oclive-api-token": apiToken,
    },
    body: JSON.stringify({
      roleId: firstCase.role_id,
      sessionId: `module-quality-${firstCase.id}`,
      provider: "cloud",
      cloudVendor: "module-quality-local",
      cloudApiStyle: "oclive_jsonrpc",
      remoteUrl: sidecarUrl,
      remoteToken: "module-quality-local-only",
      remoteModel: "module-quality-fixture-v1",
      adultContentAcknowledged: false,
    }),
    signal: AbortSignal.timeout(20_000),
  });
  const text = await response.text();
  if (!response.ok) {
    fail(`remote LLM test setup failed (${response.status}): ${text}`);
  }
}

function internalSessionId(roleId, sessionId) {
  const safe = [...sessionId.trim()]
    .map((character) => (/[A-Za-z0-9_-]/u.test(character) ? character : "_"))
    .slice(0, 64)
    .join("");
  return `${roleId}__sess__${safe}`.slice(0, 256);
}

function replayHistoryMessages(testCase) {
  const finalUserIndex = testCase.replay.findLastIndex(
    (turn) => turn.speaker === "user",
  );
  const history = testCase.replay.slice(0, finalUserIndex);
  const messages = [];
  let pendingUser = null;
  let timestamp = Date.UTC(2026, 0, 1);
  for (const turn of history) {
    if (turn.speaker === "user") {
      pendingUser = turn.text;
      continue;
    }
    if (turn.speaker !== "assistant") continue;
    messages.push({
      role: "user",
      content: pendingUser ?? "[module-quality prior context]",
      timestamp,
    });
    timestamp += 1_000;
    messages.push({ role: "assistant", content: turn.text, timestamp });
    timestamp += 1_000;
    pendingUser = null;
  }
  return messages;
}

export async function importReplayHistory(baseUrl, apiToken, testCase) {
  const messages = replayHistoryMessages(testCase);
  if (messages.length === 0) return;
  const sessionId = `module-quality-${testCase.id}`;
  const response = await fetch(`${baseUrl}/chat/storage`, {
    method: "POST",
    headers: {
      "content-type": "application/json",
      "x-oclive-api-token": apiToken,
    },
    body: JSON.stringify({
      op: "import_buckets",
      buckets: [
        {
          role_id: testCase.role_id,
          scene_id: testCase.scene_id,
          session_id: internalSessionId(testCase.role_id, sessionId),
          messages,
        },
      ],
    }),
    signal: AbortSignal.timeout(20_000),
  });
  const text = await response.text();
  if (!response.ok) {
    fail(
      `case ${testCase.id} replay import failed (${response.status}): ${text}`,
    );
  }
}

export async function postChat(
  baseUrl,
  apiToken,
  rolesRoot,
  testCase,
) {
  const finalUserTurn = [...testCase.replay]
    .reverse()
    .find((turn) => turn.speaker === "user");
  assert(finalUserTurn, `case ${testCase.id} has no user turn`);
  const response = await fetch(`${baseUrl}/chat`, {
    method: "POST",
    headers: {
      "content-type": "application/json",
      "x-oclive-api-token": apiToken,
    },
    body: JSON.stringify({
      role_path: join(rolesRoot, testCase.role_id),
      message: finalUserTurn.text,
      session_id: `module-quality-${testCase.id}`,
      scene_id: testCase.scene_id,
    }),
    signal: AbortSignal.timeout(20_000),
  });
  const text = await response.text();
  const responseBody = text ? JSON.parse(text) : null;
  if (!response.ok) {
    fail(
      `case ${testCase.id} chat failed (${response.status}): ${JSON.stringify(responseBody)}`,
    );
  }
  return responseBody;
}
