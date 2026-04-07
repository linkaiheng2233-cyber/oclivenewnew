/**
 * Tauri beforeDev / beforeBuild：用本文件所在目录定位仓库根（…/scripts → 上一级），
 * 再执行 `npm run dev` 或 `npm run build`。
 *
 * 在仓库根执行 `npm run tauri:dev` 时，进程 cwd 常为项目根，此时不可用 `npm --prefix ..`
 *（会错误解析到盘符根目录，如 D:\\）。
 */
const path = require("path");
const { spawn } = require("child_process");

const repoRoot = path.join(__dirname, "..");
const script = process.argv[2] === "build" ? "build" : "dev";

const child = spawn("npm", ["run", script], {
  cwd: repoRoot,
  stdio: "inherit",
  shell: true,
});
child.on("exit", (code) => process.exit(code ?? 0));
