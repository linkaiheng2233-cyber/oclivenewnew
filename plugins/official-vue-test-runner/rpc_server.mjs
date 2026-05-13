/**
 * Official Vue test runner — JSON-RPC sidecar (directory plugin).
 * Methods: echo.ping, health, list_test_files, run_test, get_history, clear_history
 */
import http from "node:http";
import fs from "node:fs";
import path from "node:path";
import os from "node:os";
import { fileURLToPath } from "node:url";
import { spawn, spawnSync } from "node:child_process";

const PROTOCOL_HEADER = "x-oclive-remote-protocol";
const PROTOCOL_VALUE = "oclive-remote-jsonrpc-v1";

const PLUGIN_ROOT = path.dirname(fileURLToPath(import.meta.url));
const HISTORY_PATH = path.join(PLUGIN_ROOT, "test_history.json");
const HISTORY_CAP = 200;

function readHistoryFile() {
  try {
    const raw = fs.readFileSync(HISTORY_PATH, "utf8");
    const j = JSON.parse(raw);
    if (!j || typeof j !== "object") return { version: 1, runs: [] };
    if (!Array.isArray(j.runs)) j.runs = [];
    return j;
  } catch {
    return { version: 1, runs: [] };
  }
}

function writeHistoryFile(data) {
  fs.writeFileSync(HISTORY_PATH, JSON.stringify(data, null, 2), "utf8");
}

function appendRunHistory(entry) {
  const data = readHistoryFile();
  data.version = 1;
  data.runs.unshift(entry);
  if (data.runs.length > HISTORY_CAP) {
    data.runs.length = HISTORY_CAP;
  }
  writeHistoryFile(data);
}

function handleGetHistory(params) {
  let limit = 20;
  if (params && typeof params.limit === "number" && Number.isFinite(params.limit)) {
    limit = Math.min(100, Math.max(1, Math.floor(params.limit)));
  }
  const data = readHistoryFile();
  return {
    limit,
    totalStored: data.runs.length,
    runs: data.runs.slice(0, limit),
  };
}

function handleClearHistory() {
  writeHistoryFile({ version: 1, runs: [] });
  return { cleared: true };
}

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

