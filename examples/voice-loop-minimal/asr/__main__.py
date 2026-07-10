"""CLI: python -m asr.transcribe [--wav path] or stdin JSON."""

from asr.transcribe import main

if __name__ == "__main__":
    raise SystemExit(main())
