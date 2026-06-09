import fs from "node:fs";
import path from "node:path";
import { buildPresetFromRolePack } from "./preset_builder.mjs";

/** @type {Map<string, { mtimeMs: number, preset: string }>} */
const cache = new Map();

const TRACKED_FILES = [
  "polish_prompt.md",
  "core_personality.txt",
  "pipeline.ocblueprint",
];

/**
 * @param {string} rolesDir
 * @param {string} roleId
 * @returns {string}
 */
export function getPresetForRole(rolesDir, roleId) {
  const id = roleId?.trim();
  if (!id) {
    return buildPresetFromRolePack(path.join(rolesDir || ".", "_empty"));
  }

  const roleDir = path.join(rolesDir, id);
  const mtimeMs = rolePackMtimeMs(roleDir);
  const cached = cache.get(id);
  if (cached && cached.mtimeMs === mtimeMs) {
    return cached.preset;
  }

  const preset = buildPresetFromRolePack(roleDir);
  cache.set(id, { mtimeMs, preset });
  return preset;
}

/** @param {string} roleDir */
export function rolePackMtimeMs(roleDir) {
  let max = 0;
  for (const name of TRACKED_FILES) {
    const p = path.join(roleDir, name);
    try {
      const st = fs.statSync(p);
      if (st.mtimeMs > max) {
        max = st.mtimeMs;
      }
    } catch {
      // missing file
    }
  }
  return max;
}

/** Test helper */
export function clearPresetCache() {
  cache.clear();
}
