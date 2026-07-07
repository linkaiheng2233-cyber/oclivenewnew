/**
 * Tauri beforeDev / beforeBuild：用本文件所在目录定位仓库根（…/scripts → 上一级），
 * 再执行 `npm run dev` 或 `npm run build`。
 *
 * 在仓库根执行 `npm run tauri:dev` 时，进程 cwd 常为项目根，此时不可用 `npm --prefix ..`
 *（会错误解析到盘符根目录，如 D:\\）。
 */
const path = require("path");
const { spawn, spawnSync } = require("child_process");

const repoRoot = path.join(__dirname, "..");
const script = process.argv[2] === "build" ? "build" : "dev";
const workspace =
  process.env.OCLIVE_TAURI_SHELL === "theater"
    ? "@oclive/theater"
    : "@oclive/chat-pro";

function viteDevProbeOk() {
  const probe = spawnSync(
    process.execPath,
    [path.join(__dirname, "dev-probe.mjs")],
    { encoding: "utf8", shell: false, timeout: 8000 },
  );
  return probe.status === 0;
}

function freeVitePortIfBusy() {
  if (script !== "dev") return;
  if (process.env.OCLIVE_DEV_FREE_PORT === "0") return;
  if (process.platform !== "win32") return;

  if (viteDevProbeOk()) {
    console.info("[tauri-npm] port 1420 already serving dev web — keeping existing Vite");
    return;
  }

  const query = spawnSync(
    "powershell.exe",
    [
      "-NoProfile",
      "-Command",
      "(Get-NetTCPConnection -LocalPort 1420 -State Listen -ErrorAction SilentlyContinue | Select-Object -ExpandProperty OwningProcess -Unique) -join ' '",
    ],
    { encoding: "utf8", shell: false },
  );

  const pidText = (query.stdout || "").trim();
  if (!pidText) return;

  const pids = pidText
    .split(/\s+/)
    .map((s) => Number(s))
    .filter((n) => Number.isInteger(n) && n > 0 && n !== process.pid);

  for (const pid of pids) {
    console.warn(`[tauri-npm] port 1420 busy, stopping PID ${pid}`);
    spawnSync("taskkill", ["/PID", String(pid), "/T", "/F"], {
      stdio: "inherit",
      shell: false,
    });
  }
}

freeVitePortIfBusy();

if (script === "dev" && viteDevProbeOk()) {
  console.info("[tauri-npm] dev web already up on :1420 — skip spawning Vite");
  process.exit(0);
}

const child = spawn("npm", ["run", script, "-w", workspace], {
  cwd: repoRoot,
  stdio: "inherit",
  shell: true,
});
child.on("exit", (code) => process.exit(code ?? 0));