/** Pull file:line:col from Vitest / Jest stack text. */
function extractPrimaryLocation(text) {
  if (!text || typeof text !== "string") return null;
  const re = /(?:^|\n|\r)\s*at\s+.*?\(([^():]+\.(?:m?[jt]sx?|vue|cjs|mjs)):(\d+):(\d+)\)/;
  const m = text.match(re);
  if (m) {
    return { file: m[1], line: Number(m[2]), column: Number(m[3]) };
  }
  const re2 = /([A-Za-z]:[^:\s"']+\.(?:m?[jt]sx?|vue)):(\d+):(\d+)/;
  const m2 = text.match(re2);
  if (m2) {
    return { file: m2[1], line: Number(m2[2]), column: Number(m2[3]) };
  }
  return null;
}

/**
 * Vitest JSON → unified test output (see creator-docs/testing/TEST_OUTPUT_SCHEMA.md).
 */
function buildStructuredRunResult({
  report,
  exitCode,
  durationMs,
  cwd,
  runAll,
  specPath,
  rawReportLength,
}) {
  const rep = report && typeof report === "object" ? report : null;

  const numPassedTests = rep && typeof rep.numPassedTests === "number" ? rep.numPassedTests : 0;
  const numFailedTests = rep && typeof rep.numFailedTests === "number" ? rep.numFailedTests : 0;
  const numPendingTests = rep && typeof rep.numPendingTests === "number" ? rep.numPendingTests : 0;
  const numTotalTests = rep && typeof rep.numTotalTests === "number" ? rep.numTotalTests : 0;

  const numPassedSuites =
    rep && typeof rep.numPassedTestSuites === "number" ? rep.numPassedTestSuites : 0;
  const numFailedSuites =
    rep && typeof rep.numFailedTestSuites === "number" ? rep.numFailedTestSuites : 0;
  const numTotalSuites =
    rep && typeof rep.numTotalTestSuites === "number" ? rep.numTotalTestSuites : 0;

  const vitestReportedSuccess = rep && rep.success === true;
  const passRate =
    numTotalTests > 0 ? Math.round((numPassedTests / numTotalTests) * 10_000) / 10_000 : null;

  const failuresOut = [];
  const suitesOut = [];
  const testResults = rep && Array.isArray(rep.testResults) ? rep.testResults : [];

  for (const tr of testResults) {
    const suiteFile = typeof tr.name === "string" ? tr.name : "";
    const asserts = Array.isArray(tr.assertionResults) ? tr.assertionResults : [];
    let sp = 0;
    let sf = 0;
    let ssk = 0;
    for (const ar of asserts) {
      if (ar.status === "passed") sp += 1;
      else if (ar.status === "failed") sf += 1;
      else ssk += 1;
    }
    let suiteMs = null;
    if (typeof tr.endTime === "number" && typeof tr.startTime === "number") {
      suiteMs = Math.max(0, tr.endTime - tr.startTime);
    }
    suitesOut.push({
      id: suiteFile || "(suite)",
      name: suiteFile || "(suite)",
      passed: sp,
      failed: sf,
      skipped: ssk,
      durationMs: suiteMs,
    });

    for (const ar of asserts) {
      if (ar.status !== "failed") continue;
      const title = typeof ar.title === "string" ? ar.title : "";
      const fullName = typeof ar.fullName === "string" ? ar.fullName : title;
      const messages = Array.isArray(ar.failureMessages)
        ? ar.failureMessages.map((x) => String(x))
        : [];
      const primaryMsg = messages[0] || "";
      let loc =
        ar.location && typeof ar.location === "object"
          ? {
              file:
                typeof ar.location.path === "string"
                  ? ar.location.path
                  : typeof ar.location.file === "string"
                    ? ar.location.file
                    : suiteFile,
              line:
                typeof ar.location.line === "number"
                  ? ar.location.line
                  : typeof ar.location.lineNumber === "number"
                    ? ar.location.lineNumber
                    : null,
              column:
                typeof ar.location.column === "number"
                  ? ar.location.column
                  : typeof ar.location.columnNumber === "number"
                    ? ar.location.columnNumber
                    : null,
            }
          : null;
      if (!loc || loc.line == null) {
        const fromStack = extractPrimaryLocation(primaryMsg);
        if (fromStack) {
          loc = fromStack;
        } else if (suiteFile) {
          loc = { file: suiteFile, line: null, column: null };
        }
      }
      failuresOut.push({
        file: loc?.file || suiteFile || "(unknown file)",
        line: loc?.line ?? null,
        column: loc?.column ?? null,
        message: primaryMsg,
        expected: null,
        actual: null,
        suiteTitle: suiteFile || null,
        testTitle: title || null,
        fullName: fullName || null,
        messages,
      });
    }
  }

  const runOk =
    exitCode === 0 &&
    (vitestReportedSuccess || (rep == null && numFailedTests === 0 && numTotalTests === 0));

  const headline =
    numTotalTests > 0
      ? `${numPassedTests} passed, ${numFailedTests} failed, ${numPendingTests} skipped / ${numTotalTests} tests · ${durationMs} ms`
      : rep == null
        ? `No JSON report parsed (exit ${exitCode}) · ${durationMs} ms`
        : `0 tests in report · ${durationMs} ms`;

  return {
    schemaVersion: 1,
    kind: "oclive.unit_test_run.v1",
    summary: {
      passed: numPassedTests,
      failed: numFailedTests,
      skipped: numPendingTests,
      total: numTotalTests,
      passRate,
      durationMs,
      exitCode,
      ok: runOk,
    },
    suites: suitesOut,
    suiteTotals: {
      passed: numPassedSuites,
      failed: numFailedSuites,
      total: numTotalSuites,
    },
    failures: failuresOut,
    meta: {
      headline,
      cwd,
      scope: runAll ? "all" : specPath || "all",
      runner: "vitest",
      vitestSuccessFlag: vitestReportedSuccess,
      rawReportPresent: rawReportLength > 0,
    },
  };
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

  const resolvedCwd = path.resolve(base);
  const structured = buildStructuredRunResult({
    report: rep,
    exitCode: exec.exitCode,
    durationMs,
    cwd: resolvedCwd,
    runAll,
    specPath,
    rawReportLength: exec.rawReportLength,
  });

  appendRunHistory({
    at: new Date().toISOString(),
    durationMs,
    exitCode: exec.exitCode,
    runOk: structured.summary.ok,
    passRate: structured.summary.passRate,
    passed: structured.summary.passed,
    failed: structured.summary.failed,
    total: structured.summary.total,
    runAll,
    specPath: runAll ? null : specPath || null,
    cwd: resolvedCwd,
    failureCount: structured.failures.length,
  });

  return {
    cwd: resolvedCwd,
    specPath: runAll ? null : specPath || null,
    runAll,
    exitCode: exec.exitCode,
    durationMs,
    summary: { passed, failed, total },
    failures,
    stderrTail: exec.stderrTail,
    rawReportPresent: exec.rawReportLength > 0,
    structured,
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
      if (msg.method === "get_history") {
        res.writeHead(200);
        res.end(jsonRpcResult(id, handleGetHistory(msg.params)));
        return;
      }
      if (msg.method === "clear_history") {
        res.writeHead(200);
        res.end(jsonRpcResult(id, handleClearHistory()));
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
