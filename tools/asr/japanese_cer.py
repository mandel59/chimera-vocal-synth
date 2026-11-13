"""Japanese intelligibility of sung kana lyrics, scored by Whisper: sing each sentence one mora per
eighth note on a near-monotone line with `chimera render notes` in the male and female voices,
transcribe it in Japanese with faster-whisper (multilingual small, greedy, so the same audio gives
the same text), read the transcript back into hiragana (pykakasi) and count character errors.

usage: japanese_cer.py <outdir> [--against earlier/cer.json] [--chimera path/to/chimera] [--speech]

Writes <outdir>/cer.json and the renders. With --against it compares the training sentences'
character error rate (both bodies together): BETTER if it fell by at least MARGIN points, WORSE if
it rose by more than SLACK, and NO CLEAR CHANGE between. The held-out sentences are reported but
never decide, and their transcripts aren't printed: a tuning loop must not listen to them.
--chimera renders with another build (default target/release/chimera), to measure a baseline.
--speech reads the sentences with `chimera render text` instead of singing them.

A sentence's errors count at most its length: greedy Whisper sometimes loops on one character
("ぬぅぅぅ…") and would otherwise swamp every other sentence.

The sentences are in japanese_sentences.tsv (set, focus, text): "train" (24) and "held" (12),
each spread over a focus the sentence is dense in: sh (し しゃ…), ch, j, r (ら行), devoice (i/u
between voiceless consonants) and control. They are hiragana, 12-20 kana, with particles written
as sung (わ, え) and no は or へ at all, because the transcript is read back by spelling.

Set up once with tools/asr/setup.sh. Build the CLI first: cargo build --release -p chimera-cli.
"""

import json
import os
import subprocess
import sys
import unicodedata
from concurrent.futures import ThreadPoolExecutor

import wave

import numpy as np
import pykakasi
from faster_whisper import WhisperModel

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
CACHE = os.path.expanduser("~/.cache/chimera-asr/models")
BODIES = ("male", "female")
# character error rate (points) a change may add before it counts as a regression
SLACK = 3.0
# fall in character error rate (points) that counts as an improvement: about 12 of the ~800
# training kana (both bodies)
MARGIN = 1.5

KKS = pykakasi.kakasi()


# edits and load16k as in speech_wer.py (which runs on import, and is a judge left untouched)
def edits(ref, hyp):
    d = list(range(len(hyp) + 1))
    for i, r in enumerate(ref, 1):
        prev, d[0] = d[0], i
        for j, h in enumerate(hyp, 1):
            prev, d[j] = d[j], min(d[j] + 1, d[j - 1] + 1, prev + (r != h))
    return d[len(hyp)]


def load16k(path):
    """Mono float32 at 16 kHz from a 16-bit PCM WAV (band-limited FFT resampling)."""
    with wave.open(path) as w:
        ch, rate = w.getnchannels(), w.getframerate()
        x = np.frombuffer(w.readframes(w.getnframes()), dtype="<i2").astype(np.float64) / 32768.0
    x = x.reshape(-1, ch).mean(axis=1)
    n = int(round(len(x) * 16000 / rate))
    return (np.fft.irfft(np.fft.rfft(x)[: n // 2 + 1], n) * (n / len(x))).astype(np.float32)


def kana(text):
    """Hiragana only: the reading of kanji and katakana, punctuation and spaces dropped."""
    s = unicodedata.normalize("NFKC", "".join(x["hira"] for x in KKS.convert(text)))
    return "".join(c for c in s if "ぁ" <= c <= "ゟ" or c == "ー")


def melody(text):
    """One note per mora (a small kana joins the one before; っ takes no note), the last held."""
    n = sum(1 for c in text if c not in "ゃゅょぁぃぅぇぉっ")
    line = [67, 67, 69, 67, 65, 67]
    return {"bpm": 100, "lyrics": text + "。", "notes": [[line[i % len(line)], 0.5] for i in range(n - 1)] + [[64, 1.5]]}


def main():
    args = sys.argv[1:]
    against = None
    if "--against" in args:
        i = args.index("--against")
        against = args[i + 1]
        del args[i : i + 2]
    chimera = os.path.join(REPO, "target/release/chimera")
    if "--chimera" in args:
        i = args.index("--chimera")
        chimera = os.path.abspath(args[i + 1])
        del args[i : i + 2]
    speech = "--speech" in args
    if speech:
        args.remove("--speech")
    out = args[0]
    os.makedirs(out, exist_ok=True)
    sentences = [line.rstrip("\n").split("\t") for line in open(os.path.join(HERE, "japanese_sentences.tsv")) if line.strip()]
    jobs = [(part, focus, text, body, os.path.join(out, f"{part}-{focus}-{body}-{i + 1:02d}"))
            for i, (part, focus, text) in enumerate(sentences) for body in BODIES]

    def render(job):
        _, _, text, body, stem = job
        if speech:
            subprocess.run([chimera, "render", "text", stem, text + "。", body], check=True, capture_output=True)
            return
        with open(stem + ".json", "w") as f:
            json.dump(melody(text), f, ensure_ascii=False)
        subprocess.run([chimera, "render", "notes", stem + ".json", stem, body], check=True, capture_output=True)

    with ThreadPoolExecutor(os.cpu_count()) as pool:
        list(pool.map(render, jobs))

    model = WhisperModel("small", device="cpu", compute_type="int8", cpu_threads=os.cpu_count(), download_root=CACHE)
    rows, totals = [], {}
    for part, focus, text, body, stem in jobs:
        segs, _ = model.transcribe(load16k(stem + ".wav"), language="ja", beam_size=1, temperature=0.0,
                                   condition_on_previous_text=False, vad_filter=False)
        heard = "".join(s.text.strip() for s in segs)
        e = min(edits(text, kana(heard)), len(text))
        rows.append({"set": part, "focus": focus, "body": body, "text": text, "heard": heard, "errors": e, "chars": len(text)})
        # "all" is the training sentences; "held" the others
        keys = (body, focus, "all") if part == "train" else ("held",)
        for key in keys:
            t = totals.setdefault(key, [0, 0])
            t[0] += e
            t[1] += len(text)
    cer = {k: round(100.0 * e / n, 2) for k, (e, n) in totals.items()}
    with open(os.path.join(out, "cer.json"), "w") as f:
        json.dump({"cer": cer, "sentences": rows}, f, ensure_ascii=False, indent=1)
    for r in rows:
        if r["errors"] and r["set"] == "train":
            print(f"  {r['focus']:8s} {r['body']:6s} {r['errors']:2d}/{r['chars']}  {r['text']}  →  {r['heard'][:40]}")
    print("character error rate (%): " + ", ".join(f"{k} {v}" for k, v in sorted(cer.items())))
    if against:
        before = json.load(open(against))["cer"]
        old, new = before["all"], cer["all"]
        verdict = "WORSE" if new > old + SLACK else "BETTER" if new <= old - MARGIN else "NO CLEAR CHANGE"
        held = f"; held-out {before['held']} → {cer['held']} %, not used to decide" if "held" in before else ""
        print(f"japanese verdict against {against}: {verdict} (training character error rate {old} → {new} %, "
              f"better from -{MARGIN}, worse past +{SLACK}{held})")


main()
