import fs from "fs";
import path from "path";

function walk(dir, prefix = "") {
  const out = [];
  for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
    const rel = prefix ? `${prefix}/${e.name}` : e.name;
    const full = path.join(dir, e.name);
    if (e.isDirectory()) out.push(...walk(full, rel));
    else if (e.name.endsWith(".md")) out.push(rel);
  }
  return out;
}

const zh = walk("creator-docs").sort();
const en = walk("creator-docs-en").sort();
const zhSet = new Set(zh);
const enSet = new Set(en);
const onlyZh = [...zhSet].filter((x) => !enSet.has(x));
const onlyEn = [...enSet].filter((x) => !zhSet.has(x));
console.log(JSON.stringify({ zh: zh.length, en: en.length, onlyZh, onlyEn }, null, 2));
