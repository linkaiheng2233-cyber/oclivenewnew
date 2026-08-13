#!/usr/bin/env python3
"""Stress the shared-GPU llama-server + CosyVoice2 runtime.

Normal mode starts llama-server first, cold-loads CosyVoice, then runs repeated
LLM/TTS pairs. Admission mode proves that an unsafe cold load is rejected
without disturbing the already-running LLM.
"""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
import json
import os
import socket
import subprocess
import tempfile
import threading
import time
import urllib.request
from pathlib import Path
from typing import Any


CREATE_NO_WINDOW = getattr(subprocess, "CREATE_NO_WINDOW", 0)
IS_WINDOWS = os.name == "nt"


def parse_args() -> argparse.Namespace:
    repo = Path(__file__).resolve().parent.parent
    workspace = repo.parent
    default_python = (
        repo
        / "examples"
        / "voice-loop-minimal"
        / ".venv-cosyvoice"
        / "Scripts"
        / "python.exe"
    )
    default_model_dir = (
        Path(os.environ.get("APPDATA", ""))
        / "OCLive"
        / "models"
        / "tts"
        / "cosyvoice2-0.5b"
    )
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--voice-python",
        type=Path,
        default=Path(os.environ.get("OCLIVE_VOICE_PYTHON", default_python)),
    )
    parser.add_argument(
        "--voice-model-dir",
        type=Path,
        default=Path(
            os.environ.get("OCLIVE_COSYVOICE_MODEL_DIR", default_model_dir)
        ),
    )
    parser.add_argument(
        "--ref-audio",
        type=Path,
        default=repo
        / "distros"
        / "chat-pro"
        / "roles"
        / "mumu"
        / "assets"
        / "voice"
        / "ref_neutral.wav",
    )
    parser.add_argument(
        "--llama-server",
        type=Path,
        default=Path(
            os.environ.get(
                "OCLIVE_LLAMA_SERVER_PATH",
                workspace
                / "components"
                / "llama.cpp"
                / "b10090-cuda12"
                / "llama-server.exe",
            )
        ),
    )
    parser.add_argument(
        "--llm-model",
        type=Path,
        default=Path(
            os.environ.get(
                "OCLIVE_LOCAL_LLM_MODEL_PATH",
                workspace
                / "models"
                / "qwen2.5-7b-instruct-q4_k_m"
                / "qwen2.5-7b-instruct-q4_k_m.gguf",
            )
        ),
    )
    parser.add_argument("--gpu-index", default="0")
    parser.add_argument("--gpu-layers", type=int, default=22)
    parser.add_argument(
        "--llm-priority",
        type=int,
        choices=(-1, 0, 1, 2, 3),
        default=None,
        help="Optional llama.cpp --prio override used by scheduling benchmarks",
    )
    parser.add_argument(
        "--llm-poll",
        type=int,
        choices=range(0, 101),
        default=None,
        metavar="0..100",
        help="Optional llama.cpp --poll override used by scheduling benchmarks",
    )
    parser.add_argument(
        "--voice-first-chunk-priority",
        choices=("on", "off"),
        default="on",
        help="Enable or disable the CosyVoice first-chunk CUDA priority policy",
    )
    parser.add_argument(
        "--voice-first-hop-tokens",
        type=int,
        default=20,
        help="Initial CosyVoice stream hop when first-chunk priority is enabled",
    )
    parser.add_argument("--voice-runs", type=int, default=5)
    parser.add_argument(
        "--duration-minutes",
        type=float,
        default=0.0,
        help="Run real wall-clock minutes after warmup; 0 keeps --voice-runs mode",
    )
    parser.add_argument(
        "--gpu-sample-interval-seconds",
        type=float,
        default=1.0,
    )
    parser.add_argument("--max-gpu-sample-failures", type=int, default=0)
    parser.add_argument(
        "--checkpoint-interval-seconds",
        type=float,
        default=60.0,
        help="Atomically refresh bounded progress evidence when --output is set",
    )
    parser.add_argument("--llm-max-tokens", type=int, default=64)
    parser.add_argument(
        "--voice-text",
        default="爸爸，今天辛苦啦。先喝口水，我们慢慢来。",
        help="Static TTS probe text; evidence records only its character count",
    )
    parser.add_argument(
        "--voice-emo-text",
        default="用温暖、自然、关心的语气",
        help="CosyVoice instruction used by warmup and measured requests",
    )
    parser.add_argument(
        "--voice-ref-text",
        default="早上好呀，我是沐沐。",
        help="Reference transcript paired with --ref-audio",
    )
    parser.add_argument("--min-headroom-mib", type=int, default=768)
    parser.add_argument("--max-voice-ttfc-ms", type=int, default=8000)
    parser.add_argument("--max-llm-ttft-ms", type=int, default=5000)
    parser.add_argument("--max-steady-growth-mib", type=int, default=256)
    parser.add_argument("--expect-admission-denied", action="store_true")
    parser.add_argument(
        "--output",
        type=Path,
        help="Atomically write the schema-v3 JSON evidence to this path",
    )
    return parser.parse_args()


