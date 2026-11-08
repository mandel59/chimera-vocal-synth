"""Japanese intelligibility of sung kana lyrics, scored by Whisper: sing each sentence one mora per
eighth note on a near-monotone line with `chimera render notes` in the male and female voices,
transcribe it in Japanese with faster-whisper (multilingual small, greedy, so the same audio gives
the same text), read the transcript back into hiragana (pykakasi) and count character errors.

usage: japanese_cer.py <outdir> [--against earlier/cer.json]

Writes <outdir>/cer.json and the renders. With --against it compares the character error rate
(both bodies together): BETTER if it fell by at least MARGIN points, WORSE if it rose by more than
SLACK, and NO CLEAR CHANGE between.

A sentence's errors count at most its length: greedy Whisper sometimes loops on one character
("ぬぅぅぅ…") and would otherwise swamp every other sentence.

Most sentences are dense in し ち じ (the alveolo-palatal ɕ tɕ dʑ); a few without them are a
control. Particles are written as sung (わ, not は), and the lyrics avoid は and へ as particles
because the transcript is read back by spelling.

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
# fall in character error rate (points) that counts as an improvement: about 5 of the 368 kana
MARGIN = 1.5

SENTENCES = [
    "しずかなあさにしろいねこがねむる",
    "ちいさなちょうちょがのにとまった",
    "じぶんのじかんをたいせつにしよう",
    "しんぶんをよみながらおちゃをのむ",
    "きしゃのまどからやまがみえる",
    "あしたもいっしょにうたをうたおう",
    "ちかてつのえきでともだちをまつ",
    "じゅうじにしゅっぱつします",
    "さくらのはながさいている",
    "そらにほしがひかっている",
    "むかしむかしあるところにおじいさんがいました",
    "にじのむこうにちいさなしまがある",
]

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
    out = args[0]
    os.makedirs(out, exist_ok=True)
    chimera = os.path.join(REPO, "target/release/chimera")
    jobs = [(text, body, os.path.join(out, f"{body}-{i + 1:02d}")) for i, text in enumerate(SENTENCES) for body in BODIES]

    def render(job):
        text, body, stem = job
        with open(stem + ".json", "w") as f:
            json.dump(melody(text), f, ensure_ascii=False)
        subprocess.run([chimera, "render", "notes", stem + ".json", stem, body], check=True, capture_output=True)

    with ThreadPoolExecutor(os.cpu_count()) as pool:
        list(pool.map(render, jobs))

    model = WhisperModel("small", device="cpu", compute_type="int8", cpu_threads=os.cpu_count(), download_root=CACHE)
    rows, totals = [], {}
    for text, body, stem in jobs:
        segs, _ = model.transcribe(load16k(stem + ".wav"), language="ja", beam_size=1, temperature=0.0,
                                   condition_on_previous_text=False, vad_filter=False)
        heard = "".join(s.text.strip() for s in segs)
        e = min(edits(text, kana(heard)), len(text))
        rows.append({"body": body, "text": text, "heard": heard, "errors": e, "chars": len(text)})
        for key in (body, "all"):
            t = totals.setdefault(key, [0, 0])
            t[0] += e
            t[1] += len(text)
    cer = {k: round(100.0 * e / n, 2) for k, (e, n) in totals.items()}
    with open(os.path.join(out, "cer.json"), "w") as f:
        json.dump({"cer": cer, "sentences": rows}, f, ensure_ascii=False, indent=1)
    for r in rows:
        if r["errors"]:
            print(f"  {r['body']:6s} {r['errors']:2d}/{r['chars']}  {r['text']}  →  {r['heard'][:40]}")
    print("character error rate (%): " + ", ".join(f"{k} {v}" for k, v in sorted(cer.items())))
    if against:
        old, new = json.load(open(against))["cer"]["all"], cer["all"]
        verdict = "WORSE" if new > old + SLACK else "BETTER" if new <= old - MARGIN else "NO CLEAR CHANGE"
        print(f"japanese verdict against {against}: {verdict} (character error rate {old} → {new} %, "
              f"better from -{MARGIN}, worse past +{SLACK})")


main()
