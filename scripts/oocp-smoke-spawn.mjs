import { spawn } from "node:child_process";
import process from "node:process";
import net from "node:net";

function pickPort() {
  // Prefer explicit env; otherwise pick a semi-random high port.
  const fromEnv =
    process.env.OOCP_API_PORT ||
    process.env.OCLIVE_API_PORT ||
    process.env.PORT;
  if (fromEnv) {
    const p = Number(fromEnv);
    if (Number.isInteger(p) && p > 0 && p < 65536) return p;
  }
  return 49000 + Math.floor(Math.random() * 10000);
}

async function canListen(port) {
  return await new Promise((resolve) => {
    const server = net.createServer();
    server.unref();
    server.once("error", () => resolve(false));
    server.listen({ host: "127.0.0.1", port }, () => {
      server.close(() => resolve(true));
    });
  });
}

async function findUsablePort() {
  const envPort =
    process.env.OOCP_API_PORT || process.env.OCLIVE_API_PORT || process.env.PORT;
  if (envPort) {
    const p = Number(envPort);
    if (Number.isInteger(p) && p > 0 && p < 65536 && (await canListen(p))) return p;
  }
  for (let i = 0; i < 30; i++) {
    const p = pickPort();
    // eslint-disable-next-line no-await-in-loop
    if (await canListen(p)) return p;
  }
  // fallback: best effort
  return pickPort();
}

async function sleep(ms) {
  await new Promise((r) => setTimeout(r, ms));
}

async function waitForHealth({ port, timeoutMs }) {
  const deadline = Date.now() + timeoutMs;
  const url = `http://127.0.0.1:${port}/health`;
  while (Date.now() < deadline) {
    try {
      const res = await fetch(url, { method: "GET" });
      if (res.ok) {
        const text = (await res.text()).trim();
        if (text === "ok") return true;
      }
    } catch {
      // ignore until ready
    }
    await sleep(200);
  }
  return false;
}

function killChild(child) {
  if (!child || child.killed) return;
  try {
    // On Windows, SIGTERM maps to TerminateProcess for node child_process.
    child.kill("SIGTERM");
  } catch {
    // ignore
  }
}

async function main() {
  const port = await findUsablePort();
  const wsUrl = `ws://127.0.0.1:${port}/oocp`;
  const tempDir = process.env.TEMP || process.env.TMPDIR || process.cwd();
  const cargoTargetDir = `${tempDir}\\oclive_oocp_smoke_target_${port}`;

  console.log("");
  console.log(`[spawn-smoke] starting OOCP core on port ${port}`);
  console.log(`[spawn-smoke] ws: ${wsUrl}`);
  console.log(`[spawn-smoke] CARGO_TARGET_DIR: ${cargoTargetDir}`);
  console.log("");

  // Start core (HTTP API + OOCP WS) as a child process.
  // We intentionally run cargo directly to avoid npm script indirection.
  const child = spawn(
    "cargo",
    ["run", "-p", "oclivenewnew-tauri", "--", "--api", "--port", String(port)],
    {
      stdio: "inherit",
      env: {
        ...process.env,
        // Avoid Windows "exe locked" issues when another instance is running.
        CARGO_TARGET_DIR: cargoTargetDir,
        OOCP_API_PORT: String(port),
        OCLIVE_API_PORT: String(port), // backward compat
      },
    },
  );

  const onExit = (code) => {
    console.error(`[spawn-smoke] core exited early (code=${code ?? "null"})`);
  };
  child.once("exit", onExit);

  // First run may compile from scratch; allow generous warmup.
  const ready = await waitForHealth({ port, timeoutMs: 180000 });
  if (!ready) {
    console.error(
      `[FAIL] core did not become healthy in time (http://127.0.0.1:${port}/health)`,
    );
    killChild(child);
    process.exit(1);
  }

  console.log("");
  console.log("[spawn-smoke] core healthy, running smoke...");

  const smokeCmd = `npx tsx tools/oocp-client/examples/smoke.ts "${wsUrl}"`;
  const smoke = spawn(smokeCmd, {
    stdio: "inherit",
    env: process.env,
    shell: true, // Windows-friendly (npx.cmd resolution)
  });

  const smokeExitCode = await new Promise((resolve) => {
    smoke.once("exit", (code) => resolve(code ?? 1));
  });

  console.log("");
  console.log("[spawn-smoke] stopping core...");
  killChild(child);

  // Give the child a moment to exit cleanly.
  await sleep(500);

  process.exit(smokeExitCode);
}

process.on("SIGINT", () => process.exit(130));
process.on("SIGTERM", () => process.exit(143));

main().catch((e) => {
  console.error("[spawn-smoke] unexpected error:", e);
  process.exit(1);
});

