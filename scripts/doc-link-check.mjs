import fs from "fs";
import path from "path";

const roots = ["creator-docs", "creator-docs-en"];
const keyFiles = [
  "getting-started/DOCUMENTATION_INDEX.md",
  "getting-started/PROJECT_OVERVIEW.md",
  "getting-started/KERNEL_AND_MODULES_ARCHITECTURE.md",
  "role-pack/CREATOR_LEARNING_PATH.md",
  "plugin-and-architecture/PLUGIN_AUTHOR_LEARNING_PATH.md",
  "role-pack/V1_TO_V2_MIGRATION.md",
  "testing/TESTING_GUIDE.md",
];

const linkRe = /\[[^\]]+\]\(([^)]+)\)/g;

function checkFile(filePath, repoRoot) {
  const text = fs.readFileSync(filePath, "utf8");
  const dir = path.dirname(filePath);
  const broken = [];
  let m;
  while ((m = linkRe.exec(text)) !== null) {
    let target = m[1].trim();
    if (target.startsWith("http") || target.startsWith("#") || target.startsWith("mailto:"))
      continue;
    const hash = target.includes("#") ? target.slice(target.indexOf("#")) : "";
    target = target.split("#")[0].split("?")[0];
    if (!target || target.endsWith(".md") === false && !target.includes(".")) {
      // allow repo-relative without extension sometimes
    }
    const resolved = path.normalize(path.join(dir, target));
    if (!fs.existsSync(resolved)) {
      broken.push({ link: m[1], resolved });
    }
  }
  return broken;
}

const allBroken = [];
for (const root of roots) {
  for (const rel of keyFiles) {
    const fp = path.join(root, rel);
    if (!fs.existsSync(fp)) {
      allBroken.push({ file: fp, error: "missing file" });
      continue;
    }
    const broken = checkFile(fp, root);
    for (const b of broken) allBroken.push({ file: fp, ...b });
  }
}

if (allBroken.length) {
  console.error(JSON.stringify(allBroken, null, 2));
  process.exit(1);
}
console.log("OK: no broken relative .md links in key files");
