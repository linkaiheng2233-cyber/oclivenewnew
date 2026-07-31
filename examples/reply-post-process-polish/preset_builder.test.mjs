import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { clearPresetCache, getPresetForRole, rolePackMtimeMs } from "./preset_cache.mjs";
import { buildPresetFromRolePack } from "./preset_builder.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROLES_DIR = path.resolve(__dirname, "../../distros/chat-pro/roles");

test("getPresetForRole caches by role pack mtime", () => {
  clearPresetCache();
  const first = getPresetForRole(ROLES_DIR, "枫侵月");
  const second = getPresetForRole(ROLES_DIR, "枫侵月");
  assert.equal(first, second);
  assert.match(first, /枫侵月/);
});

test("rolePackMtimeMs reflects tracked files", () => {
  const mtime = rolePackMtimeMs(path.join(ROLES_DIR, "枫侵月"));
  assert.ok(mtime > 0);
});

test("buildPresetFromRolePack handles missing role dir gracefully", () => {
  const preset = buildPresetFromRolePack(path.join(ROLES_DIR, "__missing_role__"));
  assert.match(preset, /（无人设摘要）/);
});

test("cache invalidates when polish_prompt.md is added", () => {
  const tmpRole = fs.mkdtempSync(path.join(path.dirname(__dirname), "polish-cache-"));
  const roleId = path.basename(tmpRole);
  const rolesRoot = path.dirname(tmpRole);
  try {
    fs.writeFileSync(path.join(tmpRole, "core_personality.txt"), "alpha personality");
    clearPresetCache();
    const before = getPresetForRole(rolesRoot, roleId);
    assert.match(before, /alpha personality/);

    fs.writeFileSync(path.join(tmpRole, "polish_prompt.md"), "override preset");
    clearPresetCache();
    const after = getPresetForRole(rolesRoot, roleId);
    assert.equal(after, "override preset");
  } finally {
    fs.rmSync(tmpRole, { recursive: true, force: true });
  }
});
