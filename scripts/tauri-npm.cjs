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

function buildLlamaDirectoryPluginSidecar() {
  const isWin = process.platform === "win32";
  const profile = script === "build" ? "release" : "debug";
  const fs = require("fs");

  const sidecarManifest = path.join(
    repoRoot,
    "src-tauri",
    "sidecars",
    "oclive-llama-sidecar",
    "Cargo.toml",
  );

  const cargoArgs = ["build", "--manifest-path", sidecarManifest];
  if (profile === "release") cargoArgs.push("--release");

  const build = spawnSync("cargo", cargoArgs, {
    cwd: repoRoot,
    stdio: "inherit",
    shell: false,
    encoding: "utf8",
  });
  if (build.status !== 0) {
    process.exit(build.status ?? 1);
  }

  const targetDir = path.join(
    repoRoot,
    "..",
    "oclive-dev-artifacts",
    "oclivenewnew-cargo-target",
    profile,
  );
  const exeName = isWin ? "oclive-llama-sidecar.exe" : "oclive-llama-sidecar";
  const builtPath = path.join(targetDir, exeName);

  const pluginBinDir = path.join(
    repoRoot,
    "plugins",
    "com.oclive.llama.local",
    "bin",
  );
  const pluginMainPath = path.join(
    pluginBinDir,
    isWin ? "oclive-llama-sidecar.exe" : "oclive-llama-sidecar",
  );
  const pluginCompatPath = path.join(pluginBinDir, "oclive-llama-sidecar");

  try {
    fs.mkdirSync(pluginBinDir, { recursive: true });
    fs.copyFileSync(builtPath, pluginMainPath);
    if (isWin) {
      // Allow manifest `command: bin/oclive-llama-sidecar` to work even if Windows
      // extension resolution behaves differently for relative paths.
      fs.copyFileSync(builtPath, pluginCompatPath);
    }

    // Optional: bundle a prebuilt llama-server binary if provided by env.
    // This keeps the repo lightweight while allowing release packaging to include it.
    const llamaServerSrc = String(process.env.OCLIVE_LLAMA_SERVER_PATH || "").trim();
    if (llamaServerSrc) {
      const llamaServerDest = path.join(
        pluginBinDir,
        isWin ? "llama-server.exe" : "llama-server",
      );
      fs.copyFileSync(llamaServerSrc, llamaServerDest);
      if (isWin) {
        // Also provide no-extension path for relative command invocations.
        fs.copyFileSync(llamaServerSrc, path.join(pluginBinDir, "llama-server"));
      }
    }
  } catch (e) {
    console.error("[tauri-npm] copy llama sidecar failed:", e?.message || e);
    process.exit(1);
  }
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

freeVitePortIfBusy();
buildLlamaDirectoryPluginSidecar();

const child = spawn("npm", ["run", script], {
  cwd: repoRoot,
  stdio: "inherit",
  shell: true,
});
child.on("exit", (code) => process.exit(code ?? 0));
