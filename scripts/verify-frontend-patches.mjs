/**
 * 自检：关键前端改动是否在本仓库落盘。用法：npm run verify:ui
 * 跳过：set OCLIVE_SKIP_VERIFY=1
 */
import { existsSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

if (process.env.OCLIVE_SKIP_VERIFY === "1") {
  console.log("[verify:ui] skipped (OCLIVE_SKIP_VERIFY=1)");
  process.exit(0);
}

const root = join(dirname(fileURLToPath(import.meta.url)), "..");

const checks = [
  {
    name: "PluginBackendSessionPanel.vue exists",
    ok: () => existsSync(join(root, "src/components/PluginBackendSessionPanel.vue")),
  },
  {
    name: "pluginStore has panelMainTab",
    ok: () =>
      readFileSync(join(root, "src/stores/pluginStore.ts"), "utf8").includes("panelMainTab"),
  },
  {
    name: "PluginManagerPanel has V1 professional chrome (插件工作台)",
    ok: () =>
      readFileSync(join(root, "src/views/PluginManagerPanel.vue"), "utf8").includes(
        "插件工作台",
      ),
  },
  {
    name: "App.vue has settings-gear-btn",
    ok: () => readFileSync(join(root, "src/App.vue"), "utf8").includes("settings-gear-btn"),
  },
  {
    name: "App.vue routes plugin manager via openPluginManagerPanel",
    ok: () =>
      readFileSync(join(root, "src/App.vue"), "utf8").includes("openPluginManagerPanel"),
  },
];

let failed = false;
for (const c of checks) {
  const pass = c.ok();
  console[pass ? "log" : "error"](`${pass ? "OK" : "FAIL"}  ${c.name}`);
  if (!pass) failed = true;
}

if (failed) {
  console.error("\n[verify:ui] 未通过。\n");
  process.exit(1);
}
console.log("\n[verify:ui] All checks passed.");
process.exit(0);
