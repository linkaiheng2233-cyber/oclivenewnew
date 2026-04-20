/**
 * 转发到同目录下的 `tauri-npm.cjs`（dev|build）。
 * 在仓库根执行 `npm run tauri:dev` 时，beforeDevCommand 的 cwd 为项目根，应使用：
 *   node scripts/tauri-run.cjs dev
 * 若在 `src-tauri` 下直接 `cargo tauri dev`，cwd 常为 src-tauri，请用：
 *   node ../scripts/tauri-run.cjs dev
 * 或始终在仓库根执行：`npm run tauri:dev`
 */
const path = require("path");
const { spawn } = require("child_process");

const npmScript = path.join(__dirname, "tauri-npm.cjs");
const mode = process.argv[2] === "build" ? "build" : "dev";

const child = spawn(process.execPath, [npmScript, mode], {
  stdio: "inherit",
  // Avoid Windows path splitting issues with "C:\\Program Files\\...\\node.exe".
  shell: false,
});
child.on("exit", (code) => process.exit(code ?? 0));
