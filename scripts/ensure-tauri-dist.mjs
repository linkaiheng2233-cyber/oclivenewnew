#!/usr/bin/env node
/**
 * Tauri `generate_context!()` requires `build.distDir` to exist at compile time.
 * When only running `cargo check` / `check:rust` (without `npm run build`), create a
 * minimal stub `index.html` so the proc-macro does not panic.
 *
 * Boundary: stub is compile-time gate only — it does **not** replace `npm run build` for
 * release, Playwright, or loom/native E2E. Does not overwrite an existing frontend build output.
 * Generated paths live under `dist/` (gitignored).
 */
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const tauriRoot = path.join(repoRoot, 'distros', 'desktop-tauri');
const confPath = path.join(tauriRoot, 'tauri.conf.json');

if (!fs.existsSync(confPath)) {
  console.error(`[ensure-tauri-dist] missing ${confPath}`);
  process.exit(1);
}

const conf = JSON.parse(fs.readFileSync(confPath, 'utf8'));
const distRel = conf.build?.frontendDist ?? conf.build?.distDir ?? '../chat-pro/dist';
const distAbs = path.resolve(tauriRoot, distRel);
const indexPath = path.join(distAbs, 'index.html');

if (fs.existsSync(indexPath)) {
  process.exit(0);
}

fs.mkdirSync(distAbs, { recursive: true });
fs.writeFileSync(
  indexPath,
  `<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <title>OCLive (dist stub)</title>
  </head>
  <body>
  <p>Stub dist for <code>cargo check</code>. Run <code>npm run build</code> for real assets.</p>
  </body>
</html>
`,
  'utf8',
);

console.log(
  `[ensure-tauri-dist] created stub at distros/desktop-tauri/${distRel.replace(/\\/g, '/')} (tauri macro gate)`,
);
