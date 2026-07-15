import { spawnSync } from "child_process";
import path from "path";
import process from "process";
import { fileURLToPath } from "url";

const repoRoot = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
  "..",
);
let input = "";
process.stdin.setEncoding("utf8");
for await (const chunk of process.stdin) input += chunk;

const result = spawnSync(
  process.execPath,
  [path.join(repoRoot, "scripts", "cursor-marathon.mjs"), "hook"],
  { cwd: repoRoot, input, encoding: "utf8" },
);

if (result.status !== 0) {
  process.stderr.write(result.stderr || "oclive marathon stop hook failed");
  spawnSync(
    process.execPath,
    [
      path.join(repoRoot, "scripts", "cursor-marathon.mjs"),
      "finish",
      "--outcome",
      "failed",
      "--reason",
      "stop-hook-error",
    ],
    { cwd: repoRoot, encoding: "utf8" },
  );
  process.stdout.write("{}");
  process.exit(0);
}
process.stdout.write(result.stdout || "{}");
