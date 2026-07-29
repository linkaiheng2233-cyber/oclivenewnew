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
import tempfile
from pathlib import Path
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
assert stream_fake.stream is True, 'default sidecar path must request real streaming'
assert stream_steps == ['first'], 'first PCM must be yielded before full inference completes'
remaining = list(lines)
assert any('"stream_mode": "streaming"' in line for line in remaining)

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
