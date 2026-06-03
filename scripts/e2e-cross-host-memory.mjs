#!/usr/bin/env node
/**
 * Smoke: spawn headless kernel with canonical app data, POST /chat, assert DB file exists.
 * Usage (from repo root): node scripts/e2e-cross-host-memory.mjs
 */
import { spawn } from 'child_process';
import fs from 'fs';
import os from 'os';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, '..');
const port = 18420;
const appData = path.join(os.tmpdir(), `oclive_e2e_${Date.now()}`);
const rolesDir = path.join(repoRoot, 'roles');

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
  const candidates = [
    path.join(repoRoot, '..', 'oclive-dev-artifacts', 'oclivenewnew-cargo-target', 'debug', 'oclive-kernel-server.exe'),
    path.join(repoRoot, '..', 'oclive-dev-artifacts', 'oclivenewnew-cargo-target', 'debug', 'oclivenewnew-tauri.exe'),
    path.join(repoRoot, 'target', 'debug', 'oclive-kernel-server.exe'),
    path.join(repoRoot, 'target', 'debug', 'oclivenewnew-tauri.exe'),
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
  const args = bin.includes('kernel-server') ? ['--api', '--port', String(port)] : ['--api', '--port', String(port)];
  const child = spawn(bin, args, {
    env: {
      ...process.env,
      OCLIVE_APP_DATA: appData,
      OCLIVE_USE_CANONICAL_APP_DATA: '1',
      OCLIVE_HTTP_API_MOCK_LLM: '1',
      OCLIVE_ROLES_DIR: rolesDir,
    },
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

  child.kill();
  console.log('[e2e-cross-host-memory] ok:', body.reply.slice(0, 40));
  console.log('[e2e-cross-host-memory] app_data:', appData);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
