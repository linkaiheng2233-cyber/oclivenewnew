#!/usr/bin/env node
/**
 * Smoke: spawn headless kernel with canonical OCLIVE_APP_DATA; assert app.db on disk.
 * Optional chat when OCLIVE_E2E_CHAT=1 (requires LLM or mock — mock uses in-memory DB).
 * Usage (from repo root): node scripts/e2e-cross-host-memory.mjs
 */
import { spawn } from 'child_process';
import fs from 'fs';
import os from 'os';
import path from 'path';
import { fileURLToPath } from 'url';
import { chatProRolesDir, resolveRepoRoot } from './lib/chat-pro-roles-dir.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = resolveRepoRoot();
const port = Number(process.env.OCLIVE_E2E_PORT || 18420);
const appData = path.join(os.tmpdir(), `oclive_e2e_${Date.now()}`);
const rolesDir = chatProRolesDir(repoRoot);
const wantChat = process.env.OCLIVE_E2E_CHAT === '1';

function sleep(ms) {
  return new Promise((r) => setTimeout(r, ms));
}

async function healthOk() {
  try {
    const res = await fetch(`http://127.0.0.1:${port}/health`, { signal: AbortSignal.timeout(2000) });
    return res.ok;
  } catch {
    return false;
  }
}

function findKernelBinary() {
  if (process.env.OCLIVE_E2E_KERNEL && fs.existsSync(process.env.OCLIVE_E2E_KERNEL)) {
    return process.env.OCLIVE_E2E_KERNEL;
  }
  const candidates = [
    path.join(repoRoot, '..', 'oclive-dev-artifacts', 'oclivenewnew-cargo-target', 'debug', 'oclive-kernel-server.exe'),
    path.join(repoRoot, '..', 'oclive-dev-artifacts', 'oclivenewnew-cargo-target', 'debug', 'oclive-kernel-server'),
    path.join(repoRoot, '..', 'oclive-dev-artifacts', 'oclivenewnew-cargo-target', 'debug', 'oclivenewnew-tauri.exe'),
    path.join(repoRoot, 'target', 'debug', 'oclive-kernel-server.exe'),
    path.join(repoRoot, 'target', 'debug', 'oclive-kernel-server'),
    path.join(repoRoot, 'target', 'debug', 'oclivenewnew-tauri.exe'),
    path.join(repoRoot, 'distros/desktop-tauri', 'target', 'debug', 'oclive-kernel-server'),
    path.join(repoRoot, 'distros/desktop-tauri', 'target', 'debug', 'oclive-kernel-server.exe'),
  ];
  return candidates.find((p) => fs.existsSync(p));
}

async function main() {
  const bin = findKernelBinary();
  if (!bin) {
    console.warn('[e2e-cross-host-memory] skip: no kernel binary (run cargo build -p oclive_kernel_server)');
    process.exit(0);
  }
  fs.mkdirSync(appData, { recursive: true });
  const spawnEnv = {
    ...process.env,
    OCLIVE_APP_DATA: appData,
    OCLIVE_USE_CANONICAL_APP_DATA: '1',
    OCLIVE_ROLES_DIR: rolesDir,
  };
  if (wantChat && process.env.OCLIVE_HTTP_API_MOCK_LLM) {
    spawnEnv.OCLIVE_HTTP_API_MOCK_LLM = process.env.OCLIVE_HTTP_API_MOCK_LLM;
  }

  const child = spawn(bin, ['--api', '--port', String(port)], {
    env: spawnEnv,
    stdio: 'ignore',
    windowsHide: true,
  });

  let ready = false;
  for (let i = 0; i < 40; i++) {
    if (await healthOk()) {
      ready = true;
      break;
    }
    await sleep(500);
  }
  if (!ready) {
    child.kill();
    throw new Error('kernel /health timeout');
  }

  const dbPath = path.join(appData, 'app.db');
  if (!fs.existsSync(dbPath)) {
    child.kill();
    throw new Error(`app.db missing under ${appData}`);
  }
  console.log('[e2e-cross-host-memory] app.db ok:', fs.statSync(dbPath).size, 'bytes');

  if (wantChat) {
    const rolePath = path.join(rolesDir, 'mumu');
    const res = await fetch(`http://127.0.0.1:${port}/chat`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({
        role_path: rolePath,
        message: 'e2e cross-host memory',
        session_id: 'e2e',
        scene_id: 'vscode',
      }),
    });
    const body = await res.json();
    if (!res.ok || !body.reply) {
      child.kill();
      throw new Error(`chat failed: ${JSON.stringify(body)}`);
    }
    console.log('[e2e-cross-host-memory] chat ok:', body.reply.slice(0, 40));
  }

  child.kill();
  console.log('[e2e-cross-host-memory] app_data:', appData);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