def free_port() -> int:
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return int(sock.getsockname()[1])


def direct_opener() -> urllib.request.OpenerDirector:
    return urllib.request.build_opener(urllib.request.ProxyHandler({}))


def open_json(
    url: str,
    payload: dict[str, Any] | None = None,
    *,
    timeout: float = 60,
):
    data = (
        None
        if payload is None
        else json.dumps(payload, ensure_ascii=True).encode("ascii")
    )
    request = urllib.request.Request(
        url,
        data=data,
        headers={"Content-Type": "application/json"},
    )
    return direct_opener().open(request, timeout=timeout)


def wait_ready(url: str, timeout: float) -> dict[str, Any]:
    deadline = time.time() + timeout
    last_error: Exception | None = None
    while time.time() < deadline:
        try:
            with open_json(url, timeout=2) as response:
                return json.load(response)
        except Exception as exc:  # noqa: BLE001
            last_error = exc
            time.sleep(0.25)
    raise RuntimeError(f"runtime not ready: {url}: {last_error}")


def gpu_memory_mib(gpu_index: str) -> tuple[int, int]:
    result = subprocess.run(
        [
            "nvidia-smi",
            "-i",
            gpu_index,
            "--query-gpu=memory.used,memory.total",
            "--format=csv,noheader,nounits",
        ],
        check=True,
        capture_output=True,
        text=True,
        timeout=5,
        creationflags=CREATE_NO_WINDOW,
    )
    used_raw, total_raw = (
        part.strip() for part in result.stdout.splitlines()[0].split(",", 1)
    )
    return int(used_raw), int(total_raw)


class GpuSampler:
    def __init__(self, gpu_index: str, interval_seconds: float) -> None:
        self.gpu_index = gpu_index
        self.interval_seconds = interval_seconds
        self.peak_used_mib = 0
        self.total_mib = 0
        self.sample_count = 0
        self.sample_failures = 0
        self.thread_joined = False
        self._stop = threading.Event()
        self._thread = threading.Thread(target=self._sample, daemon=True)

    def _sample(self) -> None:
        while not self._stop.is_set():
            try:
                used, total = gpu_memory_mib(self.gpu_index)
                self.peak_used_mib = max(self.peak_used_mib, used)
                self.total_mib = total
                self.sample_count += 1
            except (OSError, subprocess.SubprocessError, ValueError, IndexError):
                self.sample_failures += 1
            self._stop.wait(self.interval_seconds)

    def start(self) -> None:
        used, total = gpu_memory_mib(self.gpu_index)
        self.peak_used_mib = used
        self.total_mib = total
        self.sample_count = 1
        self._thread.start()

    def stop(self) -> None:
        self._stop.set()
        self._thread.join(timeout=5)
        self.thread_joined = not self._thread.is_alive()


def terminate(process: subprocess.Popen[Any] | None) -> dict[str, Any]:
    if process is None:
        return {
            "started": False,
            "pid": None,
            "returncode": None,
            "reaped": True,
            "tree_termination": {
                "attempted": False,
                "method": "none",
                "returncode": None,
                "succeeded": True,
            },
        }
    pid = process.pid
    tree_termination = {
        "attempted": False,
        "method": "single_process",
        "returncode": None,
        "succeeded": True,
    }
    try:
        if process.poll() is None:
            if IS_WINDOWS:
                tree_termination.update(
                    {
                        "attempted": True,
                        "method": "taskkill_tree",
                    }
                )
                result = subprocess.run(
                    ["taskkill", "/PID", str(pid), "/T", "/F"],
                    check=False,
                    capture_output=True,
                    text=True,
                    timeout=15,
                    creationflags=CREATE_NO_WINDOW,
                )
                tree_termination["returncode"] = result.returncode
                tree_termination["succeeded"] = result.returncode == 0
                if result.returncode != 0 and process.poll() is None:
                    process.terminate()
            else:
                process.terminate()
        process.wait(timeout=10)
    except subprocess.TimeoutExpired:
        try:
            process.kill()
            process.wait(timeout=5)
        except (OSError, subprocess.SubprocessError):
            pass
    except OSError:
        pass
    return {
        "started": True,
        "pid": pid,
        "returncode": process.returncode,
        "reaped": process.poll() is not None,
        "tree_termination": tree_termination,
    }


