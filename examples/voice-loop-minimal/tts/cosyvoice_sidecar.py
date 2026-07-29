"""CosyVoice2 HTTP sidecar — persistent TTS process for voice expansion.

Endpoints:
  GET  /health   — { ok, engine, model_dir, warmed, message }
  POST /warm     — preload model (optional body: { model_dir })
  POST /synthesize — { text, emo_text?, ref_audio?, ref_text?, speed? }
  POST /synthesize/stream — NDJSON stream of PCM chunks (lower time-to-first-sound)

Set env:
  OCLIVE_COSYVOICE_MODEL_DIR — path to imported CosyVoice2-0.5B weights
  OCLIVE_COSYVOICE_PORT      — listen port (default 50000)
  OCLIVE_COSYVOICE_PRECISION — auto (default), mixed_fp16, or fp32
  OCLIVE_COSYVOICE_MIN_FREE_VRAM_MIB — optional cold-load admission override

Run: python -m tts.cosyvoice_sidecar
Stdout: OCLIVE_SIDECAR_READY http://127.0.0.1:<port>
"""

from __future__ import annotations

import hashlib
import gc
import json
import os
import subprocess
import sys
import threading
import time
import wave
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any

_ENGINE = "cosyvoice2"
_model_dir: Path | None = None
_model = None
_warmed = False
_primed = False
_prime_failed_reason = ""
_precision_requested = "auto"
_precision_active = "fp32"
_precision_fallback_reason = ""
_last_load_error = ""
_load_strategy = "not_loaded"
_load_admission_detail = ""
_load_vram_probe = "unavailable"
_load_free_vram_before_mib = 0
_load_total_vram_mib = 0
_load_min_free_vram_mib = 0
_load_peak_allocated_mib = 0
_load_peak_reserved_mib = 0
_lock = threading.Lock()
_synth_lock = threading.Lock()
_prepared_speakers: dict[tuple[str, int, int, str], str] = {}
_PREPARED_SPEAKER_CACHE_MAX = 8
_PRECISION_AUTO = "auto"
_PRECISION_FP32 = "fp32"
_PRECISION_MIXED_FP16 = "mixed_fp16"
_PRECISION_VALUES = {
    _PRECISION_AUTO,
    _PRECISION_FP32,
    _PRECISION_MIXED_FP16,
}
_DEFAULT_MIXED_MIN_FREE_VRAM_MIB = 2560
_DEFAULT_FP32_MIN_FREE_VRAM_MIB = 4096
_DEFAULT_FP32_EXPANSION_MIN_FREE_VRAM_MIB = 2048


def _resolve_model_dir() -> Path:
    env = os.environ.get("OCLIVE_COSYVOICE_MODEL_DIR", "").strip()
    if env:
        return Path(env)
    return Path("models/tts/cosyvoice2-0.5b")


def _default_prompt_wav_path() -> Path:
    env = os.environ.get("OCLIVE_COSYVOICE_DEFAULT_PROMPT_WAV", "").strip()
    if env:
        return Path(env)
    try:
        import cosyvoice  # type: ignore[import-untyped]

        repo_root = Path(cosyvoice.__file__).resolve().parent.parent
        return repo_root / "asset" / "zero_shot_prompt.wav"
    except ImportError:
        return Path()


def _resolve_prompt_wav(ref_audio: str) -> Path | None:
    if ref_audio.strip() and Path(ref_audio).is_file():
        return Path(ref_audio)
    fallback = _default_prompt_wav_path()
    return fallback if fallback.is_file() else None


def _prepare_speaker(
    model: Any,
    prompt_path: Path,
    prompt_text: str,
) -> tuple[str, bool, str]:
    """Cache prompt features for one voice + prompt-text combination.

    CosyVoice2 stores prompt text together with the speaker features. Instruct
    mode therefore keys by the formatted emotion instruction as well as the
    audio revision; reusing a speaker id across different instructions would
    reintroduce the old prompt-text leakage.
    """
    add_speaker = getattr(model, "add_zero_shot_spk", None)
    frontend = getattr(model, "frontend", None)
    spk2info = getattr(frontend, "spk2info", None)
    if not callable(add_speaker) or not isinstance(spk2info, dict):
        return "", False, ""
    try:
        stat = prompt_path.stat()
        resolved = str(prompt_path.resolve())
        key = (resolved, stat.st_mtime_ns, stat.st_size, prompt_text)
        cached_id = _prepared_speakers.get(key)
        if cached_id and cached_id in spk2info:
            return cached_id, True, ""
        digest = hashlib.sha256(
            f"{resolved}\0{stat.st_mtime_ns}\0{stat.st_size}\0{prompt_text}".encode("utf-8")
        ).hexdigest()[:24]
        speaker_id = f"oclive-{digest}"
        add_speaker(prompt_text, str(prompt_path), speaker_id)
        if len(_prepared_speakers) >= _PREPARED_SPEAKER_CACHE_MAX:
            oldest_key = next(iter(_prepared_speakers))
            oldest_id = _prepared_speakers.pop(oldest_key)
            spk2info.pop(oldest_id, None)
        _prepared_speakers[key] = speaker_id
        return speaker_id, False, ""
    except Exception as exc:  # noqa: BLE001
        return "", False, f"prompt_prepare_failed:{exc}"


def _format_instruct_text(emo_text: str) -> str:
    text = emo_text.strip()
    if not text:
        return ""
    if "<|endofprompt|>" in text:
        return text
    return f"{text}<|endofprompt|>"


