import assert from 'node:assert/strict';
import { EventEmitter } from 'node:events';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath, pathToFileURL } from 'node:url';
import vm from 'node:vm';

// Evaluate the unmodified production module and dispatch requests through its HTTP
// callback. Only IO boundaries and the OS value are substituted; no copied handlers,
// string-rewritten production code, listener, child engine or real audio is involved.
const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const pluginRoot = path.join(repoRoot, 'distros/chat-pro/plugins/com.oclive.voice.asr');
const gatewayPath = path.join(pluginRoot, 'rpc_server.mjs');
const source = fs.readFileSync(gatewayPath, 'utf8');
const profilesText = fs.readFileSync(path.join(pluginRoot, 'asr_profiles.json'), 'utf8');
const bundled = 'bundled-cosyvoice2-zh';
const config = { tts_expansion_enabled: true, tts_profile: bundled, synth_provider: 'bundled' };

async function gateway(platform, pluginConfig = config) {
  const effects = { engine_fs: 0, network: 0, spawn: 0 };
  let listener;
  const fakeFs = {
    readFileSync(file) {
      if (path.resolve(file) === path.join(pluginRoot, 'asr_profiles.json')) return profilesText;
      effects.engine_fs++;
      throw new Error('fixture forbids non-profile file reads');
    },
    existsSync() { effects.engine_fs++; return false; },
  };
  const fakeHttp = {
    createServer(callback) {
      listener = callback;
      return {
        listen(_port, _host, ready) { ready(); },
        address() { return { port: 0 }; },
        close() {},
      };
    },
  };
  const context = vm.createContext({
    Buffer, URL, AbortController, AbortSignal, setTimeout, clearTimeout,
    console,
    process: {
      platform,
      env: { OCLIVE_PLUGIN_CONFIG: JSON.stringify(pluginConfig) },
      stdout: { write() {} },
      stderr: { write() {} },
      cwd: () => repoRoot,
      on() {},
    },
    fetch: async () => { effects.network++; throw new Error('fixture forbids network'); },
  });
  const builtins = new Map([
    ['node:child_process', { spawn: () => { effects.spawn++; throw new Error('fixture forbids engine spawn'); } }],
    ['node:fs', { default: fakeFs }],
    ['node:http', { default: fakeHttp }],
    ['node:os', { default: { homedir: () => '/fixture-no-user-home' } }],
    ['node:path', { default: path }],
    ['node:url', { fileURLToPath }],
  ]);
  const module = new vm.SourceTextModule(source, {
    context,
    identifier: pathToFileURL(gatewayPath).href,
    initializeImportMeta(meta) { meta.url = pathToFileURL(gatewayPath).href; },
  });
  await module.link((specifier) => {
    const values = builtins.get(specifier);
    assert.ok(values, `unapproved production import: ${specifier}`);
    return new vm.SyntheticModule(Object.keys(values), function () {
      for (const [key, value] of Object.entries(values)) this.setExport(key, value);
    }, { context });
  });
  await module.evaluate();
  assert.equal(typeof listener, 'function');
  return {
    effects,
    async rpc(method, params = {}) {
      const request = new EventEmitter();
      request.method = 'POST';
      request.url = '/rpc';
      const body = await new Promise((resolve) => {
        listener(request, { setHeader() {}, writeHead() {}, end: resolve });
        request.emit('data', Buffer.from(JSON.stringify({ jsonrpc: '2.0', id: 1, method, params })));
        request.emit('end');
      });
      const envelope = JSON.parse(body);
      assert.equal(envelope.id, 1);
      assert.equal(envelope.error, undefined, JSON.stringify(envelope));
      assert.ok(envelope.result);
      return envelope.result;
    },
  };
}

function noEngineEffects(instance) {
  assert.deepEqual(instance.effects, { engine_fs: 0, network: 0, spawn: 0 });
}