def run_llm(
    endpoint: str,
    gate: threading.Event,
    max_tokens: int,
) -> dict[str, Any]:
    prompt = (
        "Analyze safe shared-GPU scheduling for a local language model and "
        "streaming speech synthesis. Keep the answer concise. "
    ) * 80
    payload = {
        "model": "local",
        "messages": [{"role": "user", "content": prompt}],
        "max_tokens": max_tokens,
        "temperature": 0.2,
        "stream": True,
    }
    gate.wait()
    started = time.perf_counter()
    first = None
    events = 0
    with open_json(
        f"{endpoint}/v1/chat/completions",
        payload,
        timeout=120,
    ) as response:
        for raw in response:
            line = raw.decode("utf-8", "replace").strip()
            if not line.startswith("data:") or line == "data: [DONE]":
                continue
            event = json.loads(line[5:].strip())
            content = (
                event.get("choices", [{}])[0].get("delta", {}).get("content", "")
            )
            if content:
                first = first or time.perf_counter()
                events += 1
    ended = time.perf_counter()
    return {
        "ttft_ms": round((first - started) * 1000) if first else None,
        "total_ms": round((ended - started) * 1000),
        "events": events,
    }


def run_voice(
    endpoint: str,
    gate: threading.Event,
    ref_audio: Path,
    text: str,
    emo_text: str,
    ref_text: str,
) -> dict[str, Any]:
    payload = {
        "text": text,
        "emo_text": emo_text,
        "ref_audio": str(ref_audio),
        "ref_text": ref_text,
        "speed": 1.0,
    }
    gate.wait()
    started = time.perf_counter()
    first = None
    chunks = 0
    done: dict[str, Any] = {}
    first_chunk_meta: dict[str, Any] = {}
    errors: list[dict[str, Any]] = []
    with open_json(
        f"{endpoint}/synthesize/stream",
        payload,
        timeout=120,
    ) as response:
        for raw in response:
            event = json.loads(raw.decode("utf-8"))
            if event.get("event") == "chunk":
                if first is None:
                    first = time.perf_counter()
                    first_chunk_meta = event
                chunks += 1
            elif event.get("event") == "done":
                done = event
            elif event.get("event") == "error":
                errors.append(
                    {
                        key: value
                        for key, value in event.items()
                        if key not in {"pcm_base64", "audio_base64"}
                    }
                )
    ended = time.perf_counter()
    client_ttfc_ms = round((first - started) * 1000) if first else None
    timings_ms = done.get("timings_ms") or first_chunk_meta.get("timings_ms") or {}
    server_payload_ready_ms = timings_ms.get("server_payload_ready")
    client_delivery_overhead_ms = None
    if client_ttfc_ms is not None and isinstance(server_payload_ready_ms, int):
        client_delivery_overhead_ms = max(
            0,
            client_ttfc_ms - server_payload_ready_ms,
        )
    return {
        "ttfc_ms": client_ttfc_ms,
        "total_ms": round((ended - started) * 1000),
        "chunks": chunks,
        "sidecar_ttfc_ms": done.get("ttfc_ms"),
        "stream_mode": done.get("stream_mode"),
        "timings_schema_version": (
            done.get("timings_schema_version")
            or first_chunk_meta.get("timings_schema_version")
        ),
        "timings_ms": timings_ms,
        "prompt_cache_hit": (
            done.get("prompt_cache_hit")
            if "prompt_cache_hit" in done
            else first_chunk_meta.get("prompt_cache_hit")
        ),
        "first_chunk_priority_applied": (
            done.get("first_chunk_priority_applied")
            if "first_chunk_priority_applied" in done
            else first_chunk_meta.get("first_chunk_priority_applied")
        ),
        "first_chunk_hop_tokens": (
            done.get("first_chunk_hop_tokens")
            or first_chunk_meta.get("first_chunk_hop_tokens")
            or 0
        ),
        "client_delivery_overhead_ms": client_delivery_overhead_ms,
        "errors": errors,
    }