def _model_ready(path: Path) -> tuple[bool, str]:
    if not path.is_dir():
        return False, "model_dir_missing"
    manifest = path / "MANIFEST.json"
    if not manifest.is_file():
        return False, "manifest_missing"
    return True, ""


def _resolve_precision_request() -> tuple[str, str]:
    raw = os.environ.get("OCLIVE_COSYVOICE_PRECISION", _PRECISION_AUTO)
    normalized = raw.strip().lower().replace("-", "_") or _PRECISION_AUTO
    if normalized == "fp16":
        normalized = _PRECISION_MIXED_FP16
    if normalized in _PRECISION_VALUES:
        return normalized, ""
    return _PRECISION_FP32, f"unsupported_precision:{normalized}"


def _cuda_available() -> bool:
    try:
        import torch

        return bool(torch.cuda.is_available())
    except ImportError:
        return False


def _empty_cuda_cache() -> None:
    try:
        import torch

        if torch.cuda.is_available():
            torch.cuda.empty_cache()
    except ImportError:
        return


def _required_free_vram_mib(precision: str, *, expansion: bool = False) -> tuple[int, str]:
    default = (
        _DEFAULT_FP32_EXPANSION_MIN_FREE_VRAM_MIB
        if expansion
        else (
            _DEFAULT_MIXED_MIN_FREE_VRAM_MIB
            if precision == _PRECISION_MIXED_FP16
            else _DEFAULT_FP32_MIN_FREE_VRAM_MIB
        )
    )
    raw = os.environ.get("OCLIVE_COSYVOICE_MIN_FREE_VRAM_MIB", "").strip()
    if not raw:
        return default, ""
    try:
        return max(0, int(raw)), ""
    except ValueError:
        return default, f"invalid_min_free_vram_mib:{raw}"


def _gpu_load_admission(
    precision: str,
    *,
    expansion: bool = False,
    torch_module: Any | None = None,
    memory_probe: Any | None = None,
) -> tuple[bool, int, int, int, str, str]:
    """Check global free VRAM before a cold load or an in-place FP32 expansion."""
    torch_api = torch_module
    if torch_api is None:
        try:
            import torch as torch_api
        except ImportError:
            return True, 0, 0, 0, "", "unavailable"
    if not torch_api.cuda.is_available():
        return True, 0, 0, 0, "", "cpu"
    required, config_detail = _required_free_vram_mib(precision, expansion=expansion)
    probe = memory_probe or _global_gpu_memory_info_mib
    try:
        free_mib, total_mib, probe_source = probe(torch_api)
    except Exception as exc:  # noqa: BLE001
        # An unavailable metric must not brick CPU-compatible or future CUDA
        # runtimes. The load remains observable and CUDA OOM is still caught.
        detail = f"vram_probe_failed:{exc}"
        if config_detail:
            detail = f"{config_detail};{detail}"
        return True, 0, 0, required, detail, "unavailable"
    admitted = required == 0 or free_mib >= required
    detail = config_detail
    if not admitted:
        denial = f"free_vram_mib={free_mib};required_mib={required}"
        detail = f"{detail};{denial}" if detail else denial
    return admitted, free_mib, total_mib, required, detail, probe_source


def _valid_host_resource_admission(value: Any) -> tuple[bool, str]:
    """Accept the host lease only for the mixed cold-load gate.

    This is a coordination hand-off, not a security boundary: the sidecar is
    loopback-only and the local machine owner already controls both processes.
    FP32 expansion keeps its stricter sidecar-local gate because it can require
    substantially more VRAM than the mixed runtime reservation.
    """
    if not isinstance(value, dict) or value.get("granted") is not True:
        return False, ""
    if int(value.get("schema_version") or 0) != 1:
        return False, ""
    lease_id = str(value.get("lease_id") or "").strip()
    try:
        reservation_mib = max(0, int(value.get("reservation_mib") or 0))
    except (TypeError, ValueError):
        return False, ""
    if not lease_id.startswith("resource-lease-"):
        return False, ""
    return True, f"lease_id={lease_id};reservation_mib={reservation_mib}"


