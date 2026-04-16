#!/usr/bin/env node
import fs from "node:fs/promises";
import path from "node:path";

const SLOT_OPTIONS = new Set([
  "chat_toolbar",
  "settings.panel",
  "role.detail",
  "sidebar",
  "chat.header",
]);

function readArg(name, fallback = "") {
  const key = `--${name}`;
  const idx = process.argv.indexOf(key);
  if (idx < 0) return fallback;
  return String(process.argv[idx + 1] ?? "").trim();
}

function printUsage() {
  console.log(
    [
      "Usage:",
      "  node scripts/scaffold-ui-slot-plugin.mjs --id com.example.plugin --slot role.detail [--title \"My Card\"]",
      "",
      "Slots:",
      "  chat_toolbar | settings.panel | role.detail | sidebar | chat.header",
    ].join("\n"),
  );
}

async function writeFileSafe(filePath, content) {
  await fs.mkdir(path.dirname(filePath), { recursive: true });
  await fs.writeFile(filePath, content, "utf8");
}

function buildManifest(pluginId, slot, htmlEntry, vueEntry) {
  return `${JSON.stringify(
    {
      schema_version: 1,
      id: pluginId,
      version: "0.1.0",
      ui_slots: [
        {
          slot,
          entry: htmlEntry,
          vueComponent: vueEntry,
          bridge: {
            invoke: ["list_roles", "get_role_info"],
            events: ["role:switched", "message:sent", "theme:changed"],
          },
        },
      ],
    },
    null,
    2,
  )}\n`;
}

function buildHtml(title) {
  return `<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>${title}</title>
    <style>
      :root { font-family: "Segoe UI", system-ui, sans-serif; }
      body { margin: 0; padding: 12px; font-size: 13px; line-height: 1.45; }
      .card { border: 1px solid #d9d9d9; border-radius: 10px; padding: 10px; }
    </style>
  </head>
  <body>
    <div class="card">iframe fallback is active for ${title}</div>
  </body>
</html>
`;
}

function buildVue(title) {
  return `<script setup lang="ts">
import { inject } from "vue";

type OcliveApi = {
  invoke(command: string, params?: unknown): Promise<unknown>;
};

const oclive = inject<OcliveApi | null>("oclive", null);

async function ping(): Promise<void> {
  if (!oclive) return;
  await oclive.invoke("list_roles", {});
}
</script>

<template>
  <section class="card" aria-label="${title}">
    <h3>${title}</h3>
    <p>Scaffolded slot plugin component.</p>
    <button type="button" class="btn" @click="ping">Ping host API</button>
  </section>
</template>

<style scoped>
.card {
  width: 100%;
  box-sizing: border-box;
  padding: 12px;
  border-radius: 14px;
  border: 1px solid color-mix(in srgb, var(--border-light, #ddd) 72%, transparent);
  background: color-mix(in srgb, var(--bg-primary, #fff) 90%, transparent);
}
h3 {
  margin: 0;
  font-size: 14px;
}
p {
  margin: 6px 0 10px;
  font-size: 12px;
  color: var(--text-secondary, #666);
}
.btn {
  min-height: 30px;
  border-radius: 10px;
  border: 1px solid color-mix(in srgb, var(--border-light, #ddd) 75%, transparent);
  background: var(--bg-elevated, #f6f6f6);
  color: var(--text-primary, #333);
  padding: 4px 10px;
  cursor: pointer;
}
</style>
`;
}

async function main() {
  const pluginId = readArg("id");
  const slot = readArg("slot");
  const title = readArg("title", "UI Slot Template");

  if (!pluginId || !slot) {
    printUsage();
    process.exitCode = 1;
    return;
  }
  if (!SLOT_OPTIONS.has(slot)) {
    console.error(`Invalid slot: ${slot}`);
    printUsage();
    process.exitCode = 1;
    return;
  }

  const root = process.cwd();
  const pluginRoot = path.join(root, "plugins", pluginId);
  const slotsDir = path.join(pluginRoot, "slots");
  const htmlName = "slot.html";
  const vueName = "SlotCard.vue";
  const htmlRel = `slots/${htmlName}`;
  const vueRel = `slots/${vueName}`;

  try {
    await fs.access(pluginRoot);
    console.error(`Plugin already exists: ${pluginRoot}`);
    process.exitCode = 1;
    return;
  } catch {
    // directory does not exist, proceed
  }

  await fs.mkdir(slotsDir, { recursive: true });
  await writeFileSafe(
    path.join(pluginRoot, "manifest.json"),
    buildManifest(pluginId, slot, htmlRel, vueRel),
  );
  await writeFileSafe(path.join(slotsDir, htmlName), buildHtml(title));
  await writeFileSafe(path.join(slotsDir, vueName), buildVue(title));

  console.log(`Created plugin scaffold at: ${pluginRoot}`);
  console.log("Next steps:");
  console.log("1) Fill in SlotCard.vue UI logic");
  console.log("2) Add plugin id into roles/<role>/ui.json slot order/visible");
  console.log("3) Restart app or refresh plugin catalog");
}

void main();