def run_pair(
    llm_endpoint: str,
    voice_endpoint: str,
    ref_audio: Path,
    max_tokens: int,
    voice_text: str,
    voice_emo_text: str,
    voice_ref_text: str,
) -> tuple[dict[str, Any], dict[str, Any]]:
    gate = threading.Event()
    results: dict[str, dict[str, Any]] = {}
    failures: list[BaseException] = []

    def collect(name: str, target, *target_args) -> None:
        try:
            results[name] = target(*target_args)
        except BaseException as exc:  # noqa: BLE001
            failures.append(exc)

    threads = [
        threading.Thread(
            target=collect,
            args=("llm", run_llm, llm_endpoint, gate, max_tokens),
        ),
        threading.Thread(
            target=collect,
            args=(
                "voice",
                run_voice,
                voice_endpoint,
                gate,
                ref_audio,
                voice_text,
                voice_emo_text,
                voice_ref_text,
            ),
        ),
    ]
    for thread in threads:
        thread.start()
    gate.set()
    for thread in threads:
        thread.join(timeout=150)
    if any(thread.is_alive() for thread in threads):
        raise TimeoutError("concurrent stress pair did not finish within 150 seconds")
    if failures:
        raise RuntimeError(f"concurrent stress pair failed: {failures[0]}")
    return results["llm"], results["voice"]


def percentile(values: list[int], fraction: float) -> int | None:
    if not values:
        return None
    ordered = sorted(values)
    index = max(0, int((len(ordered) * fraction) + 0.999999) - 1)
    return ordered[index]


def timing_distribution(values: list[int]) -> dict[str, Any]:
    return {
        "samples": values,
        "p50": percentile(values, 0.5),
        "p95": percentile(values, 0.95),
        "max": max(values) if values else None,
    }


def validate_inputs(args: argparse.Namespace) -> None:
    for label, path in (
        ("voice python", args.voice_python),
        ("voice model", args.voice_model_dir),
        ("reference audio", args.ref_audio),
        ("llama-server", args.llama_server),
        ("LLM model", args.llm_model),
    ):
        if not path.exists():
            raise FileNotFoundError(f"{label} not found: {path}")
    if args.voice_runs < 1:
        raise ValueError("--voice-runs must be at least 1")
    if args.voice_first_hop_tokens < 1:
        raise ValueError("--voice-first-hop-tokens must be at least 1")
    if not (args.duration_minutes >= 0 and args.duration_minutes < float("inf")):
        raise ValueError("--duration-minutes must be finite and at least 0")
    if args.gpu_sample_interval_seconds <= 0:
        raise ValueError("--gpu-sample-interval-seconds must be greater than 0")
    if args.max_gpu_sample_failures < 0:
        raise ValueError("--max-gpu-sample-failures must be at least 0")
    if args.checkpoint_interval_seconds <= 0:
        raise ValueError("--checkpoint-interval-seconds must be greater than 0")
    if not args.voice_text.strip():
        raise ValueError("--voice-text must not be empty")
    if not args.voice_emo_text.strip():
        raise ValueError("--voice-emo-text must not be empty")


def emit_summary(summary: dict[str, Any], output: Path | None) -> None:
    encoded = (json.dumps(summary, ensure_ascii=False, indent=2) + "\n").encode("utf-8")
    if output is None:
        print(encoded.decode("utf-8"), end="")
        return

    output.parent.mkdir(parents=True, exist_ok=True)
    temporary_path: Path | None = None
    try:
        with tempfile.NamedTemporaryFile(
            mode="wb", dir=output.parent, prefix=f".{output.name}.", delete=False
        ) as temporary:
            temporary_path = Path(temporary.name)
            temporary.write(encoded)
            temporary.flush()
            os.fsync(temporary.fileno())
        os.replace(temporary_path, output)
        temporary_path = None
    finally:
        if temporary_path is not None:
            temporary_path.unlink(missing_ok=True)


def checkpoint_path(output: Path | None) -> Path | None:
    if output is None:
        return None
    suffix = output.suffix or ".json"
    return output.with_name(f"{output.stem}.checkpoint{suffix}")


def process_checkpoint(process: subprocess.Popen | None) -> dict[str, Any]:
    return {
        "started": process is not None,
        "pid": process.pid if process is not None else None,
        "alive": process is not None and process.poll() is None,
    }