def _global_gpu_memory_info_mib(torch_api: Any) -> tuple[int, int, str]:
    """Prefer the driver-wide view; Windows WDDM CUDA may hide peer processes."""
    logical_index = int(torch_api.cuda.current_device())
    visible = os.environ.get("CUDA_VISIBLE_DEVICES", "").strip()
    device_selector = str(logical_index)
    if visible:
        visible_devices = [item.strip() for item in visible.split(",") if item.strip()]
        if logical_index < len(visible_devices):
            device_selector = visible_devices[logical_index]
    creation_flags = getattr(subprocess, "CREATE_NO_WINDOW", 0)
    try:
        result = subprocess.run(
            [
                "nvidia-smi",
                "-i",
                device_selector,
                "--query-gpu=memory.free,memory.total",
                "--format=csv,noheader,nounits",
            ],
            check=True,
            capture_output=True,
            text=True,
            timeout=5,
            creationflags=creation_flags,
        )
        first_line = result.stdout.splitlines()[0]
        free_raw, total_raw = (part.strip() for part in first_line.split(",", 1))
        return int(free_raw), int(total_raw), "nvidia_smi"
    except (OSError, subprocess.SubprocessError, ValueError, IndexError):
        free_bytes, total_bytes = torch_api.cuda.mem_get_info()
        return (
            int(free_bytes // (1024 * 1024)),
            int(total_bytes // (1024 * 1024)),
            "torch_cuda",
        )


def _staged_mixed_runtime_load(
    runtime_model: Any,
    llm_model: str,
    flow_model: str,
    hift_model: str,
    *,
    torch_module: Any | None = None,
) -> None:
    """Load checkpoints on CPU, then move each finalized component to CUDA.

    Upstream maps each FP32 checkpoint directly to the runtime device. On CUDA
    that creates a large transient copy before OCLive can convert the model.
    Loading sequentially on CPU and converting LLM/flow before `.to(cuda)`
    bounds the device cold peak while keeping HiFT FP32 for compatibility.
    """
    torch_api = torch_module
    if torch_api is None:
        import torch as torch_api

    for component_name, checkpoint in (("llm", llm_model), ("flow", flow_model)):
        component = getattr(runtime_model, component_name)
        state = torch_api.load(checkpoint, map_location="cpu", weights_only=True)
        component.load_state_dict(state, strict=True)
        del state
        gc.collect()
        component.half().to(runtime_model.device).eval()
        gc.collect()
        if torch_api.cuda.is_available():
            torch_api.cuda.empty_cache()

    raw_hift_state = torch_api.load(hift_model, map_location="cpu", weights_only=True)
    hift_state = {
        key.replace("generator.", ""): value
        for key, value in raw_hift_state.items()
    }
    del raw_hift_state
    runtime_model.hift.load_state_dict(hift_state, strict=True)
    del hift_state
    gc.collect()
    runtime_model.hift.to(runtime_model.device).eval()
    gc.collect()
    if torch_api.cuda.is_available():
        torch_api.cuda.empty_cache()


def _reset_cuda_load_peak() -> None:
    try:
        import torch

        if torch.cuda.is_available():
            torch.cuda.reset_peak_memory_stats()
    except (ImportError, RuntimeError):
        return


def _record_cuda_load_peak() -> None:
    global _load_peak_allocated_mib, _load_peak_reserved_mib
    try:
        import torch

        if not torch.cuda.is_available():
            return
        _load_peak_allocated_mib = int(torch.cuda.max_memory_allocated() // (1024 * 1024))
        _load_peak_reserved_mib = int(torch.cuda.max_memory_reserved() // (1024 * 1024))
    except (ImportError, RuntimeError):
        return


def _construct_staged_mixed_model(model_class: Any, path: Path) -> Any:
    from cosyvoice.cli.model import CosyVoice2Model  # type: ignore[import-untyped]

    original_load = CosyVoice2Model.load
    CosyVoice2Model.load = _staged_mixed_runtime_load
    try:
        return model_class(str(path), fp16=True)
    finally:
        CosyVoice2Model.load = original_load


def _convert_runtime_modules(model: Any, method_name: str) -> None:
    runtime_model = getattr(model, "model", None)
    if runtime_model is None:
        raise RuntimeError("runtime_model_missing")
    # HiFT stays fp32: its inference runs outside upstream autocast and is only
    # ~80 MiB, while converting it introduces float/half convolution mismatch.
    for component_name in ("llm", "flow"):
        component = getattr(runtime_model, component_name, None)
        converter = getattr(component, method_name, None)
        if not callable(converter):
            raise RuntimeError(f"{component_name}_{method_name}_unavailable")
        converter()
    mixed = method_name == "half"
    runtime_model.fp16 = mixed
    model.fp16 = mixed
    _empty_cuda_cache()


def _configure_model_precision(model: Any) -> None:
    global _precision_requested, _precision_active, _precision_fallback_reason
    requested, request_reason = _resolve_precision_request()
    _precision_requested = requested
    _precision_active = _PRECISION_FP32
    _precision_fallback_reason = request_reason
    if requested == _PRECISION_FP32:
        return
    if not _cuda_available():
        _precision_fallback_reason = request_reason or "cuda_unavailable"
        return
    try:
        _convert_runtime_modules(model, "half")
        _precision_active = _PRECISION_MIXED_FP16
    except Exception as exc:  # noqa: BLE001
        try:
            _convert_runtime_modules(model, "float")
        except Exception as restore_exc:  # noqa: BLE001
            _precision_fallback_reason = (
                f"mixed_fp16_load_failed:{exc};fp32_restore_failed:{restore_exc}"
            )
            return
        _precision_fallback_reason = f"mixed_fp16_load_failed:{exc}"


def _fallback_model_to_fp32(model: Any, reason: str) -> bool:
    global _precision_active, _precision_fallback_reason
    global _load_admission_detail, _load_vram_probe
    global _primed, _prime_failed_reason
    if _precision_active != _PRECISION_MIXED_FP16:
        return False
    admitted, free_mib, total_mib, required_mib, detail, probe_source = _gpu_load_admission(
        _PRECISION_FP32,
        expansion=True,
    )
    _load_admission_detail = detail
    _load_vram_probe = probe_source
    if not admitted:
        _precision_fallback_reason = f"{reason};fp32_admission_denied:{detail}"
        sys.stderr.write(
            "cosyvoice2 fp32 fallback denied "
            f"free_mib={free_mib} total_mib={total_mib} required_mib={required_mib}\n"
        )
        sys.stderr.flush()
        return False
    try:
        _convert_runtime_modules(model, "float")
    except Exception as exc:  # noqa: BLE001
        _precision_fallback_reason = f"{reason};fp32_restore_failed:{exc}"
        return False
    _precision_active = _PRECISION_FP32
    _precision_fallback_reason = reason
    _primed = False
    _prime_failed_reason = ""
    return True


def _load_cosyvoice_model(
    path: Path,
    host_resource_admission: Any | None = None,
) -> tuple[Any | None, str]:
    global _model, _model_dir, _warmed, _primed, _prime_failed_reason
    global _precision_requested, _precision_active, _precision_fallback_reason
    global _last_load_error, _load_strategy, _load_admission_detail, _load_vram_probe
    global _load_free_vram_before_mib, _load_total_vram_mib
    global _load_min_free_vram_mib, _load_peak_allocated_mib
    global _load_peak_reserved_mib
    ready, reason = _model_ready(path)
    if not ready:
        _last_load_error = reason
        return None, reason
    try:
        # wetext calls ModelScope's snapshot API even when its files are already
        # cached. Voice expansion is local/private by default, so make those
        # dependency lookups cache-only before CosyVoice imports wetext.
        try:
            import modelscope  # type: ignore[import-untyped]

            snapshot_download = modelscope.snapshot_download
            if not getattr(snapshot_download, "_oclive_cache_only", False):
                def cache_only_snapshot(*args: Any, **kwargs: Any) -> Any:
                    kwargs.setdefault("local_files_only", True)
                    return snapshot_download(*args, **kwargs)

                cache_only_snapshot._oclive_cache_only = True  # type: ignore[attr-defined]
                modelscope.snapshot_download = cache_only_snapshot
        except ImportError:
            pass
        from cosyvoice.cli.cosyvoice import CosyVoice2  # type: ignore[import-untyped]
    except ImportError:
        _last_load_error = "cosyvoice_not_installed"
        return None, "cosyvoice_not_installed"
    try:
        with _lock:
            if _model is not None and _model_dir == path:
                _warmed = True
                return _model, ""
            requested, request_reason = _resolve_precision_request()
            use_mixed = requested != _PRECISION_FP32 and _cuda_available()
            target_precision = (
                _PRECISION_MIXED_FP16 if use_mixed else _PRECISION_FP32
            )
            _precision_requested = requested
            _precision_active = _PRECISION_FP32
            _precision_fallback_reason = request_reason
            if requested != _PRECISION_FP32 and not use_mixed:
                _precision_fallback_reason = request_reason or "cuda_unavailable"
            _last_load_error = ""
            _load_strategy = (
                "staged_cpu_mixed_fp16" if use_mixed else "legacy_fp32"
            )
            _load_peak_allocated_mib = 0
            _load_peak_reserved_mib = 0
            (
                admitted,
                _load_free_vram_before_mib,
                _load_total_vram_mib,
                _load_min_free_vram_mib,
                _load_admission_detail,
                _load_vram_probe,
            ) = _gpu_load_admission(target_precision)
            host_admitted, host_admission_detail = _valid_host_resource_admission(
                host_resource_admission
            )
            if host_admitted and target_precision == _PRECISION_MIXED_FP16:
                admitted = True
                coordination_detail = (
                    f"host_resource_coordinator:{host_admission_detail}"
                )
                _load_admission_detail = (
                    f"{coordination_detail};sidecar_gate:{_load_admission_detail}"
                    if _load_admission_detail
                    else coordination_detail
                )
                _load_vram_probe = f"{_load_vram_probe}+host_coordinator"
            if not admitted:
                _last_load_error = "gpu_admission_denied"
                sys.stderr.write(
                    "cosyvoice2 cold load denied "
                    f"precision={target_precision} {_load_admission_detail}\n"
                )
                sys.stderr.flush()
                return None, _last_load_error

            _reset_cuda_load_peak()
            if use_mixed:
                try:
                    model = _construct_staged_mixed_model(CosyVoice2, path)
                    _precision_active = _PRECISION_MIXED_FP16
                except Exception as staged_exc:  # noqa: BLE001
                    gc.collect()
                    _empty_cuda_cache()
                    _precision_fallback_reason = (
                        f"staged_mixed_fp16_load_failed:{staged_exc}"
                    )
                    (
                        fp32_admitted,
                        _load_free_vram_before_mib,
                        _load_total_vram_mib,
                        _load_min_free_vram_mib,
                        _load_admission_detail,
                        _load_vram_probe,
                    ) = _gpu_load_admission(_PRECISION_FP32)
                    if not fp32_admitted:
                        _last_load_error = "gpu_admission_denied"
                        _precision_fallback_reason += (
                            f";fp32_admission_denied:{_load_admission_detail}"
                        )
                        return None, _last_load_error
                    _load_strategy = "legacy_fp32_fallback"
                    _reset_cuda_load_peak()
                    model = CosyVoice2(str(path), fp16=False)
            else:
                model = CosyVoice2(str(path), fp16=False)
            _record_cuda_load_peak()
            runtime_model = getattr(model, "model", None)
            original_llm_job = getattr(runtime_model, "llm_job", None)
            if callable(original_llm_job) and not getattr(
                original_llm_job, "_oclive_guarded", False
            ):
                def guarded_llm_job(*args: Any, **kwargs: Any) -> Any:
                    request_id = kwargs.get("uuid") or (args[-1] if args else "")
                    try:
                        return original_llm_job(*args, **kwargs)
                    finally:
                        # Upstream streaming waits on this flag. If its worker
                        # raises, setting it here turns an infinite wait into a
                        # normal synthesis error that the host can recover from.
                        end_states = getattr(runtime_model, "llm_end_dict", None)
                        if isinstance(end_states, dict) and request_id in end_states:
                            end_states[request_id] = True

                guarded_llm_job._oclive_guarded = True  # type: ignore[attr-defined]
                runtime_model.llm_job = guarded_llm_job
            _prepared_speakers.clear()
            _model = model
            _model_dir = path
            _warmed = True
            _primed = False
            _prime_failed_reason = ""
            _last_load_error = ""
            sys.stderr.write(
                "cosyvoice2 precision "
                f"requested={_precision_requested} active={_precision_active} "
                f"strategy={_load_strategy} "
                f"peak_reserved_mib={_load_peak_reserved_mib} "
                f"fallback={_precision_fallback_reason or 'none'}\n"
            )
            sys.stderr.flush()
            return model, ""
    except Exception as exc:  # noqa: BLE001
        _record_cuda_load_peak()
        gc.collect()
        _empty_cuda_cache()
        _last_load_error = f"cosyvoice_load_failed:{exc}"
        return None, _last_load_error


def _tensor_to_pcm_base64(audio_tensor: Any, sample_rate: int) -> tuple[str, int]:
    import base64
    import struct

    if hasattr(audio_tensor, "detach"):
        audio_tensor = audio_tensor.detach().cpu()
    if audio_tensor.ndim > 1:
        audio_tensor = audio_tensor.squeeze()
    samples = audio_tensor.numpy().tolist()
    pcm = b"".join(
        struct.pack("<h", max(-32768, min(32767, int(float(s) * 32767))))
        for s in samples
    )
    return base64.b64encode(pcm).decode("ascii"), sample_rate


def _open_synthesis_iterator(
    model: Any,
    *,
    text: str,
    emo_text: str,
    ref_audio: str,
    ref_text: str,
    speed: float,
    stream: bool,
) -> tuple[Any | None, str, str]:
    """Return (iterator, error_reason, error_message)."""
    cleaned = text.strip()
    if not cleaned:
        return None, "empty_text", ""
    speed = max(0.5, min(2.0, float(speed or 1.0)))
    prompt_text = ref_text.strip() or cleaned[:32]
    if emo_text.strip():
        prompt_path = _resolve_prompt_wav(ref_audio)
        if prompt_path is None:
            return (
                None,
                "prompt_wav_missing",
                "Provide ref_audio or install CosyVoice asset/zero_shot_prompt.wav",
            )
        instruct_text = _format_instruct_text(emo_text)
        speaker_id, _cache_hit, prepare_error = _prepare_speaker(
            model,
            prompt_path,
            instruct_text,
        )
        if prepare_error:
            sys.stderr.write(f"cosyvoice2 prompt cache fallback reason={prepare_error}\n")
            sys.stderr.flush()
        iterator = model.inference_instruct2(
            cleaned,
            instruct_text,
            str(prompt_path),
            zero_shot_spk_id=speaker_id,
            stream=stream,
            speed=speed,
        )
        return iterator, "", ""
    if ref_audio and Path(ref_audio).is_file():
        prompt_path = Path(ref_audio)
        speaker_id, _cache_hit, prepare_error = _prepare_speaker(
            model,
            prompt_path,
            prompt_text,
        )
        if prepare_error:
            sys.stderr.write(f"cosyvoice2 prompt cache fallback reason={prepare_error}\n")
            sys.stderr.flush()
        iterator = model.inference_zero_shot(
            cleaned,
            prompt_text,
            str(prompt_path),
            zero_shot_spk_id=speaker_id,
            stream=stream,
            speed=speed,
        )
        return iterator, "", ""
    return None, "ref_audio_missing", "Place role-pack ref wav or set emo_text"


def _normalize_synthesis_inputs(
    *,
    text: str,
    emo_text: str,
    ref_audio: str,
    ref_text: str,
    speed: float,
) -> tuple[str, str, str, str, float]:
    cleaned = text.strip()
    emo = emo_text.strip()
    ref = ref_audio.strip()
    if not emo and not (ref and Path(ref).is_file()):
        emo = "用自然平静的语气"
    return cleaned, emo, ref, ref_text.strip(), max(0.5, min(2.0, float(speed or 1.0)))


def _chunk_tensor_from_item(item: Any) -> Any | None:
    if isinstance(item, dict):
        for key in ("tts_speech", "yield_speech", "speech"):
            if key in item:
                return item[key]
    if hasattr(item, "shape"):
        return item
    return None


def _collect_synthesis_tensors(
    model: Any,
    *,
    text: str,
    emo_text: str,
    ref_audio: str,
    ref_text: str,
    speed: float,
) -> tuple[list[Any], str, str]:
    """Run the reliable whole-utterance path. Caller holds _synth_lock."""
    tensors: list[Any] = []
    iterator, err, msg = _open_synthesis_iterator(
        model,
        text=text,
        emo_text=emo_text,
        ref_audio=ref_audio,
        ref_text=ref_text,
        speed=speed,
        stream=False,
    )
    if iterator is None:
        return [], err, msg
    for item in iterator:
        tensor = _chunk_tensor_from_item(item)
        if tensor is not None:
            tensors.append(tensor)
    if tensors:
        return tensors, "", ""
    return [], "cosyvoice_empty", "CosyVoice returned no audio tensors"


def _prime_cosyvoice_model(
    model: Any,
    *,
    emo_text: str = "用自然平静的语气",
    ref_audio: str = "",
    ref_text: str = "",
) -> tuple[bool, int, str]:
    """One-shot dummy synthesis to compile the inference graph (output discarded)."""
    global _primed, _prime_failed_reason
    if _primed:
        return True, 0, ""
    prompt_path = _resolve_prompt_wav(ref_audio)
    if prompt_path is None:
        return False, 0, "prompt_wav_missing"
    started = time.perf_counter()
    with _synth_lock:
        if _primed:
            return True, 0, ""
        try:
            iterator, err, msg = _open_synthesis_iterator(
                model,
                text="你好呀，今天也会陪着你。",
                emo_text=emo_text or "用自然平静的语气",
                ref_audio=ref_audio or str(prompt_path),
                ref_text=ref_text,
                speed=1.0,
                # Match the reliable Windows synthesis path. Some CosyVoice2
                # builds need one whole-utterance pass before low-latency streaming.
                stream=False,
            )
            if iterator is None:
                raise RuntimeError(msg or err or "prime_iterator_unavailable")
            produced = any(_chunk_tensor_from_item(item) is not None for item in iterator)
            if not produced:
                raise RuntimeError("prime_returned_no_audio")
            _primed = True
            _prime_failed_reason = ""
            prime_ms = int((time.perf_counter() - started) * 1000)
            sys.stderr.write(f"cosyvoice2 prime ok prime_ms={prime_ms}\n")
            sys.stderr.flush()
            return True, prime_ms, ""
        except Exception as exc:  # noqa: BLE001
            _prime_failed_reason = f"prime_failed:{exc}"
            return False, int((time.perf_counter() - started) * 1000), _prime_failed_reason


def _synthesize_with_model(
    model: Any,
    *,
    text: str,
    emo_text: str,
    ref_audio: str,
    ref_text: str,
    speed: float,
) -> dict[str, Any]:
    import base64
    import io

    cleaned = text.strip()
    if not cleaned:
        return {"ok": False, "reason": "empty_text", "audio_base64": ""}

    started = time.perf_counter()
    with _synth_lock:
        tensors, err, msg = _collect_synthesis_tensors(
            model,
            text=cleaned,
            emo_text=emo_text,
            ref_audio=ref_audio,
            ref_text=ref_text,
            speed=speed,
        )
    if not tensors:
        return {"ok": False, "reason": err, "message": msg, "audio_base64": ""}

    audio_tensor = tensors[-1]

    elapsed_ms = int((time.perf_counter() - started) * 1000)
    sys.stderr.write(f"cosyvoice2 synthesize ok elapsed_ms={elapsed_ms} text_len={len(cleaned)}\n")
    sys.stderr.flush()

    sample_rate = int(getattr(model, "sample_rate", 22050))
    pcm_b64, sample_rate = _tensor_to_pcm_base64(audio_tensor, sample_rate)
    pcm = base64.b64decode(pcm_b64)
    buf = io.BytesIO()
    with wave.open(buf, "wb") as wf:
        wf.setnchannels(1)
        wf.setsampwidth(2)
        wf.setframerate(sample_rate)
        wf.writeframes(pcm)
    return {
        "ok": True,
        "audio_base64": base64.b64encode(buf.getvalue()).decode("ascii"),
        "sample_rate": sample_rate,
        "engine": _ENGINE,
        "elapsed_ms": elapsed_ms,
    }


def _stream_synthesis_lines(
    model: Any,
    *,
    text: str,
    emo_text: str,
    ref_audio: str,
    ref_text: str,
    speed: float,
):
    cleaned = text.strip()
    if not cleaned:
        yield json.dumps({"ok": False, "reason": "empty_text", "event": "error"}, ensure_ascii=False)
        return
    started = time.perf_counter()
    sample_rate = int(getattr(model, "sample_rate", 22050))
    stream_enabled = os.environ.get("OCLIVE_COSYVOICE_STREAM", "1").strip().lower() not in {
        "0",
        "false",
        "no",
    }
    # Upstream CosyVoice2 ignores speed in its streaming branch. Preserve the
    # voice-director contract by using the buffered path for non-default speed.
    use_stream = stream_enabled and abs(speed - 1.0) < 0.001
    with _synth_lock:
        iterator, err, msg = _open_synthesis_iterator(
            model,
            text=cleaned,
            emo_text=emo_text,
            ref_audio=ref_audio,
            ref_text=ref_text,
            speed=speed,
            stream=use_stream,
        )
        if iterator is None:
            yield json.dumps(
                {"ok": False, "reason": err, "message": msg, "event": "error"},
                ensure_ascii=False,
            )
            return
        chunk_index = 0
        first_chunk_ms: int | None = None
        try:
            for item in iterator:
                tensor = _chunk_tensor_from_item(item)
                if tensor is None:
                    continue
                if first_chunk_ms is None:
                    first_chunk_ms = int((time.perf_counter() - started) * 1000)
                pcm_b64, sr = _tensor_to_pcm_base64(tensor, sample_rate)
                yield json.dumps(
                    {
                        "ok": True,
                        "event": "chunk",
                        "chunk_index": chunk_index,
                        "pcm_base64": pcm_b64,
                        "sample_rate": sr,
                    },
                    ensure_ascii=False,
                )
                chunk_index += 1
        except Exception as exc:  # noqa: BLE001
            yield json.dumps(
                {
                    "ok": False,
                    "reason": "cosyvoice_inference_failed",
                    "message": str(exc),
                    "event": "error",
                },
                ensure_ascii=False,
            )
            return
    if chunk_index == 0:
        yield json.dumps(
            {
                "ok": False,
                "reason": "cosyvoice_empty",
                "message": "CosyVoice returned no audio tensors",
                "event": "error",
            },
            ensure_ascii=False,
        )
        return
    elapsed_ms = int((time.perf_counter() - started) * 1000)
    stream_mode = "streaming" if use_stream else "buffered"
    sys.stderr.write(
        f"cosyvoice2 stream ok elapsed_ms={elapsed_ms} ttfc_ms={first_chunk_ms or elapsed_ms} "
        f"chunks={chunk_index} text_len={len(cleaned)} mode={stream_mode}\n"
    )
    sys.stderr.flush()
    yield json.dumps(
        {
            "ok": True,
            "event": "done",
            "chunks": chunk_index,
            "elapsed_ms": elapsed_ms,
            "ttfc_ms": first_chunk_ms or elapsed_ms,
            "sample_rate": sample_rate,
            "engine": _ENGINE,
            "stream_mode": stream_mode,
        },
        ensure_ascii=False,
    )


def _load_telemetry_payload() -> dict[str, Any]:
    return {
        "load_strategy": _load_strategy,
        "load_admission_detail": _load_admission_detail,
        "load_vram_probe": _load_vram_probe,
        "load_free_vram_before_mib": _load_free_vram_before_mib,
        "load_total_vram_mib": _load_total_vram_mib,
        "load_min_free_vram_mib": _load_min_free_vram_mib,
        "load_peak_allocated_mib": _load_peak_allocated_mib,
        "load_peak_reserved_mib": _load_peak_reserved_mib,
    }


def health_payload() -> dict[str, Any]:
    path = _model_dir or _resolve_model_dir()
    ready, reason = _model_ready(path)
    cosy = _model is not None
    healthy = ready and cosy and not _prime_failed_reason
    failure_reason = _prime_failed_reason or _last_load_error or reason or "not_warmed"
    if healthy:
        message = "CosyVoice2 sidecar ready"
    elif failure_reason == "gpu_admission_denied":
        message = (
            "CosyVoice2 cold load deferred because GPU headroom is below the safe threshold"
        )
    elif _prime_failed_reason:
        message = (
            "CosyVoice2 prime failed; restart the sidecar after fixing the inference environment"
        )
    else:
        message = "Import CosyVoice2 model pack or install cosyvoice package"
    return {
        "ok": healthy,
        "engine": _ENGINE,
        "model_dir": str(path),
        "warmed": _warmed,
        "primed": _primed,
        "prepared_speakers": len(_prepared_speakers),
        "precision_requested": _precision_requested,
        "precision_active": _precision_active,
        "precision_fallback_reason": _precision_fallback_reason,
        **_load_telemetry_payload(),
        "retryable": failure_reason == "gpu_admission_denied",
        "reason": "" if healthy else failure_reason,
        "message": message,
    }


class Handler(BaseHTTPRequestHandler):
    def log_message(self, format: str, *args: object) -> None:
        return

    def _send_cors_headers(self) -> None:
        """Loopback sidecar consumed by the Tauri webview via cross-origin fetch."""
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Content-Type")

    def _json(self, code: int, payload: dict[str, Any]) -> None:
        body = json.dumps(payload, ensure_ascii=False).encode("utf-8")
        self.send_response(code)
        self._send_cors_headers()
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def _ndjson_stream(self, lines) -> None:
        self.send_response(200)
        self._send_cors_headers()
        self.send_header("Content-Type", "application/x-ndjson; charset=utf-8")
        self.send_header("Cache-Control", "no-cache")
        self.end_headers()
        iterator = iter(lines)
        try:
            for line in iterator:
                self.wfile.write((line + "\n").encode("utf-8"))
                self.wfile.flush()
        except (BrokenPipeError, ConnectionResetError):
            # Chat cancellation is expected. Explicitly closing the generator
            # releases its synthesis lock before the next queued utterance.
            return
        finally:
            close = getattr(iterator, "close", None)
            if callable(close):
                close()

    def do_OPTIONS(self) -> None:
        self.send_response(204)
        self._send_cors_headers()
        self.send_header("Content-Length", "0")
        self.end_headers()

    def do_GET(self) -> None:
        if self.path.rstrip("/") == "/health":
            self._json(200, health_payload())
            return
        self._json(404, {"ok": False, "reason": "not_found"})

    def do_POST(self) -> None:
        length = int(self.headers.get("Content-Length", "0") or "0")
        raw = self.rfile.read(length) if length else b"{}"
        try:
            body = json.loads(raw.decode("utf-8") or "{}")
        except json.JSONDecodeError:
            self._json(400, {"ok": False, "reason": "bad_json"})
            return

        if self.path.rstrip("/") == "/warm":
            global _model_dir, _prime_failed_reason
            md = str(body.get("model_dir") or "").strip()
            if md:
                _model_dir = Path(md)
            path = _model_dir or _resolve_model_dir()
            load_started = time.perf_counter()
            model, err = _load_cosyvoice_model(
                path,
                body.get("host_resource_admission"),
            )
            load_ms = int((time.perf_counter() - load_started) * 1000)
            if model is None:
                failure = health_payload()
                failure.update(
                    {
                        "ok": False,
                        "reason": err,
                        "engine": _ENGINE,
                        "load_ms": load_ms,
                    }
                )
                self._json(200, failure)
                return
            prime = body.get("prime", True)
            if isinstance(prime, str):
                prime = prime.strip().lower() not in {"0", "false", "no"}
            prime_ms = 0
            primed = _primed
            prime_reason = ""
            ref_audio = str(body.get("ref_audio") or "").strip()
            ref_text = str(body.get("ref_text") or "").strip()
            emo_text = str(body.get("emo_text") or "").strip()
            prepare_ms = 0
            prompt_prepared = False
            prompt_cache_hit = False
            prepare_reason = ""
            if ref_audio or emo_text:
                prompt_path = _resolve_prompt_wav(ref_audio)
                if prompt_path is None:
                    prepare_reason = "prompt_wav_missing"
                else:
                    prompt_text = (
                        _format_instruct_text(emo_text)
                        if emo_text
                        else (ref_text or "你好呀，今天也会陪着你。")
                    )
                    prepare_started = time.perf_counter()
                    with _synth_lock:
                        speaker_id, prompt_cache_hit, prepare_reason = _prepare_speaker(
                            model,
                            prompt_path,
                            prompt_text,
                        )
                    prepare_ms = int((time.perf_counter() - prepare_started) * 1000)
                    prompt_prepared = bool(speaker_id) and not prepare_reason
            if prime and not _primed:
                _prime_failed_reason = ""
                ok_prime, prime_ms, prime_reason = _prime_cosyvoice_model(
                    model,
                    emo_text=emo_text or "用自然平静的语气",
                    ref_audio=ref_audio,
                    ref_text=ref_text,
                )
                primed = ok_prime
                if (
                    not ok_prime
                    and prime_reason.startswith("prime_failed:")
                    and _fallback_model_to_fp32(model, prime_reason)
                ):
                    retry_ok, retry_ms, retry_reason = _prime_cosyvoice_model(
                        model,
                        emo_text=emo_text or "用自然平静的语气",
                        ref_audio=ref_audio,
                        ref_text=ref_text,
                    )
                    prime_ms += retry_ms
                    primed = retry_ok
                    prime_reason = retry_reason
            sys.stderr.write(
                f"cosyvoice2 warm ok load_ms={load_ms} prime_ms={prime_ms} "
                f"primed={primed} precision={_precision_active}\n"
            )
            sys.stderr.flush()
            self._json(
                200,
                {
                    "ok": (not prime) or primed,
                    "engine": _ENGINE,
                    "warmed": True,
                    "primed": primed,
                    "prime_ms": prime_ms,
                    "prime_reason": prime_reason,
                    "prompt_prepared": prompt_prepared,
                    "prompt_cache_hit": prompt_cache_hit,
                    "prompt_prepare_ms": prepare_ms,
                    "prompt_prepare_reason": prepare_reason,
                    "model_dir": str(path),
                    "load_ms": load_ms,
                    "precision_requested": _precision_requested,
                    "precision_active": _precision_active,
                    "precision_fallback_reason": _precision_fallback_reason,
                    **_load_telemetry_payload(),
                },
            )
            return

        if self.path.rstrip("/") == "/synthesize":
            path = _model_dir or _resolve_model_dir()
            model, err = _load_cosyvoice_model(path)
            if model is None:
                self._json(
                    200,
                    {"ok": False, "reason": err, "audio_base64": "", "engine": _ENGINE},
                )
                return
            cleaned, emo, ref, ref_t, spd = _normalize_synthesis_inputs(
                text=str(body.get("text") or ""),
                emo_text=str(body.get("emo_text") or ""),
                ref_audio=str(body.get("ref_audio") or ""),
                ref_text=str(body.get("ref_text") or ""),
                speed=float(body.get("speed") or 1.0),
            )
            result = _synthesize_with_model(
                model,
                text=cleaned,
                emo_text=emo,
                ref_audio=ref,
                ref_text=ref_t,
                speed=spd,
            )
            self._json(200, result)
            return

        if self.path.rstrip("/") == "/synthesize/stream":
            path = _model_dir or _resolve_model_dir()
            model, err = _load_cosyvoice_model(path)
            if model is None:
                self._ndjson_stream(
                    [
                        json.dumps(
                            {"ok": False, "reason": err, "event": "error", "engine": _ENGINE},
                            ensure_ascii=False,
                        ),
                    ],
                )
                return
            cleaned, emo, ref, ref_t, spd = _normalize_synthesis_inputs(
                text=str(body.get("text") or ""),
                emo_text=str(body.get("emo_text") or ""),
                ref_audio=str(body.get("ref_audio") or ""),
                ref_text=str(body.get("ref_text") or ""),
                speed=float(body.get("speed") or 1.0),
            )
            self._ndjson_stream(
                _stream_synthesis_lines(
                    model,
                    text=cleaned,
                    emo_text=emo,
                    ref_audio=ref,
                    ref_text=ref_t,
                    speed=spd,
                ),
            )
            return

        self._json(404, {"ok": False, "reason": "not_found"})


def main() -> None:
    port = int(os.environ.get("OCLIVE_COSYVOICE_PORT", "50000") or "50000")
    server = ThreadingHTTPServer(("127.0.0.1", port), Handler)
    url = f"http://127.0.0.1:{port}"
    sys.stdout.write(f"OCLIVE_SIDECAR_READY {url}\n")
    sys.stdout.flush()
    server.serve_forever()


if __name__ == "__main__":
    main()
