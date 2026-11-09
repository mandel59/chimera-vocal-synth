#!/usr/bin/env bash
# One-time setup for tools/asr/speech_wer.py and japanese_cer.py: a Python env with faster-whisper
# (and pykakasi, to read Japanese transcripts back as kana) in ~/.cache/chimera-asr. The models
# download there on first use: small.en for English, the multilingual small for Japanese (~0.5 GB each).
set -euo pipefail
dir="$HOME/.cache/chimera-asr"
mkdir -p "$dir/models"
[ -x "$dir/.venv/bin/python" ] || uv venv -q --python 3.12 "$dir/.venv"
uv pip install -q --python "$dir/.venv/bin/python" faster-whisper==1.2.1 pykakasi==2.3.0
echo "run: $dir/.venv/bin/python tools/asr/speech_wer.py <outdir> [--against earlier/wer.json]"
echo " or: $dir/.venv/bin/python tools/asr/japanese_cer.py <outdir> [--against earlier/cer.json]"