def emit_progress_checkpoint(
    output: Path | None,
    *,
    status: str,
    started_at: str,
    elapsed_seconds: float,
    args: argparse.Namespace,
    llm_samples: list[dict[str, Any]],
    voice_samples: list[dict[str, Any]],
    sampler: "GpuSampler",
    llm_process: subprocess.Popen | None,
    voice_process: subprocess.Popen | None,
    failures: list[str],
    cleanup: dict[str, Any] | None = None,
) -> None:
    path = checkpoint_path(output)
    if path is None:
        return
    latest_llm = llm_samples[-1] if llm_samples else {}
    latest_voice = voice_samples[-1] if voice_samples else {}
    checkpoint = {
        "schema_version": 1,
        "status": status,
        "started_at": started_at,
        "updated_at": datetime.now(timezone.utc).isoformat(),
        "elapsed_seconds": round(max(0.0, elapsed_seconds), 3),
        "requested_duration_minutes": args.duration_minutes,
        "gpu_layers": args.gpu_layers,
        "pairs_completed": len(voice_samples),
        "latest": {
            "llm_ttft_ms": latest_llm.get("ttft_ms"),
            "voice_ttfc_ms": latest_voice.get("ttfc_ms"),
            "voice_timings_ms": latest_voice.get("timings_ms") or {},
            "voice_chunks": latest_voice.get("chunks"),
        },
        "gpu_sampler": {
            "samples": sampler.sample_count,
            "failures": sampler.sample_failures,
            "peak_used_mib": sampler.peak_used_mib,
        },
        "processes": {
            "llm": process_checkpoint(llm_process),
            "voice": process_checkpoint(voice_process),
        },
        "failure_count": len(failures),
        "last_failure": failures[-1] if failures else None,
    }
    if cleanup is not None:
        checkpoint["cleanup"] = cleanup
    emit_summary(checkpoint, path)


