#!/usr/bin/env node
/**
 * Lightweight voice/TTS ratchet (no GPU, no sidecar spawn).
 * - rpc_server.mjs syntax
 * - Python TtsEngine registry >= 9 engines
 */
import { spawnSync } from 'child_process'
import fs from 'fs'
import path from 'path'
import { fileURLToPath } from 'url'

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const rpcServer = path.join(
  repoRoot,
  'distros/chat-pro/plugins/com.oclive.voice.asr/rpc_server.mjs',
)
const voiceLoop = path.join(repoRoot, 'examples/voice-loop-minimal')
const directoryPluginApi = path.join(
  repoRoot,
  'distros/desktop-tauri/src/api/directory_plugin.rs',
)
const pluginConfigApi = path.join(
  repoRoot,
  'distros/desktop-tauri/src/api/plugin_config.rs',
)
const kernelHttpClient = path.join(
  repoRoot,
  'distros/desktop-tauri/src/kernel_attach.rs',
)
const MIN_ENGINES = 9

function fail(msg) {
  console.error(`[voice-tts-ratchet] ${msg}`)
  process.exit(1)
}

if (!fs.existsSync(rpcServer)) {
  fail(`missing ${rpcServer}`)
}

const syntax = spawnSync(process.execPath, ['--check', rpcServer], {
  cwd: repoRoot,
  encoding: 'utf8',
})
if (syntax.status !== 0) {
  fail(syntax.stderr?.trim() || 'rpc_server.mjs syntax check failed')
}
const rpcServerSource = fs.readFileSync(rpcServer, 'utf8')
if (
  !rpcServerSource.includes(
    'else if (method === "config_updated") result = await handleConfigUpdated(params);',
  )
) {
  fail(
    'Voice RPC must await the asynchronous config transition before serializing its confirmation',
  )
}
for (const required of [
  'params?._oclive_resource_admission?.release_after_call === true',
  'resource_transition: {',
  '...(await releaseCosyvoiceSidecar(profileId))',
]) {
  if (!rpcServerSource.includes(required)) {
    fail(
      `coordinated voice.speak must confirm post-synthesis resource release: missing ${required}`,
    )
  }
}

