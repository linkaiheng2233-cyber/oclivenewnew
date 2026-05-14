/**
 * Removes a single blank line after our injected missing_errors_doc block
 * to satisfy clippy::empty_line_after_doc_comments / empty_line_after_outer_attr.
 */
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, "..");
const tauriSrc = path.join(root, "src-tauri", "src");

const MARKER = "/// Returns [`Err`] with a human-readable message when the operation fails.";

function fixFile(fp) {
  const lines = fs.readFileSync(fp, "utf8").split(/\r?\n/);
  let changed = false;
  for (let i = 0; i < lines.length - 1; i += 1) {
    if (lines[i] !== MARKER) continue;
    if (lines[i + 1].trim() !== "") continue;
    const nxt = (lines[i + 2] ?? "").trimStart();
    if (
      nxt.startsWith("pub fn ") ||
      nxt.startsWith("pub async fn ") ||
      nxt.startsWith("#[") ||
      nxt.startsWith("///")
    ) {
      lines.splice(i + 1, 1);
      changed = true;
    }
  }
  if (changed) fs.writeFileSync(fp, lines.join("\n"), "utf8");
  return changed;
}

function walk(dir) {
  let n = 0;
  for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, e.name);
    if (e.isDirectory()) n += walk(p);
    else if (e.name.endsWith(".rs") && fixFile(p)) n += 1;
  }
  return n;
}

console.error("fixed files:", walk(tauriSrc));
