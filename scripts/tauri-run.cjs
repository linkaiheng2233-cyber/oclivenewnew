/**
 * 转发到同目录下的 `tauri-npm.cjs`（dev|build）。
 * 在仓库根执行 `npm run tauri:dev` 时，Tauri beforeDevCommand 的 cwd 为 `distros/`，应使用：
 *   node ../scripts/tauri-run.cjs dev
 */
const fs = require("fs");
const path = require("path");
const { spawnSync, spawn } = require("child_process");

const repoRoot = path.join(__dirname, "..");
const confPath = path.join(repoRoot, "distros", "desktop-tauri", "tauri.conf.json");
const npmScript = path.join(__dirname, "tauri-npm.cjs");
const mode = process.argv[2] === "build" ? "build" : "dev";

const originalConf = fs.readFileSync(confPath, "utf8");

spawnSync(process.execPath, [path.join(__dirname, "tauri-shell-dist.mjs")], {
  stdio: "inherit",
  cwd: repoRoot,
});

const child = spawn(process.execPath, [npmScript, mode], {
  stdio: "inherit",
  shell: false,
});
child.on("exit", (code) => {
  fs.writeFileSync(confPath, originalConf, "utf8");
  process.exit(code ?? 0);
});