def main() -> int:
    args = parse_args()
    validate_inputs(args)
    repo = Path(__file__).resolve().parent.parent
    voice_root = repo / "examples" / "voice-loop-minimal"
    llm_port = free_port()
    voice_port = free_port()
    llm_endpoint = f"http://127.0.0.1:{llm_port}"
    voice_endpoint = f"http://127.0.0.1:{voice_port}"
    llm_process = None
    voice_process = None
    sampler = GpuSampler(args.gpu_index, args.gpu_sample_interval_seconds)
    failures: list[str] = []
    summary: dict[str, Any] | None = None
    cleanup: dict[str, Any] = {}
    llm_samples: list[dict[str, Any]] = []
    voice_samples: list[dict[str, Any]] = []
    steady_after_pair_mib: list[int] = []
    run_started = time.monotonic()
    run_started_at = datetime.now(timezone.utc).isoformat()
    load_started: float | None = None
    run_status = "starting"
    interrupted = False
    baseline_used, total_mib = gpu_memory_mib(args.gpu_index)
    sampler.start()
    try:
        llm_command = [
            str(args.llama_server),
            "-m",
            str(args.llm_model),
            "--host",
            "127.0.0.1",
            "--port",
            str(llm_port),
            "-ngl",
            str(args.gpu_layers),
            "-c",
            "4096",
            "-np",
            "1",
            "-fa",
            "on",
        ]
        if args.llm_priority is not None:
            llm_command.extend(["--prio", str(args.llm_priority)])
        if args.llm_poll is not None:
            llm_command.extend(["--poll", str(args.llm_poll)])
        llm_process = subprocess.Popen(
            llm_command,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            creationflags=CREATE_NO_WINDOW,
        )
        wait_ready(f"{llm_endpoint}/health", 90)
        llm_loaded_mib, _ = gpu_memory_mib(args.gpu_index)

        voice_env = os.environ.copy()
        voice_env.update(
            {
                "OCLIVE_COSYVOICE_MODEL_DIR": str(args.voice_model_dir),
                "OCLIVE_COSYVOICE_PORT": str(voice_port),
                "OCLIVE_COSYVOICE_PRECISION": "auto",
                "OCLIVE_COSYVOICE_FIRST_CHUNK_PRIORITY": (
                    "1" if args.voice_first_chunk_priority == "on" else "0"
                ),
                "OCLIVE_COSYVOICE_FIRST_HOP_TOKENS": str(
                    args.voice_first_hop_tokens
                ),
                "PYTHONPATH": str(voice_root),
                "PYTHONIOENCODING": "utf-8",
                "PYTHONUTF8": "1",
            }
        )
        voice_process = subprocess.Popen(
            [str(args.voice_python), "-m", "tts.cosyvoice_sidecar"],
            cwd=voice_root,
            env=voice_env,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            creationflags=CREATE_NO_WINDOW,
        )
        wait_ready(f"{voice_endpoint}/health", 30)
        warm_payload = {
            "model_dir": str(args.voice_model_dir),
            "prime": not args.expect_admission_denied,
            "ref_audio": str(args.ref_audio),
            "ref_text": args.voice_ref_text,
            "emo_text": args.voice_emo_text,
        }
        with open_json(
            f"{voice_endpoint}/warm",
            warm_payload,
            timeout=120,
        ) as response:
            warm = json.load(response)
        after_warm_mib, _ = gpu_memory_mib(args.gpu_index)
        run_status = "running"
        emit_progress_checkpoint(
            args.output,
            status=run_status,
            started_at=run_started_at,
            elapsed_seconds=time.monotonic() - run_started,
            args=args,
            llm_samples=llm_samples,
            voice_samples=voice_samples,
            sampler=sampler,
            llm_process=llm_process,
            voice_process=voice_process,
            failures=failures,
        )
        if args.expect_admission_denied:
            if (
                warm.get("reason") != "gpu_admission_denied"
                or warm.get("retryable") is not True
            ):
                failures.append(
                    "expected retryable gpu_admission_denied, "
                    f"got ok={warm.get('ok')} reason={warm.get('reason')} "
                    f"retryable={warm.get('retryable')}"
                )
            gate = threading.Event()
            gate.set()
            llm_samples.append(
                run_llm(llm_endpoint, gate, min(args.llm_max_tokens, 32))
            )
        else:
            if not warm.get("ok"):
                failures.append(
                    f"voice warm failed: {warm.get('reason')}: {warm.get('message')}"
                )
            if (
                warm.get("precision_active") != "mixed_fp16"
                or warm.get("load_strategy") != "staged_cpu_mixed_fp16"
            ):
                failures.append(
                    "cold load did not use staged mixed precision: "
                    f"precision={warm.get('precision_active')} "
                    f"strategy={warm.get('load_strategy')}"
                )
            load_started = time.monotonic()
            next_checkpoint = load_started + args.checkpoint_interval_seconds
            deadline = (
                load_started + (args.duration_minutes * 60)
                if args.duration_minutes > 0
                else None
            )
            while True:
                if deadline is not None:
                    if llm_samples and time.monotonic() >= deadline:
                        break
                elif len(llm_samples) >= args.voice_runs:
                    break
                if llm_process.poll() is not None:
                    raise RuntimeError(
                        f"llama-server exited early with {llm_process.returncode}"
                    )
                if voice_process.poll() is not None:
                    raise RuntimeError(
                        f"CosyVoice sidecar exited early with {voice_process.returncode}"
                    )
                llm_result, voice_result = run_pair(
                    llm_endpoint,
                    voice_endpoint,
                    args.ref_audio,
                    args.llm_max_tokens,
                    args.voice_text,
                    args.voice_emo_text,
                    args.voice_ref_text,
                )
                llm_samples.append(llm_result)
                voice_samples.append(voice_result)
                steady_used_mib, _ = gpu_memory_mib(args.gpu_index)
                steady_after_pair_mib.append(steady_used_mib)
                if voice_result["chunks"] < 1 or voice_result["errors"]:
                    failures.append(f"voice stream failed: {voice_result}")
                if time.monotonic() >= next_checkpoint:
                    emit_progress_checkpoint(
                        args.output,
                        status=run_status,
                        started_at=run_started_at,
                        elapsed_seconds=time.monotonic() - run_started,
                        args=args,
                        llm_samples=llm_samples,
                        voice_samples=voice_samples,
                        sampler=sampler,
                        llm_process=llm_process,
                        voice_process=voice_process,
                        failures=failures,
                    )
                    next_checkpoint = (
                        time.monotonic() + args.checkpoint_interval_seconds
                    )
            load_elapsed_seconds = round(time.monotonic() - load_started, 3)

        llm_ttft = [
            int(sample["ttft_ms"])
            for sample in llm_samples
            if sample.get("ttft_ms") is not None
        ]
        voice_ttfc = [
            int(sample["ttfc_ms"])
            for sample in voice_samples
            if sample.get("ttfc_ms") is not None
        ]
        sidecar_ttfc = [
            int(sample["sidecar_ttfc_ms"])
            for sample in voice_samples
            if sample.get("sidecar_ttfc_ms") is not None
        ]
        client_delivery_overhead = [
            int(sample["client_delivery_overhead_ms"])
            for sample in voice_samples
            if sample.get("client_delivery_overhead_ms") is not None
        ]
        stage_names = sorted(
            {
                key
                for sample in voice_samples
                for key, value in (sample.get("timings_ms") or {}).items()
                if isinstance(value, int) and not isinstance(value, bool)
            }
        )
        voice_stage_timings = {
            stage: timing_distribution(
                [
                    int(sample["timings_ms"][stage])
                    for sample in voice_samples
                    if isinstance((sample.get("timings_ms") or {}).get(stage), int)
                    and not isinstance(
                        (sample.get("timings_ms") or {}).get(stage), bool
                    )
                ]
            )
            for stage in stage_names
        }
        prompt_cache_hits = sum(
            1 for sample in voice_samples if sample.get("prompt_cache_hit") is True
        )
        prompt_cache_misses = sum(
            1 for sample in voice_samples if sample.get("prompt_cache_hit") is False
        )
        peak_headroom = total_mib - sampler.peak_used_mib
        if peak_headroom < args.min_headroom_mib:
            failures.append(
                f"GPU headroom {peak_headroom} MiB < {args.min_headroom_mib} MiB"
            )
        if llm_ttft and max(llm_ttft) > args.max_llm_ttft_ms:
            failures.append(
                f"LLM TTFT max {max(llm_ttft)} ms > {args.max_llm_ttft_ms} ms"
            )
        if voice_ttfc and max(voice_ttfc) > args.max_voice_ttfc_ms:
            failures.append(
                f"voice TTFC max {max(voice_ttfc)} ms > "
                f"{args.max_voice_ttfc_ms} ms"
            )
        steady_growth_mib = (
            max(0, max(steady_after_pair_mib) - after_warm_mib)
            if steady_after_pair_mib
            else 0
        )
        if steady_growth_mib > args.max_steady_growth_mib:
            failures.append(
                f"steady GPU growth {steady_growth_mib} MiB > "
                f"{args.max_steady_growth_mib} MiB"
            )

        if sampler.sample_count < 1:
            failures.append("GPU sampler produced no samples")

        summary = {
            "schema_version": 3,
            "ok": not failures,
            "scenario": (
                "admission_denied"
                if args.expect_admission_denied
                else (
                    "real_time_hardware_soak"
                    if args.duration_minutes > 0
                    else "cold_load_and_concurrency"
                )
            ),
            "gpu_layers": args.gpu_layers,
            "llm_priority": args.llm_priority,
            "llm_poll": args.llm_poll,
            "voice_first_chunk_priority": args.voice_first_chunk_priority,
            "voice_first_hop_tokens": args.voice_first_hop_tokens,
            "requested_duration_minutes": args.duration_minutes,
            "actual_load_seconds": (
                load_elapsed_seconds if not args.expect_admission_denied else 0
            ),
            "pairs_completed": len(voice_samples),
            "voice_input": {
                "text_chars": len(args.voice_text),
                "emotion_chars": len(args.voice_emo_text),
                "reference_text_chars": len(args.voice_ref_text),
                "warm_prompt_aligned": True,
            },
            "gpu_mib": {
                "baseline": baseline_used,
                "llm_loaded": llm_loaded_mib,
                "after_warm": after_warm_mib,
                "peak": sampler.peak_used_mib,
                "total": total_mib,
                "peak_headroom": peak_headroom,
                "steady_after_pair": steady_after_pair_mib,
                "steady_growth": steady_growth_mib,
            },
            "warm": {
                key: warm.get(key)
                for key in (
                    "ok",
                    "reason",
                    "retryable",
                    "precision_requested",
                    "precision_active",
                    "precision_fallback_reason",
                    "load_strategy",
                    "load_vram_probe",
                    "load_free_vram_before_mib",
                    "load_min_free_vram_mib",
                    "load_peak_allocated_mib",
                    "load_peak_reserved_mib",
                    "first_chunk_priority_requested",
                    "first_chunk_priority_active",
                    "first_chunk_priority_detail",
                    "first_chunk_hop_tokens",
                    "first_chunk_min_text_chars",
                )
            },
            "llm_ttft_ms": {
                "samples": llm_ttft,
                "p50": percentile(llm_ttft, 0.5),
                "p95": percentile(llm_ttft, 0.95),
                "max": max(llm_ttft) if llm_ttft else None,
            },
            "voice_ttfc_ms": {
                "samples": voice_ttfc,
                "p50": percentile(voice_ttfc, 0.5),
                "p95": percentile(voice_ttfc, 0.95),
                "max": max(voice_ttfc) if voice_ttfc else None,
            },
            "voice_sidecar_ttfc_ms": timing_distribution(sidecar_ttfc),
            "voice_client_delivery_overhead_ms": timing_distribution(
                client_delivery_overhead
            ),
            "voice_stage_timings_ms": voice_stage_timings,
            "voice_prompt_cache": {
                "hits": prompt_cache_hits,
                "misses": prompt_cache_misses,
                "unknown": len(voice_samples)
                - prompt_cache_hits
                - prompt_cache_misses,
            },
            "voice_request_samples": [
                {
                    "client_ttfc_ms": sample.get("ttfc_ms"),
                    "sidecar_ttfc_ms": sample.get("sidecar_ttfc_ms"),
                    "client_delivery_overhead_ms": sample.get(
                        "client_delivery_overhead_ms"
                    ),
                    "prompt_cache_hit": sample.get("prompt_cache_hit"),
                    "first_chunk_priority_applied": sample.get(
                        "first_chunk_priority_applied"
                    ),
                    "first_chunk_hop_tokens": sample.get(
                        "first_chunk_hop_tokens"
                    ),
                    "timings_ms": sample.get("timings_ms") or {},
                }
                for sample in voice_samples
            ],
            "voice_chunks": [sample["chunks"] for sample in voice_samples],
            "failures": failures,
        }
        run_status = "completed"
    except KeyboardInterrupt:
        interrupted = True
        run_status = "interrupted"
        failures.append("run interrupted by user")
    except Exception as exc:  # noqa: BLE001
        run_status = "failed"
        failures.append(f"runtime exception: {type(exc).__name__}: {exc}")
    finally:
        sampler.stop()
        cleanup = {
            "voice": terminate(voice_process),
            "llm": terminate(llm_process),
        }

    if not sampler.thread_joined:
        failures.append("GPU sampler thread did not stop")
    if sampler.sample_failures > args.max_gpu_sample_failures:
        failures.append(
            f"GPU sampler failures {sampler.sample_failures} > "
            f"{args.max_gpu_sample_failures}"
        )
    for label, state in cleanup.items():
        if state["started"] and not state["reaped"]:
            failures.append(f"{label} process was not reaped")
    if summary is None:
        partial_load_seconds = (
            round(max(0.0, time.monotonic() - load_started), 3)
            if load_started is not None
            else 0
        )
        summary = {
            "schema_version": 3,
            "scenario": (
                "interrupted" if interrupted else "startup_or_runtime_failure"
            ),
            "gpu_layers": args.gpu_layers,
            "requested_duration_minutes": args.duration_minutes,
            "actual_load_seconds": partial_load_seconds,
            "pairs_completed": len(voice_samples),
        }
    summary.update(
        {
            "status": run_status,
            "started_at": run_started_at,
            "completed_at": datetime.now(timezone.utc).isoformat(),
            "ok": not failures,
            "gpu_sampler": {
                "interval_seconds": args.gpu_sample_interval_seconds,
                "samples": sampler.sample_count,
                "failures": sampler.sample_failures,
                "max_failures": args.max_gpu_sample_failures,
                "thread_joined": sampler.thread_joined,
            },
            "cleanup": cleanup,
            "failures": failures,
        }
    )
    emit_summary(summary, args.output)
    emit_progress_checkpoint(
        args.output,
        status=run_status,
        started_at=run_started_at,
        elapsed_seconds=time.monotonic() - run_started,
        args=args,
        llm_samples=llm_samples,
        voice_samples=voice_samples,
        sampler=sampler,
        llm_process=llm_process,
        voice_process=voice_process,
        failures=failures,
        cleanup=cleanup,
    )
    if interrupted:
        return 130
    return 0 if not failures else 1


if __name__ == "__main__":
    raise SystemExit(main())
