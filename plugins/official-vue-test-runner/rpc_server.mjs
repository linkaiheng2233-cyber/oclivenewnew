/**
 * Official Vue test runner — JSON-RPC sidecar (directory plugin).
 * Methods: echo.ping, health, list_test_files, run_test
 */
import http from "node:http";
import fs from "node:fs";
import path from "node:path";
import os from "node:os";
import { spawn, spawnSync } from "node:child_process";

const PROTOCOL_HEADER = "x-oclive-remote-protocol";
const PROTOCOL_VALUE = "oclive-remote-jsonrpc-v1";

function jsonRpcResult(id, result) {
  return JSON.stringify({ jsonrpc: "2.0", id, result });
}

function jsonRpcError(id, code, message) {
  return JSON.stringify({
    jsonrpc: "2.0",
    id,
    error: { code, message },
  });
}

function useShell() {
  return process.platform === "win32";
}

function npxCmd() {
  return process.platform === "win32" ? "npx.cmd" : "npx";
}

function walkTestFiles(rootDir, out, depth = 0) {
  if (depth > 40) return;
  let stat;
  try {
    stat = fs.statSync(rootDir);
  } catch {
    return;
  }
  if (!stat.isDirectory()) return;
  let names;
  try {
    names = fs.readdirSync(rootDir);
  } catch {
    return;
  }
  for (const name of names) {
    if (name === "node_modules" || name === "dist" || name === ".git") continue;
    const full = path.join(rootDir, name);
    let st;
    try {
      st = fs.statSync(full);
    } catch {
      continue;
    }
    if (st.isDirectory()) {
      walkTestFiles(full, out, depth + 1);
    } else if (st.isFile() && (name.endsWith(".spec.ts") || name.endsWith(".test.ts"))) {
      out.push(full);
    }
  }
}

function handleEchoPing(params) {
  const text = params && params.text != null ? String(params.text) : "";
  return { pong: true, text, plugin: "com.oclive.official_vue_test_runner" };
}

function handleHealth(params) {
  const cwd = params && params.cwd != null ? String(params.cwd).trim() : "";
  const r = spawnSync(npxCmd(), ["vitest", "--version"], {
    cwd: cwd || process.cwd(),
    shell: useShell(),
    encoding: "utf8",
    timeout: 20_000,
    env: { ...process.env, CI: "1" },
  });
  const vitestLine = (r.stdout || "").trim().split("\n")[0] || "";
  const ok = r.status === 0 && vitestLine.length > 0;
  return {
    ok,
    vitest: ok ? vitestLine : null,
    node: process.version,
    platform: process.platform,
    cwd: cwd || process.cwd(),
    stderrTail: (r.stderr || "").trim().slice(0, 500),
  };
}

function handleListTestFiles(params) {
  const root = params && params.root != null ? String(params.root).trim() : "";
  const base = root || process.cwd();
  const out = [];
  walkTestFiles(path.resolve(base), out);
  out.sort((a, b) => a.localeCompare(b));
  return { root: base, files: out };
}

function runNpxVitestJson(vitestArgs, cwd, timeoutMs) {
  return new Promise((resolve) => {
    const outfile = path.join(
      os.tmpdir(),
      `vitest-out-${Date.now()}-${Math.random().toString(36).slice(2, 9)}.json`,
    );
    const child = spawn(npxCmd(), ["vitest", ...vitestArgs, "--reporter=json", `--outputFile=${outfile}`], {
      cwd,
      shell: useShell(),
      env: { ...process.env, CI: "1", FORCE_COLOR: "0" },
    });
    let stderr = "";
    child.stderr?.on("data", (c) => {
      stderr += String(c);
    });
    const timer = setTimeout(() => {
      try {
        child.kill("SIGTERM");
      } catch {
        /* ignore */
      }
    }, timeoutMs);
    child.on("close", (code) => {
      clearTimeout(timer);
      let raw = "";
      try {
        raw = fs.readFileSync(outfile, "utf8");
      } catch {
        raw = "";
      }
      try {
        fs.unlinkSync(outfile);
      } catch {
        /* ignore */
      }
      let report = null;
      try {
        report = raw ? JSON.parse(raw) : null;
      } catch {
        report = null;
      }
      resolve({
        exitCode: code ?? -1,
        stderrTail: stderr.trim().slice(0, 4000),
        report,
        rawReportLength: raw.length,
      });
    });
  });
}