const directoryPluginApiSource = fs.readFileSync(directoryPluginApi, 'utf8')
for (const required of [
  'invoke_directory_plugin_rpc_with_resources(',
  'prepare_directory_plugin_resource_rpc(',
  'finalize_directory_plugin_resource_rpc(',
  'transition_resource_adapter_via_http(',
  'mark_external_performance_preemption(',
  'request_kernel_performance_resume(',
]) {
  if (!directoryPluginApiSource.includes(required)) {
    fail(
      `native directory_plugin_invoke must share bundled voice resource coordination: missing ${required}`,
    )
  }
}
const coordinatedNativeInvokeCount = (
  directoryPluginApiSource.match(
    /invoke_directory_plugin_rpc_with_resources\(/g,
  ) || []
).length
if (coordinatedNativeInvokeCount < 2) {
  fail(
    'native directory_plugin_invoke must call the shared resource-coordinated RPC helper',
  )
}
if (
  !fs
    .readFileSync(kernelHttpClient, 'utf8')
    .includes('/resources/adapter/transition')
) {
  fail(
    'desktop resource coordination must transition the authoritative kernel adapter over HTTP',
  )
}
const pluginConfigApiSource = fs.readFileSync(pluginConfigApi, 'utf8')
for (const required of [
  'DirectoryPluginResourceConfigFinalization::Released',
  'external_performance_preempted: true',
  'request_kernel_performance_resume(kernel).await',
]) {
  if (!pluginConfigApiSource.includes(required)) {
    fail(
      `disabling bundled voice must recover externally preempted Performance LLM: missing ${required}`,
    )
  }
}

const manifestPath = path.join(
  repoRoot,
  'distros/chat-pro/plugins/com.oclive.voice.asr/manifest.json',
)
const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'))
for (const method of ['voice.import_tts_adapter', 'voice.list_tts_adapters']) {
  if (!manifest.rpcMethods?.includes(method)) {
    fail(`manifest.json missing rpcMethods entry: ${method}`)
  }
}

const pySnippet = `
import sys
import os
import importlib.util
import json
import tempfile
import threading
import time
from pathlib import Path
from types import SimpleNamespace
sys.path.insert(0, r'${voiceLoop.replace(/\\/g, '/')}')
from tts.engines.registry import get_registry
from tts.engines import _http
from tts.engines import cosyvoice2 as cosyvoice_engine
import tts.cosyvoice_sidecar as sidecar

n = len(get_registry().list_engine_ids())
assert n >= ${MIN_ENGINES}, f'expected>=${MIN_ENGINES}, got {n}'

class FakeOpener:
    def __init__(self):
        self.called = False
    def open(self, _req, timeout):
        self.called = timeout == 1.0
        return object()

fake_opener = FakeOpener()
original_opener = _http._DIRECT_OPENER
_http._DIRECT_OPENER = fake_opener
_http._open_request(_http.urlrequest.Request('http://127.0.0.1:50000/health'), 1.0)
_http._DIRECT_OPENER = original_opener
assert fake_opener.called, 'loopback TTS requests must bypass system proxies'

class FakeModel:
    def __init__(self):
        self.stream = None
        self.prompt = None
    def inference_instruct2(
        self, _text, _instruct, prompt, zero_shot_spk_id='', *, stream, speed
    ):
        self.stream = stream
        self.prompt = prompt
        return iter(({'tts_speech': object()},))

sidecar._resolve_prompt_wav = lambda _ref: Path('prompt.wav')
sidecar._primed = False
fake = FakeModel()
ok, _ms, error = sidecar._prime_cosyvoice_model(fake)
assert ok and not error
assert fake.stream is False, 'Windows-safe prime must be non-streaming'
assert fake.prompt == 'prompt.wav', 'prime must pass the compatible prompt path'

stream_steps = []
class StreamingFake(FakeModel):
    def inference_instruct2(
        self, _text, _instruct, prompt, zero_shot_spk_id='', *, stream, speed
    ):
        self.stream = stream
        def chunks():
            stream_steps.append('first')
            yield {'tts_speech': object()}
            stream_steps.append('second')
            yield {'tts_speech': object()}
        return chunks()

sidecar._tensor_to_pcm_base64 = lambda _tensor, sample_rate: ('AA==', sample_rate)
stream_fake = StreamingFake()
lines = sidecar._stream_synthesis_lines(
    stream_fake,
    text='你好呀，',
    emo_text='用自然平静的语气',
    ref_audio='',
    ref_text='',
    speed=1.0,
)
first = next(lines)
assert '"event": "chunk"' in first
first_event = json.loads(first)
assert first_event['timings_schema_version'] == 1
first_timings = first_event['timings_ms']
for timing_key in (
    'request_preprocess',
    'response_setup',
    'synth_lock_wait',
    'prompt_prepare',
    'inference_open',
    'first_tensor_wait',
    'pre_token2wav_wait',
    'first_token2wav',
    'first_pcm_encode',
    'server_first_tensor',
    'server_payload_ready',
):
    assert isinstance(first_timings[timing_key], int) and first_timings[timing_key] >= 0
assert first_timings['server_payload_ready'] >= first_timings['server_first_tensor']
assert stream_fake.stream is True, 'default sidecar path must request real streaming'
assert stream_steps == ['first'], 'first PCM must be yielded before full inference completes'
remaining = list(lines)
assert any('"stream_mode": "streaming"' in line for line in remaining)
done_event = next(json.loads(line) for line in remaining if '"event": "done"' in line)
assert done_event['timings_schema_version'] == 1
assert done_event['timings_ms']['total'] >= done_event['timings_ms']['server_payload_ready']
assert done_event['timings_ms']['speech_token_job_total'] >= 0

class FakeRuntime:
    token_hop_len = 100
    _oclive_initial_stream_hop_len = 25

class FakeModel:
    model = FakeRuntime()

sidecar._reset_stream_hop_len(FakeModel())
assert FakeModel.model.token_hop_len == 25

buffered_fake = StreamingFake()
buffered_lines = list(sidecar._stream_synthesis_lines(
    buffered_fake,
    text='太好啦。',
    emo_text='用开心明亮的语气',
    ref_audio='',
    ref_text='',
    speed=1.1,
))
assert buffered_fake.stream is False, 'non-default speed must preserve director semantics'
assert any('"stream_mode": "buffered"' in line for line in buffered_lines)

sidecar._model = object()
sidecar._model_dir = Path('.')
sidecar._prime_failed_reason = 'prime_failed:test'
sidecar._model_ready = lambda _path: (True, '')
health = sidecar.health_payload()
assert not health['ok'] and health['reason'] == 'prime_failed:test'

with tempfile.TemporaryDirectory() as unload_dir, tempfile.TemporaryDirectory() as other_dir:
    unload_cache_calls = []
    old_empty_cuda_cache = sidecar._empty_cuda_cache
    try:
        resident = object()
        sidecar._model = resident
        sidecar._model_dir = Path(unload_dir)
        sidecar._warmed = True
        sidecar._primed = True
        sidecar._prepared_speakers[('prompt', 1, 1, 'text')] = 'speaker'
        sidecar._empty_cuda_cache = lambda: unload_cache_calls.append(True)
        mismatch = sidecar.unload_cosyvoice_model(other_dir)
        assert not mismatch['ok'] and not mismatch['released']
        assert sidecar._model is resident, 'model mismatch must not unload another runtime'
        released = sidecar.unload_cosyvoice_model(unload_dir)
        assert released['ok'] and released['released']
        assert not released['already_unloaded']
        assert sidecar._model is None
        assert not sidecar._warmed and not sidecar._primed
        assert not sidecar._prepared_speakers
        assert unload_cache_calls == [True]
        repeated = sidecar.unload_cosyvoice_model(unload_dir)
        assert repeated['ok'] and repeated['released'] and repeated['already_unloaded']

        sidecar._model = object()
        sidecar._warmed = True
        transition_result = []
        with sidecar._synth_lock:
            worker = threading.Thread(
                target=lambda: transition_result.append(
                    sidecar.unload_cosyvoice_model(unload_dir)
                )
            )
            worker.start()
            time.sleep(0.05)
            assert not transition_result, 'unload must wait for active synthesis'
        worker.join(timeout=1.0)
        assert not worker.is_alive()
        assert transition_result[0]['released']
    finally:
        sidecar._empty_cuda_cache = old_empty_cuda_cache

class PrecisionComponent:
    def __init__(self):
        self.half_calls = 0
        self.float_calls = 0
        self.to_calls = []
        self.loaded_state = {}
    def half(self):
        self.half_calls += 1
        return self
    def float(self):
        self.float_calls += 1
        return self
    def to(self, device):
        self.to_calls.append(device)
        return self
    def eval(self):
        return self
    def load_state_dict(self, state, strict):
        assert strict is True
        self.loaded_state = dict(state)

class PrecisionRuntime:
    def __init__(self):
        self.llm = PrecisionComponent()
        self.flow = PrecisionComponent()
        self.hift = PrecisionComponent()
        self.fp16 = False
        self.device = 'cuda'

class PrecisionModel:
    def __init__(self):
        self.model = PrecisionRuntime()
        self.fp16 = False

old_precision = os.environ.get('OCLIVE_COSYVOICE_PRECISION')
old_min_free = os.environ.get('OCLIVE_COSYVOICE_MIN_FREE_VRAM_MIB')
old_cuda_available = sidecar._cuda_available
old_gpu_admission = sidecar._gpu_load_admission
try:
    os.environ['OCLIVE_COSYVOICE_PRECISION'] = 'auto'
    sidecar._cuda_available = lambda: True
    sidecar._gpu_load_admission = lambda *_args, **_kwargs: (
        True, 4096, 8192, 2048, '', 'test'
    )
    precision_model = PrecisionModel()
    sidecar._configure_model_precision(precision_model)
    assert sidecar._precision_active == 'mixed_fp16'
    assert precision_model.model.llm.half_calls == 1
    assert precision_model.model.flow.half_calls == 1
    assert precision_model.model.hift.half_calls == 0, 'HiFT must stay fp32'
    assert precision_model.fp16 and precision_model.model.fp16
    assert sidecar._fallback_model_to_fp32(precision_model, 'prime_failed:test')
    assert sidecar._precision_active == 'fp32'
    assert precision_model.model.llm.float_calls == 1
    assert precision_model.model.flow.float_calls == 1
    assert not precision_model.fp16 and not precision_model.model.fp16

    class FakeCuda:
        empty_cache_calls = 0
        @staticmethod
        def is_available():
            return True
        @staticmethod
        def empty_cache():
            FakeCuda.empty_cache_calls += 1
        @staticmethod
        def mem_get_info():
            mib = 1024 * 1024
            return 2500 * mib, 8192 * mib

    class FakeTorch:
        cuda = FakeCuda
        @staticmethod
        def load(path, map_location, weights_only):
            assert map_location == 'cpu' and weights_only is True
            return {'generator.weight': path} if path == 'hift.pt' else {'weight': path}

    staged_runtime = PrecisionRuntime()
    sidecar._staged_mixed_runtime_load(
        staged_runtime,
        'llm.pt',
        'flow.pt',
        'hift.pt',
        torch_module=FakeTorch,
    )
    assert staged_runtime.llm.half_calls == 1
    assert staged_runtime.flow.half_calls == 1
    assert staged_runtime.hift.half_calls == 0, 'staged loader must leave HiFT fp32'
    assert staged_runtime.llm.to_calls == ['cuda']
    assert staged_runtime.flow.to_calls == ['cuda']
    assert staged_runtime.hift.to_calls == ['cuda']
    assert staged_runtime.hift.loaded_state == {'weight': 'hift.pt'}
    denied_status = cosyvoice_engine._runtime_status({
        'retryable': True,
        'load_vram_probe': 'nvidia_smi',
        'load_free_vram_before_mib': 2500,
    })
    assert denied_status['retryable'] is True
    assert denied_status['load_free_vram_before_mib'] == 2500

    os.environ['OCLIVE_COSYVOICE_MIN_FREE_VRAM_MIB'] = '2560'
    admitted = old_gpu_admission(
        'mixed_fp16',
        torch_module=FakeTorch,
        memory_probe=lambda _torch: (2500, 8192, 'test'),
    )
    assert admitted[0] is False
    assert admitted[1:4] == (2500, 8192, 2560)
    assert 'required_mib=2560' in admitted[4]
    assert admitted[5] == 'test'
    valid_host, host_detail = sidecar._valid_host_resource_admission({
        'schema_version': 1,
        'granted': True,
        'lease_id': 'resource-lease-42',
        'reservation_mib': 768,
    })
    assert valid_host and 'reservation_mib=768' in host_detail
    invalid_host, _ = sidecar._valid_host_resource_admission({
        'schema_version': 1,
        'granted': True,
        'lease_id': 'forged-shape',
        'reservation_mib': 768,
    })
    assert not invalid_host
finally:
    sidecar._cuda_available = old_cuda_available
    sidecar._gpu_load_admission = old_gpu_admission
    if old_precision is None:
        os.environ.pop('OCLIVE_COSYVOICE_PRECISION', None)
    else:
        os.environ['OCLIVE_COSYVOICE_PRECISION'] = old_precision
    if old_min_free is None:
        os.environ.pop('OCLIVE_COSYVOICE_MIN_FREE_VRAM_MIB', None)
    else:
        os.environ['OCLIVE_COSYVOICE_MIN_FREE_VRAM_MIB'] = old_min_free

old_http_json = cosyvoice_engine.http_json
try:
    captured_payloads = []
    def fake_http_json(_url, payload=None, **_kwargs):
        captured_payloads.append(payload)
        return {
            'ok': True,
            'warmed': True,
            'primed': True,
            'precision_requested': 'auto',
            'precision_active': 'mixed_fp16',
            'precision_fallback_reason': '',
            'load_strategy': 'staged_cpu_mixed_fp16',
            'load_vram_probe': 'nvidia_smi',
            'load_peak_reserved_mib': 1500,
        }
    cosyvoice_engine.http_json = fake_http_json
    host_admission = {
        'schema_version': 1,
        'granted': True,
        'lease_id': 'resource-lease-42',
        'reservation_mib': 768,
    }
    warm = cosyvoice_engine.Cosyvoice2Engine().warm(
        model_dir='.',
        manifest={'id': 'test'},
        host_resource_admission=host_admission,
    )
    assert warm['ok'] is True
    assert captured_payloads[-1]['host_resource_admission'] == host_admission
    with tempfile.TemporaryDirectory() as temp_dir:
        probe_dir = Path(temp_dir)
        (probe_dir / 'MANIFEST.json').write_text('{}', encoding='utf-8')
        probe = cosyvoice_engine.Cosyvoice2Engine().probe(
            str(probe_dir), manifest={'id': 'test'}
        )
        assert probe['precision_active'] == 'mixed_fp16'
        assert probe['primed'] is True
        assert probe['load_strategy'] == 'staged_cpu_mixed_fp16'
        assert probe['load_vram_probe'] == 'nvidia_smi'
        assert probe['load_peak_reserved_mib'] == 1500
finally:
    cosyvoice_engine.http_json = old_http_json

stress_path = Path(r'${path.join(repoRoot, 'scripts/stress-voice-gpu-runtime.py').replace(/\\/g, '/')}')
stress_spec = importlib.util.spec_from_file_location('oclive_voice_gpu_stress', stress_path)
stress = importlib.util.module_from_spec(stress_spec)
stress_spec.loader.exec_module(stress)
with tempfile.TemporaryDirectory() as temp_dir:
    output = Path(temp_dir) / 'soak.json'
    args = SimpleNamespace(output=output, duration_minutes=72.0, gpu_layers=22)
    sampler = SimpleNamespace(sample_count=3, sample_failures=0, peak_used_mib=6400)
    stress.emit_progress_checkpoint(
        output,
        status='running',
        started_at='2026-08-01T00:00:00+00:00',
        elapsed_seconds=60.0,
        args=args,
        llm_samples=[{'ttft_ms': 250}],
        voice_samples=[{'ttfc_ms': 1800, 'timings_ms': {'first_token2wav': 900}, 'chunks': 1}],
        sampler=sampler,
        llm_process=None,
        voice_process=None,
        failures=[],
    )
    checkpoint = json.loads((Path(temp_dir) / 'soak.checkpoint.json').read_text(encoding='utf-8'))
    assert checkpoint['status'] == 'running'
    assert checkpoint['pairs_completed'] == 1
    assert checkpoint['latest']['voice_timings_ms']['first_token2wav'] == 900
print('engines', n, 'prime-safe', ok)
`
const customPy = process.env.OCLIVE_VOICE_PYTHON?.trim()
const pyCmd = customPy || (process.platform === 'win32' ? 'py' : 'python3')
const pyArgs = customPy
  ? ['-c', pySnippet]
  : process.platform === 'win32'
    ? ['-3', '-c', pySnippet]
    : ['-c', pySnippet]

const registry = spawnSync(pyCmd, pyArgs, {
  cwd: voiceLoop,
  encoding: 'utf8',
  env: { ...process.env, PYTHONPATH: voiceLoop },
})
if (registry.status !== 0) {
  fail(
    [registry.stderr, registry.stdout].filter(Boolean).join('\n').trim()
      || 'Python registry check failed (is Python 3.10+ available?)',
  )
}

console.log('[voice-tts-ratchet] PASS')
