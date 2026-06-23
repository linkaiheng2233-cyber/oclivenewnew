/**
 * Official Vue / Vitest test runner (directory plugin JSON-RPC sidecar).
 * Ready line: OCLIVE_READY http://127.0.0.1:<port>/rpc
 */
import http from "node:http";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

const PROTOCOL_HEADER = "x-oclive-remote-protocol";
const PROTOCOL_VALUE = "oclive-remote-jsonrpc-v1";

const TEST_GLOBS = [".test.ts", ".test.tsx", ".spec.ts", ".spec.tsx"];

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

function isTestFile(name) {
  const lower = name.toLowerCase();
  return TEST_GLOBS.some((s) => lower.endsWith(s));
}

function walkTestFiles(dir, baseDir, out) {
  let entries;
  try {
    entries = fs.readdirSync(dir, { withFileTypes: true });
  } catch {
    return;
  }
  for (const ent of entries) {
    if (ent.name === "node_modules" || ent.name === "dist" || ent.name === ".git") {
      continue;
    }
    const full = path.join(dir, ent.name);
    if (ent.isDirectory()) {
      walkTestFiles(full, baseDir, out);
      continue;
    }
    if (ent.isFile() && isTestFile(ent.name)) {
      out.push(path.relative(baseDir, full).split(path.sep).join("/"));
    }
  }
}

function resolveWorkspace(params) {
  const cwd = params?.cwd;
  if (typeof cwd !== "string" || !cwd.trim()) {
    throw new Error("params.cwd required");
  }
  const abs = path.resolve(cwd.trim());
  if (!fs.existsSync(abs)) {
    throw new Error(`cwd not found: ${abs}`);
  }
  return abs;
}

function handleHealth(params) {
  const workspace = resolveWorkspace(params);
  const pkg = path.join(workspace, "package.json");
  const hasPkg = fs.existsSync(pkg);
  let vitestInPkg = false;
  if (hasPkg) {
    try {
      const v = JSON.parse(fs.readFileSync(pkg, "utf8"));
      const deps = { ...v.dependencies, ...v.devDependencies };
      vitestInPkg = Boolean(deps.vitest);
    } catch {
      /* ignore */
    }
  }
  return {
    status: "ok",
    workspace,
    packageJson: hasPkg,
    vitestDeclared: vitestInPkg,
    pluginVersion: "0.1.0",
  };
}

function handleListTestFiles(params) {
  const rootRaw = params?.root;
  if (typeof rootRaw !== "string" || !rootRaw.trim()) {
    throw new Error("params.root required");
  }
  const root = path.resolve(rootRaw.trim());
  if (!fs.existsSync(root)) {
    return { files: [], root, warning: "root not found" };
  }
  const files = [];
  walkTestFiles(root, root, files);
  files.sort((a, b) => a.localeCompare(b));
  return { files, root, count: files.length };
}

function parseVitestJsonReport(stdout) {
  try {
    const start = stdout.indexOf("{");
    const end = stdout.lastIndexOf("}");
    if (start < 0 || end <= start) return null;
    return JSON.parse(stdout.slice(start, end + 1));
  } catch {
    return null;
  }
}

function summarizeVitestOutput(stdout, stderr, exitCode, startedAt) {
  const durationMs = Date.now() - startedAt;
  const report = parseVitestJsonReport(stdout);
  if (report && typeof report === "object") {
    const numTotal = Number(report.numTotalTests ?? report.testResults?.length ?? 0);
    const numPassed = Number(report.numPassedTests ?? 0);
    const numFailed = Number(report.numFailedTests ?? 0);
    const failures = [];
    const suites = report.testResults ?? [];
    for (const suite of suites) {
      for (const t of suite.assertionResults ?? []) {
        if (t.status === "failed") {
          failures.push({
            file: suite.name,
            title: t.fullName ?? t.title,
            message: (t.failureMessages ?? []).join("\n").slice(0, 2000),
          });
        }
      }
    }
    return {
      ok: exitCode === 0 && numFailed === 0,
      exitCode,
      durationMs,
      summary: {
        total: numTotal,
        passed: numPassed,
        failed: numFailed,
        passRate: numTotal > 0 ? numPassed / numTotal : exitCode === 0 ? 1 : 0,
      },
      failures: failures.slice(0, 50),
      stdoutTail: stdout.slice(-4000),
      stderrTail: stderr.slice(-2000),
    };
  }
  return {
    ok: exitCode === 0,
    exitCode,
    durationMs,
    summary: {
      total: null,
      passed: null,
      failed: null,
      passRate: exitCode === 0 ? 1 : 0,
    },
    failures: [],
    stdoutTail: stdout.slice(-6000),
    stderrTail: stderr.slice(-3000),
  };
}

