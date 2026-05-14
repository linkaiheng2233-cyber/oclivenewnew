/**
 * Inserts minimal `/// # Errors` blocks for clippy missing_errors_doc (oclivenewnew-tauri).
 *
 * Usage: node scripts/add-missing-errors-doc-tauri.mjs tauri_pedantic1.txt
 */
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, "..");
const tauriSrc = path.join(root, "src-tauri", "src");

const ERR_LINES = [
  "/// # Errors",
  "///",
  "/// Returns [`Err`] with a human-readable message when the operation fails.",
];

function parseLocations(text) {
  const locs = [];
  const re = /-->\s+src-tauri\\src\\([^:\r\n]+):(\d+):/g;
  let m;
  while ((m = re.exec(text)) !== null) {
    const rel = m[1].replace(/\\/g, "/");
    const line = Number(m[2], 10);
    if (!Number.isFinite(line)) continue;
    locs.push({ rel, line });
  }
  return locs;
}

/** @returns {number} insert position (0-based line index) or -1 */
function computeInsertPos(lines, fnLine0) {
  if (fnLine0 < 0 || fnLine0 >= lines.length) return -1;
  const ln = lines[fnLine0].trimStart();
  if (!ln.startsWith("pub fn ") && !ln.startsWith("pub async fn ")) {
    return -1;
  }
  let k = fnLine0 - 1;
  while (k >= 0) {
    const t = lines[k].trim();
    if (t === "") {
      k -= 1;
      continue;
    }
    if (t.startsWith("#[") || t.startsWith("#![") || t.startsWith("///")) {
      k -= 1;
      continue;
    }
    break;
  }
  return k + 1;
}

function regionHasErrors(lines, from, to) {
  for (let i = from; i < to && i < lines.length; i += 1) {
    if (lines[i].includes("# Errors")) return true;
  }
  return false;
}

function signatureHasResult(lines, fnLine0) {
  let i = fnLine0;
  const max = Math.min(lines.length, fnLine0 + 25);
  let buf = "";
  while (i < max) {
    buf += lines[i];
    if (buf.includes("{")) break;
    i += 1;
  }
  return buf.includes("Result");
}

function main() {
  const inFile = process.argv[2];
  if (!inFile) {
    console.error("usage: node scripts/add-missing-errors-doc-tauri.mjs <clippy-log.txt>");
    process.exit(1);
  }
  const text = fs.readFileSync(path.resolve(inFile), "utf8");
  const locs = parseLocations(text);
  /** @type {Map<string, number[]>} */
  const byFile = new Map();
  for (const { rel, line } of locs) {
    const arr = byFile.get(rel) ?? [];
    arr.push(line);
    byFile.set(rel, arr);
  }
  let fileCount = 0;
  let editCount = 0;
  for (const [rel, lineNums] of byFile) {
    const fp = path.join(tauriSrc, ...rel.split("/"));
    if (!fs.existsSync(fp)) {
      console.error("missing file", fp);
      continue;
    }
    const lines = fs.readFileSync(fp, "utf8").split(/\r?\n/);
    /** @type {{origInsert: number, fnLine0: number}[]} */
    const jobs = [];
    for (const line1 of [...new Set(lineNums)]) {
      const fnLine0 = line1 - 1;
      const origInsert = computeInsertPos(lines, fnLine0);
      if (origInsert < 0) continue;
      if (!signatureHasResult(lines, fnLine0)) continue;
      if (regionHasErrors(lines, origInsert, fnLine0)) continue;
      jobs.push({ origInsert, fnLine0 });
    }
    jobs.sort((a, b) => a.origInsert - b.origInsert);
    const byInsert = new Map();
    for (const j of jobs) {
      byInsert.set(j.origInsert, j);
    }
    const uniqJobs = [...byInsert.values()].sort((a, b) => a.origInsert - b.origInsert);
    let offset = 0;
    let changed = false;
    for (const { origInsert, fnLine0 } of uniqJobs) {
      const insertAt = origInsert + offset;
      if (regionHasErrors(lines, insertAt, fnLine0 + offset)) continue;
      lines.splice(insertAt, 0, ...ERR_LINES);
      offset += ERR_LINES.length;
      editCount += 1;
      changed = true;
    }
    if (changed) {
      fs.writeFileSync(fp, lines.join("\n"), "utf8");
      fileCount += 1;
    }
  }
  console.error(`inserted ${editCount} doc blocks in ${fileCount} files`);
}

main();