const speakParams = {
  text: '测试文本',
  directive: { synth_profile: bundled, emo_text: '用自然语气' },
};

for (const platform of ['linux', 'darwin']) {
  for (const [method, params] of [
    ['voice.probe', {}],
    ['voice.transcribe', { audio_base64: 'AAAA' }],
    ['voice.probe_tts', { profile: bundled }],
    ['voice.warm', { profile: bundled }],
    ['voice.speak', speakParams],
    ['voice.speak', { ...speakParams, _oclive_resource_admission: { release_after_call: true } }],
  ]) {
    test(`${platform} ${method}${params._oclive_resource_admission ? ' coordinated' : ''}: explicit refusal before engine IO`, { timeout: 2000 }, async () => {
      const instance = await gateway(platform);
      const result = await instance.rpc(method, params);
      assert.equal(result.ok, false);
      assert.equal(result.reason, 'unsupported_platform');
      if (method === 'voice.speak') assert.equal(result.audio_base64, '');
      if (method.startsWith('voice.') && !['voice.probe', 'voice.transcribe'].includes(method)) {
        assert.equal(result.profile, bundled);
        assert.equal(result.platform, platform);
      }
      noEngineEffects(instance);
    });
  }
}

test('profile listing retains the actual Windows support and non-Windows unsupported declarations', async () => {
  for (const platform of ['win32', 'linux', 'darwin']) {
    const instance = await gateway(platform);
    const result = await instance.rpc('voice.list_profiles');
    assert.equal(result.profiles.find(row => row.id === bundled).platform_ready, platform === 'win32');
    assert.equal(result.profiles.find(row => row.id === 'sherpa-paraformer-zh-small').platform_ready, platform === 'win32');
    assert.equal(result.profiles.find(row => row.id === 'local-cosyvoice-http').platform_ready, true);
    noEngineEffects(instance);
  }
});

test('unknown TTS profile refuses without engine IO, including coordinated speak', async () => {
  for (const method of ['voice.probe_tts', 'voice.warm', 'voice.speak']) {
    const instance = await gateway('win32');
    const result = await instance.rpc(method, {
      text: '测试文本', profile: 'missing-profile',
      directive: { synth_profile: 'missing-profile', emo_text: '用自然语气' },
      _oclive_resource_admission: { release_after_call: true },
    });
    assert.equal(result.ok, false);
    assert.equal(result.reason, 'profile_not_found');
    noEngineEffects(instance);
  }
});

test('valid Windows bundled profile reaches engine discovery rather than platform refusal', async () => {
  const instance = await gateway('win32');
  const result = await instance.rpc('voice.probe_tts', { profile: bundled });
  assert.equal(result.ok, false);
  assert.equal(result.reason, 'engine_root_missing');
  assert.ok(instance.effects.engine_fs > 0);
  assert.equal(instance.effects.spawn, 0);
});

test('local HTTP and cloud profiles retain their no-bundled-warm behavior on all declared OSes', async () => {
  for (const platform of ['win32', 'linux', 'darwin']) {
    for (const profile of ['local-cosyvoice-http', 'cloud-tts-openai']) {
      const instance = await gateway(platform, { ...config, tts_profile: profile });
      const result = await instance.rpc('voice.warm', { profile });
      assert.equal(result.ok, true);
      assert.equal(result.skipped, true);
      assert.equal(result.profile, profile);
      noEngineEffects(instance);
    }
  }
});

test('speak keeps empty-text and expansion-disabled priority', async () => {
  for (const [params, settings, expected] of [
    [{ text: '' }, config, 'empty_text'],
    [speakParams, { ...config, tts_expansion_enabled: false }, 'tts_expansion_disabled'],
  ]) {
    const instance = await gateway('linux', settings);
    const result = await instance.rpc('voice.speak', params);
    assert.equal(result.ok, false);
    assert.equal(result.reason, expected);
    noEngineEffects(instance);
  }
});