function handleRunTest(params) {
  const workspace = resolveWorkspace(params);
  const runAll = Boolean(params?.runAll);
  const specPath = typeof params?.specPath === "string" ? params.specPath.trim() : "";
  const timeoutMs = Math.min(
    Math.max(Number(params?.timeoutMs) || 600_000, 5_000),
    900_000,
  );

  const args = ["vitest", "run", "--reporter=json"];
  if (!runAll) {
    if (!specPath) {
      throw new Error("params.specPath required when runAll is false");
    }
    args.push(specPath);
  }

  const startedAt = Date.now();
  const child = spawnSync("npx", args, {
    cwd: workspace,
    encoding: "utf8",
    shell: process.platform === "win32",
    timeout: timeoutMs,
    env: { ...process.env, CI: "1" },
    maxBuffer: 32 * 1024 * 1024,
  });

  const stdout = child.stdout ?? "";
  const stderr = child.stderr ?? "";
  const exitCode = child.status ?? (child.error ? 1 : 0);
  const result = summarizeVitestOutput(stdout, stderr, exitCode, startedAt);
  result.command = `npx ${args.join(" ")}`;
  result.workspace = workspace;
  if (child.error) {
    result.spawnError = String(child.error.message ?? child.error);
    result.ok = false;
  }
  if (child.signal) {
    result.signal = child.signal;
    result.ok = false;
  }
  return result;
}

const handlers = {
  health: handleHealth,
  list_test_files: handleListTestFiles,
  run_test: handleRunTest,
};

const server = http.createServer(async (req, res) => {
  if (req.method !== "POST") {
    res.writeHead(405);
    res.end();
    return;
  }
  res.setHeader(PROTOCOL_HEADER, PROTOCOL_VALUE);
  let body = "";
  for await (const chunk of req) body += chunk;
  let msg;
  try {
    msg = JSON.parse(body);
  } catch {
    res.writeHead(400);
    res.end(jsonRpcError(null, -32700, "parse error"));
    return;
  }
  const id = msg.id ?? null;
  if (msg.jsonrpc !== "2.0" || typeof msg.method !== "string") {
    res.writeHead(400);
    res.end(jsonRpcError(id, -32600, "invalid request"));
    return;
  }
  const fn = handlers[msg.method];
  if (!fn) {
    res.writeHead(200);
    res.end(jsonRpcError(id, -32601, `method not found: ${msg.method}`));
    return;
  }
  try {
    const result = await fn(msg.params ?? {});
    res.writeHead(200, { "Content-Type": "application/json; charset=utf-8" });
    res.end(jsonRpcResult(id, result));
  } catch (e) {
    res.writeHead(200, { "Content-Type": "application/json; charset=utf-8" });
    res.end(jsonRpcError(id, -32000, e instanceof Error ? e.message : String(e)));
  }
});

server.listen(0, "127.0.0.1", () => {
  const addr = server.address();
  const port = typeof addr === "object" && addr ? addr.port : 0;
  process.stdout.write(`OCLIVE_READY http://127.0.0.1:${port}/rpc\n`);
});

process.on("SIGTERM", () => server.close());
process.on("SIGINT", () => server.close());
