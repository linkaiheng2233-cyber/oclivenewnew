#!/usr/bin/env node
import { readFileSync, readdirSync, statSync, writeFileSync, existsSync } from 'fs';
import { join } from 'path';

const ROOT = join(import.meta.dirname, '..');
const fixes = [
  ['| 前端 | `src/`（Vue） |', '| 前端 | `distros/shared/` + `distros/chat-pro/`（Vue） |'],
  ['| Frontend | `src/` (Vue) |', '| Frontend | `distros/shared/` + `distros/chat-pro/` (Vue workspaces) |'],
  [
    '| **仅前端** (`src/`, `distros/chat-pro/e2e/`',
    '| **仅前端** (`distros/shared/`, `distros/chat-pro/`, `distros/chat-pro/e2e/`',
  ],
  ['| **Frontend only** (`src/`', '| **Frontend only** (`distros/shared/`'],
  [
    '`src/main.js`、`distros/shared/src/utils/directoryShellBootstrap.ts`、`src/DirectoryShellApp.vue`',
    '`distros/shared/src/main.js`、`distros/shared/src/utils/directoryShellBootstrap.ts`、`distros/shared/src/DirectoryShellApp.vue`',
  ],
  [
    '`src/main.js`, `distros/shared/src/utils/directoryShellBootstrap.ts`, `src/DirectoryShellApp.vue`',
    '`distros/shared/src/main.js`, `distros/shared/src/utils/directoryShellBootstrap.ts`, `distros/shared/src/DirectoryShellApp.vue`',
  ],
  [
    '| **D-DOCDRIFT-01** | 重组后 normative 文档仍引用 `kernel/crates/` / `distros/desktop-tauri/` / 根 `src/` |',
    '| **D-DOCDRIFT-01** | 重组后 normative 文档路径漂移（旧布局引用） |',
  ],
  [
    '| **D-SCRIPT-02** | `check-stale-paths.mjs` 误报 `memory_backend` 反例行、漏报行内旧路径 |',
    '| **D-SCRIPT-02** | `check-stale-paths.mjs` 误报/漏报（反例说明与行内路径） |',
  ],
  ['| `src/shells/inner/InnerVisualShell.vue` |', '| `distros/chat-pro/src/shells/inner/InnerVisualShell.vue` |'],
  ['src/shells/inner/', 'distros/chat-pro/src/shells/inner/'],
  ['`src/`（Vue workspaces）', '`distros/`（Vue workspaces）'],
  ['| 整个 `src/`（Vue） |', '| 整个 `distros/` 前端（Vue） |'],
  ['视觉要有 `src/`', '视觉要有 `distros/` 前端'],
  ['不动 `src/`', '不动 `distros/` 前端'],
  ['仅 `src/`', '仅 `distros/` 前端'],
  ['改 `src/`', '改 `distros/` 前端'],
  ['未改 `src/`', '未改 `distros/` 前端'],
  ['整个 `src/`', '整个 `distros/` 前端'],
  ['跳过**：`npm run tauri:dev` 章节（除非联调日）、立绘/Live2D 全部文档、整个 `src/`。', '跳过**：`npm run tauri:dev` 章节（除非联调日）、立绘/Live2D 全部文档、整个 `distros/` 前端。'],
  ['只改 `src/`', '只改 `distros/` 前端'],
  ['| `src/` 行数 |', '| `distros/` 前端行数 |'],
  ['| `src/` | `distros/chat-pro/roles/` |', '| 根 `roles/` | `distros/chat-pro/roles/` |'],
];

function walk(d, out = []) {
  for (const name of readdirSync(d)) {
    const p = join(d, name);
    if (name === 'archive' || name === 'node_modules' || name === 'dist') continue;
    const st = statSync(p);
    if (st.isDirectory()) walk(p, out);
    else if (name.endsWith('.md')) out.push(p);
  }
  return out;
}

const files = [];
for (const r of ['creator-docs', 'creator-docs-en', 'human-docs', 'human-docs-en', 'handoff']) {
  walk(join(ROOT, r), files);
}
for (const f of ['CONTRIBUTING.md', 'CONTRIBUTING.en.md', 'handoff/TECHNICAL_DEBT_INVENTORY.md']) {
  const p = join(ROOT, f);
  if (existsSync(p)) files.push(p);
}

let changed = 0;
for (const fp of [...new Set(files)]) {
  let text = readFileSync(fp, 'utf8');
  let out = text;
  for (const [from, to] of fixes) out = out.split(from).join(to);
  if (out !== text) {
    writeFileSync(fp, out, 'utf8');
    changed++;
  }
}
console.log(`fix-remaining-doc-paths: ${changed} file(s)`);
