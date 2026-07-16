import { spawnSync } from "child_process";
import fs from "fs";
import path from "path";
import process from "process";
import { fileURLToPath } from "url";

const repoRoot = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
  "..",
);
const hookLogPath = path.join(repoRoot, ".cursor", "oclive-marathon-hook.log");

function appendLog(line) {
  try {
    fs.mkdirSync(path.dirname(hookLogPath), { recursive: true });
    fs.appendFileSync(hookLogPath, `${new Date().toISOString()} ${line}\n`, "utf8");
  } catch {
    // best-effort
  }
}

let input = "";
process.stdin.setEncoding("utf8");
for await (const chunk of process.stdin) input += chunk;

const result = spawnSync(
  process.execPath,
  [path.join(repoRoot, "scripts", "cursor-marathon.mjs"), "hook"],
  { cwd: repoRoot, input, encoding: "utf8" },
);

if (result.status !== 0) {
  const detail = (result.stderr || result.stdout || "oclive marathon stop hook failed")
    .toString()
    .trim()
    .slice(0, 500);
  process.stderr.write(detail ? `${detail}\n` : "oclive marathon stop hook failed\n");
  appendLog(`fail-open status=${result.status} detail=${detail}`);
  // Fail open: never finish/kill the marathon session on a transient hook error.
  // Killing here was ending overnight runs after the first stop.
  process.stdout.write("{}");
  process.exit(0);
}

const stdout = (result.stdout || "{}").toString().trim() || "{}";
process.stdout.write(stdout);
