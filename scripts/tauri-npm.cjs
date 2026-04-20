/**
 * Tauri beforeDev / beforeBuild：用本文件所在目录定位仓库根（…/scripts → 上一级），
 * 再执行 `npm run dev` 或 `npm run build`。
 *
 * 在仓库根执行 `npm run tauri:dev` 时，进程 cwd 常为项目根，此时不可用 `npm --prefix ..`
 *（会错误解析到盘符根目录，如 D:\\）。
 */
const path = require("path");
const fs = require("fs");
const { spawn, spawnSync } = require("child_process");

const repoRoot = path.join(__dirname, "..");
const script = process.argv[2] === "build" ? "build" : "dev";

function checkTextIncludes(filePath, needles) {
  if (!fs.existsSync(filePath)) return { ok: false, reason: "missing" };
  const text = fs.readFileSync(filePath, "utf8");
  for (const needle of needles) {
    if (!text.includes(needle)) {
      return { ok: false, reason: `missing marker: ${needle}` };
    }
  }
  return { ok: true, reason: "" };
}

function ensureV2Guard() {
  if (script !== "dev") return;
  if (process.env.OCLIVE_DEV_GUARD === "0") return;

  const checks = [
    {
      rel: "src/App.vue",
      needles: [
        "import PluginManagerV2Panel",
        "uiStore.experimentalPluginManagerV2",
        "<PluginManagerV2Panel",
      ],
    },
    {
      rel: "src/views/PluginManagerV2Panel.vue",
      needles: [
        'import { PluginManagerV2 } from "../components/PluginManagerV2"',
        "<PluginManagerV2",
        'aria-label="插件与后端管理 V2"',
      ],
    },
    {
      rel: "src/views/SettingsView.vue",
      needles: ["experimentalPluginManagerV2", "启用新版插件管理界面（V2 预览）"],
    },
    {
      rel: "src/stores/uiStore.ts",
      needles: ["experimentalPluginManagerV2", "setExperimentalPluginManagerV2"],
    },
  ];

  const failures = [];
  for (const item of checks) {
    const abs = path.join(repoRoot, item.rel);
    const res = checkTextIncludes(abs, item.needles);
    if (!res.ok) failures.push(`- ${item.rel}: ${res.reason}`);
  }

  if (failures.length === 0) return;
  console.error("[tauri-npm] V2 guard blocked dev startup.");
  console.error(
    "[tauri-npm] Detected possible local rollback/overwrite in critical files:",
  );
  for (const line of failures) console.error(line);
  console.error(
    "[tauri-npm] Quick recovery: git checkout main && git pull && git restore --source=HEAD -- src/App.vue src/views/PluginManagerV2Panel.vue src/views/SettingsView.vue src/stores/uiStore.ts",
  );
  console.error(
    "[tauri-npm] If you intentionally changed these files, rerun with OCLIVE_DEV_GUARD=0.",
  );
  process.exit(1);
}

function freeVitePortIfBusy() {
  if (script !== "dev") return;
  if (process.env.OCLIVE_DEV_FREE_PORT === "0") return;
  if (process.platform !== "win32") return;

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

ensureV2Guard();
freeVitePortIfBusy();

const child = spawn("npm", ["run", script], {
  cwd: repoRoot,
  stdio: "inherit",
  shell: true,
});
child.on("exit", (code) => process.exit(code ?? 0));