async function handleRunTest(params) {
  const cwd = params && params.cwd != null ? String(params.cwd).trim() : "";
  const specPath = params && params.specPath != null ? String(params.specPath).trim() : "";
  const runAll = params && params.runAll === true;
  const base = cwd || process.cwd();
  const timeoutMs =
    params && typeof params.timeoutMs === "number" && Number.isFinite(params.timeoutMs)
      ? Math.min(Math.max(params.timeoutMs, 5000), 900_000)
      : 300_000;

  const vitestArgs = ["run"];
  if (!runAll && specPath) {
    vitestArgs.push(specPath);
  }

  const started = Date.now();
  const exec = await runNpxVitestJson(vitestArgs, path.resolve(base), timeoutMs);
  const durationMs = Date.now() - started;

  const rep = exec.report;
  let passed = 0;
  let failed = 0;
  let total = 0;
  const failures = [];
  if (rep && typeof rep === "object") {
    if (typeof rep.numPassedTests === "number") passed = rep.numPassedTests;
    if (typeof rep.numFailedTests === "number") failed = rep.numFailedTests;
    if (typeof rep.numTotalTests === "number") total = rep.numTotalTests;
    const testResults = Array.isArray(rep.testResults) ? rep.testResults : [];
    for (const tr of testResults) {
      const asserts = Array.isArray(tr.assertionResults) ? tr.assertionResults : [];
      for (const ar of asserts) {
        if (ar.status === "failed") {
          failures.push({
            file: tr.name || "",
            title: ar.title || "",
            messages: ar.failureMessages || [],
          });
        }
      }
    }
  }

  return {
    cwd: path.resolve(base),
    specPath: runAll ? null : specPath || null,
    runAll,
    exitCode: exec.exitCode,
    durationMs,
    summary: { passed, failed, total },
    failures,
    stderrTail: exec.stderrTail,
    rawReportPresent: exec.rawReportLength > 0,
  };
}

const server = http.createServer((req, res) => {
  if (req.method !== "POST" || !req.url || !req.url.startsWith("/rpc")) {
    res.writeHead(404, { "Content-Type": "text/plain; charset=utf-8" });
    res.end("not found");
    return;
  }
  const chunks = [];
  req.on("data", (c) => chunks.push(c));
  req.on("end", async () => {
    const raw = Buffer.concat(chunks).toString("utf8");
    let msg;
    try {
      msg = JSON.parse(raw);
    } catch {
      res.writeHead(400, { "Content-Type": "application/json; charset=utf-8" });
      res.end(jsonRpcError(null, -32700, "parse error"));
      return;
    }
    const id = msg.id ?? null;
    if (msg.jsonrpc !== "2.0" || typeof msg.method !== "string") {
      res.writeHead(400, { "Content-Type": "application/json; charset=utf-8" });
      res.end(jsonRpcError(id, -32600, "invalid request"));
      return;
    }
    res.setHeader("Content-Type", "application/json; charset=utf-8");
    res.setHeader(PROTOCOL_HEADER, PROTOCOL_VALUE);

    try {
      if (msg.method === "echo.ping") {
        res.writeHead(200);
        res.end(jsonRpcResult(id, handleEchoPing(msg.params)));
        return;
      }
      if (msg.method === "health") {
        res.writeHead(200);
        res.end(jsonRpcResult(id, handleHealth(msg.params)));
        return;
      }
      if (msg.method === "list_test_files") {
        res.writeHead(200);
        res.end(jsonRpcResult(id, handleListTestFiles(msg.params)));
        return;
      }
      if (msg.method === "run_test") {
        const out = await handleRunTest(msg.params);
        res.writeHead(200);
        res.end(jsonRpcResult(id, out));
        return;
      }
      res.writeHead(200);
      res.end(jsonRpcError(id, -32601, `method not found: ${msg.method}`));
    } catch (e) {
      const m = e instanceof Error ? e.message : String(e);
      res.writeHead(200);
      res.end(jsonRpcError(id, -32603, m));
    }
  });
});

server.listen(0, "127.0.0.1", () => {
  const addr = server.address();
  const port = typeof addr === "object" && addr ? addr.port : 0;
  const url = `http://127.0.0.1:${port}/rpc`;
  process.stdout.write(`OCLIVE_READY ${url}\n`);
});

process.on("SIGTERM", () => server.close());
process.on("SIGINT", () => server.close());
