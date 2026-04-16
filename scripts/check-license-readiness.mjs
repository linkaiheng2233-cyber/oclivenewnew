#!/usr/bin/env node
import fs from "node:fs/promises";
import path from "node:path";

const ROOT_LICENSE = "LICENSE";
const MUMU_OFFICIAL_PLUGIN_IDS = [
  "com.oclive.mumu.chat-header-status",
  "com.oclive.mumu.quick-actions",
  "com.oclive.mumu.role-detail-card",
  "com.oclive.mumu.sidebar-glance",
  "com.oclive.mumu.settings-panel",
];

async function exists(filePath) {
  try {
    await fs.access(filePath);
    return true;
  } catch {
    return false;
  }
}

async function main() {
  const cwd = process.cwd();
  const missing = [];

  const rootLicensePath = path.join(cwd, ROOT_LICENSE);
  if (!(await exists(rootLicensePath))) {
    missing.push(ROOT_LICENSE);
  }

  for (const pluginId of MUMU_OFFICIAL_PLUGIN_IDS) {
    const pluginLicense = path.join(cwd, "plugins", pluginId, "LICENSE");
    if (!(await exists(pluginLicense))) {
      missing.push(path.relative(cwd, pluginLicense));
    }
  }

  if (missing.length > 0) {
    console.error("License readiness check failed. Missing files:");
    for (const m of missing) {
      console.error(`- ${m}`);
    }
    process.exitCode = 1;
    return;
  }

  console.log("License readiness check passed.");
  console.log(`Root: ${ROOT_LICENSE}`);
  console.log(`Official plugins checked: ${MUMU_OFFICIAL_PLUGIN_IDS.length}`);
}

void main();
