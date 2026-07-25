#!/usr/bin/env python3
"""Stress the shared-GPU llama-server + CosyVoice2 runtime.

Normal mode starts llama-server first, cold-loads CosyVoice, then runs repeated
LLM/TTS pairs. Admission mode proves that an unsafe cold load is rejected
without disturbing the already-running LLM.
"""

from __future__ import annotations

import argparse
import json
import os
import socket
import subprocess
import threading
import time
import urllib.request
from pathlib import Path
from typing import Any


CREATE_NO_WINDOW = getattr(subprocess, "CREATE_NO_WINDOW", 0)


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
    parser.add_argument("--gpu-layers", type=int, default=24)
    parser.add_argument("--voice-runs", type=int, default=5)
    parser.add_argument("--llm-max-tokens", type=int, default=64)
    parser.add_argument("--min-headroom-mib", type=int, default=768)
    parser.add_argument("--max-voice-ttfc-ms", type=int, default=8000)
    parser.add_argument("--max-llm-ttft-ms", type=int, default=5000)
    parser.add_argument("--max-steady-growth-mib", type=int, default=256)
    parser.add_argument("--expect-admission-denied", action="store_true")
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
    def __init__(self, gpu_index: str) -> None:
        self.gpu_index = gpu_index
        self.peak_used_mib = 0
        self.total_mib = 0
        self._stop = threading.Event()
        self._thread = threading.Thread(target=self._sample, daemon=True)

    def _sample(self) -> None:
        while not self._stop.is_set():
            try:
                used, total = gpu_memory_mib(self.gpu_index)
                self.peak_used_mib = max(self.peak_used_mib, used)
                self.total_mib = total
            except (OSError, subprocess.SubprocessError, ValueError, IndexError):
                pass
            self._stop.wait(0.05)

    def start(self) -> None:
        used, total = gpu_memory_mib(self.gpu_index)
        self.peak_used_mib = used
        self.total_mib = total
        self._thread.start()

    def stop(self) -> None:
        self._stop.set()
        self._thread.join(timeout=5)


def terminate(process: subprocess.Popen[Any] | None) -> None:
    if process is None:
        return
    if process.poll() is None:
        process.terminate()
    try:
        process.wait(timeout=10)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait(timeout=5)


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
) -> dict[str, Any]:
    payload = {
        "text": "\u7238\u7238\uff0c\u4eca\u5929\u8f9b\u82e6\u5566\u3002"
        "\u5148\u559d\u53e3\u6c34\uff0c\u6211\u4eec\u6162\u6162\u6765\u3002",
        "emo_text": "\u7528\u6e29\u6696\u3001\u81ea\u7136\u3001"
        "\u5173\u5fc3\u7684\u8bed\u6c14",
        "ref_audio": str(ref_audio),
        "ref_text": "\u65e9\u4e0a\u597d\u5440\uff0c\u6211\u662f\u6c90\u6c90\u3002",
        "speed": 1.0,
    }
    gate.wait()
    started = time.perf_counter()
    first = None
    chunks = 0
    done: dict[str, Any] = {}
    errors: list[dict[str, Any]] = []
    with open_json(
        f"{endpoint}/synthesize/stream",
        payload,
        timeout=120,
    ) as response:
        for raw in response:
            event = json.loads(raw.decode("utf-8"))
            if event.get("event") == "chunk":
                first = first or time.perf_counter()
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
    return {
        "ttfc_ms": round((first - started) * 1000) if first else None,
        "total_ms": round((ended - started) * 1000),
        "chunks": chunks,
        "sidecar_ttfc_ms": done.get("ttfc_ms"),
        "stream_mode": done.get("stream_mode"),
        "errors": errors,
    }


def run_pair(
    llm_endpoint: str,
    voice_endpoint: str,
    ref_audio: Path,
    max_tokens: int,
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
            args=("voice", run_voice, voice_endpoint, gate, ref_audio),
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
    sampler = GpuSampler(args.gpu_index)
    failures: list[str] = []
    baseline_used, total_mib = gpu_memory_mib(args.gpu_index)
    sampler.start()
    try:
        llm_process = subprocess.Popen(
            [
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
            ],
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
            "ref_text": "\u65e9\u4e0a\u597d\u5440\uff0c\u6211\u662f\u6c90\u6c90\u3002",
            "emo_text": "\u7528\u6e29\u6696\u81ea\u7136\u7684\u8bed\u6c14",
        }
        with open_json(
            f"{voice_endpoint}/warm",
            warm_payload,
            timeout=120,
        ) as response:
            warm = json.load(response)
        after_warm_mib, _ = gpu_memory_mib(args.gpu_index)

        llm_samples: list[dict[str, Any]] = []
        voice_samples: list[dict[str, Any]] = []
        steady_after_pair_mib: list[int] = []
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
            for _ in range(args.voice_runs):
                llm_result, voice_result = run_pair(
                    llm_endpoint,
                    voice_endpoint,
                    args.ref_audio,
                    args.llm_max_tokens,
                )
                llm_samples.append(llm_result)
                voice_samples.append(voice_result)
                steady_used_mib, _ = gpu_memory_mib(args.gpu_index)
                steady_after_pair_mib.append(steady_used_mib)
                if voice_result["chunks"] < 1 or voice_result["errors"]:
                    failures.append(f"voice stream failed: {voice_result}")

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

        summary = {
            "ok": not failures,
            "scenario": (
                "admission_denied"
                if args.expect_admission_denied
                else "cold_load_and_concurrency"
            ),
            "gpu_layers": args.gpu_layers,
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
            "voice_chunks": [sample["chunks"] for sample in voice_samples],
            "failures": failures,
        }
        print(json.dumps(summary, ensure_ascii=False, indent=2))
        return 0 if not failures else 1
    finally:
        sampler.stop()
        terminate(voice_process)
        terminate(llm_process)


if __name__ == "__main__":
    raise SystemExit(main())
