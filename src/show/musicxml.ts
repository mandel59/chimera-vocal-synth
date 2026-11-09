/**
 * MusicXML → a melody to sing: one voice's notes and rests (in beats at the song's first tempo),
 * with their lyrics, then the lyrics matched to the notes as syllables.
 *
 * Read: score-partwise (`.musicxml`, `.xml`, or compressed `.mxl`). The part sung is the first
 * with lyrics, and in it the voice of its first lyric. Chords sing their first note; grace and
 * cue notes are left out; tied notes become one; a tempo change rescales the beats that follow
 * (so the song keeps one beat). Lyric verse 1 (or the first lyric) is sung.
 *
 * Matched (`singScore`): Japanese is read a phrase at a time (between rests), so ん, っ and が行
 * follow the kana around them, and each note takes as many syllables as it has morae (a note of
 * two morae is split in two, a note of none holds the vowel before). Other lyrics are read a word
 * at a time (MusicXML's begin/middle/end), and the word's syllables go to its notes in order: a
 * syllable short, the last holds on; a syllable over, the last note is split. A note without a
 * lyric holds the vowel before it (melisma). A score with no lyrics at all sings the given
 * syllables, one per note.
 */
import { child, childText, children, parseXml, type XmlElement } from "./xml.ts";
import { readZip } from "./zip.ts";
import type { Language, Syllable } from "../wasm/types.ts";

export type Syllabic = "single" | "begin" | "middle" | "end";

/** A note (midi) or a rest (midi null), its length in beats, and its lyric if it has one. */
export interface ScoreNote {
  midi: number | null;
  beats: number;
  lyric?: { text: string; syllabic: Syllabic };
}

export interface Score {
  title: string;
  /** the song's beat (quarter notes per minute): its first tempo, or 120 */
  bpm: number;
  notes: ScoreNote[];
}

const STEP: Record<string, number> = { C: 0, D: 2, E: 4, F: 5, G: 7, A: 9, B: 11 };

function hasLyric(el: XmlElement): boolean {
  return el.name === "lyric" || el.children.some(hasLyric);
}

function tempoOf(el: XmlElement): number | null {
  const t = parseFloat(el.attrs.tempo ?? "");
  return Number.isFinite(t) && t > 0 ? t : null;
}

/** A measure's tempo marks, in order: <sound tempo> alone or in a <direction>. */
function soundTempo(el: XmlElement): number | null {
  if (el.name === "sound") return tempoOf(el);
  if (el.name === "direction") {
    for (const s of children(el, "sound")) {
      const t = tempoOf(s);
      if (t) return t;
    }
    // a metronome mark without <sound>: per-minute of a quarter
    const m = child(child(child(el, "direction-type"), "metronome"), "per-minute");
    const unit = childText(child(child(el, "direction-type"), "metronome"), "beat-unit");
    const pm = parseFloat(m?.text ?? "");
    if (Number.isFinite(pm) && pm > 0) return unit === "half" ? pm * 2 : unit === "eighth" ? pm / 2 : pm;
  }
  return null;
}

function lyricOf(note: XmlElement): ScoreNote["lyric"] {
  const lyrics = children(note, "lyric");
  const l = lyrics.find((x) => (x.attrs.number ?? "1") === "1") ?? lyrics[0];
  if (!l) return undefined;
  // an elision puts two syllables on one note: their texts, with a space between
  const text = children(l, "text")
    .map((t) => t.text)
    .join(" ")
    .trim();
  if (!text) return undefined; // an <extend/> alone continues the syllable before
  const s = childText(l, "syllabic");
  return { text, syllabic: s === "begin" || s === "middle" || s === "end" ? s : "single" };
}

/** Parse a MusicXML document (score-partwise) into the melody of one voice. */
export function parseMusicXml(xml: string, fallbackTitle = "Score"): Score {
  const root = parseXml(xml);
  if (root.name === "score-timewise") throw new Error("score-timewise MusicXML isn't supported; export it as partwise");
  if (root.name !== "score-partwise") throw new Error(`not a MusicXML score (<${root.name}>)`);
  const title = childText(child(root, "work"), "work-title") || childText(root, "movement-title") || fallbackTitle;
  const parts = children(root, "part");
  const part = parts.find(hasLyric) ?? parts[0];
  if (!part) throw new Error("the score has no parts");
  const measures = children(part, "measure");
  // the voice sung: the first lyric's (or the first sounding note's)
  const notesIn = measures.flatMap((m) => children(m, "note"));
  const lead = notesIn.find((n) => child(n, "lyric")) ?? notesIn.find((n) => !child(n, "rest"));
  const voice = childText(lead, "voice") ?? "1";

  let divisions = 1;
  let tempo: number | null = null;
  let first: number | null = null;
  const raw: { note: ScoreNote; tempo: number | null; tieStop: boolean }[] = [];
  for (const m of measures) {
    for (const el of m.children) {
      if (el.name === "attributes") {
        const d = parseFloat(childText(el, "divisions") ?? "");
        if (Number.isFinite(d) && d > 0) divisions = d;
        continue;
      }
      if (el.name === "sound" || el.name === "direction") {
        const t = soundTempo(el);
        if (t) {
          tempo = t;
          first ??= t;
        }
        continue;
      }
      if (el.name !== "note") continue;
      if (child(el, "grace") || child(el, "cue") || child(el, "chord")) continue;
      if ((childText(el, "voice") ?? "1") !== voice) continue;
      const dur = parseFloat(childText(el, "duration") ?? "");
      if (!Number.isFinite(dur) || dur <= 0) continue;
      const pitch = child(el, "pitch");
      let midi: number | null = null;
      if (pitch && !child(el, "rest")) {
        const step = STEP[childText(pitch, "step") ?? ""];
        const octave = parseInt(childText(pitch, "octave") ?? "", 10);
        const alter = parseFloat(childText(pitch, "alter") ?? "0") || 0;
        if (step !== undefined && Number.isFinite(octave)) midi = (octave + 1) * 12 + step + alter;
      }
      const tieStop = children(el, "tie").some((t) => t.attrs.type === "stop") || children(child(el, "notations"), "tied").some((t) => t.attrs.type === "stop");
      raw.push({ note: { midi, beats: dur / divisions, lyric: midi === null ? undefined : lyricOf(el) }, tempo, tieStop });
    }
  }
  const bpm = first ?? 120;
  const notes: ScoreNote[] = [];
  for (const r of raw) {
    // beats at the song's first tempo: a faster passage takes fewer of them
    const beats = r.note.beats * (r.tempo ? bpm / r.tempo : 1);
    const prev = notes[notes.length - 1];
    if (prev && prev.midi !== null && r.note.midi === prev.midi && r.tieStop && !r.note.lyric) {
      prev.beats += beats;
      continue;
    }
    if (prev && prev.midi === null && r.note.midi === null) {
      prev.beats += beats;
      continue;
    }
    notes.push({ ...r.note, beats });
  }
  // rests at the start and the end are silence the player adds anyway
  while (notes.length && notes[0].midi === null) notes.shift();
  while (notes.length && notes[notes.length - 1].midi === null) notes.pop();
  if (!notes.some((n) => n.midi !== null)) throw new Error("the score has no notes to sing");
  return { title, bpm, notes };
}

/** Read a score file: MusicXML text, or a compressed `.mxl` (its container names the score). */
export async function readScoreFile(bytes: Uint8Array, fileName: string): Promise<Score> {
  const base = fileName.replace(/\.[^.]*$/, "");
  const zipped = bytes[0] === 0x50 && bytes[1] === 0x4b;
  if (!zipped) return parseMusicXml(new TextDecoder().decode(bytes), base);
  const entries = readZip(bytes);
  let path: string | undefined;
  const container = entries.find((e) => e.name === "META-INF/container.xml");
  if (container) {
    const c = parseXml(new TextDecoder().decode(await container.read()));
    path = child(child(c, "rootfiles"), "rootfile")?.attrs["full-path"];
  }
  const entry = entries.find((e) => e.name === path) ?? entries.find((e) => !e.name.startsWith("META-INF/") && /\.(xml|musicxml)$/i.test(e.name));
  if (!entry) throw new Error("the .mxl archive holds no score");
  return parseMusicXml(new TextDecoder().decode(await entry.read()), base);
}

// ---- lyrics to syllables

/** A sung note (midi) or a rest (midi null) with the syllable it sings. */
export interface SungScoreNote {
  midi: number | null;
  beats: number;
  syl?: Syllable;
}

const isKana = (c: string) => /[ぁ-ゖァ-ヺー]/.test(c);
const SMALL = "ぁぃぅぇぉゃゅょゎっァィゥェォャュョヮッ";

/** Morae a note's kana take: each kana but the small ones and っ; ー is one. */
export function morae(text: string): number {
  return [...text].filter((c) => isKana(c) && !SMALL.includes(c)).length;
}

/** The syllable that holds `prev`'s vowel onto another note. */
export function hold(prev: Syllable): Syllable {
  return { text: "~", onset: [], nucleus: prev.nucleus, coda: [], melisma: true, wordEnd: prev.wordEnd, phraseEnd: false };
}

const AH: Syllable = { text: "ah", onset: [], nucleus: { ipa: "ɑ", stress: 1 }, coda: [], wordEnd: true, phraseEnd: false };

/**
 * The score's notes with a syllable each (rests keep none). `syllabify` reads lyrics as the Sing
 * tab does (`lyricsToSyllables`); `fallback` is sung, a syllable a note, when the score has no
 * lyrics.
 */
export function singScore(score: Score, syllabify: (text: string) => Syllable[], fallback: Syllable[] = [], language?: Language): SungScoreNote[] {
  const out: SungScoreNote[] = [];
  let prev: Syllable | null = null;
  const push = (midi: number, beats: number, syl: Syllable) => {
    out.push({ midi, beats, syl });
    if (!syl.melisma) prev = syl;
  };
  const holdPrev = (midi: number, beats: number) => push(midi, beats, hold(prev ?? AH));
  /** a note singing several syllables: split evenly */
  const pushSplit = (midi: number, beats: number, syls: Syllable[]) => syls.forEach((s) => push(midi, beats / syls.length, s));

  const anyLyric = score.notes.some((n) => n.lyric);
  // runs of notes between rests, and each rest a run of its own
  const runs: ScoreNote[][] = [];
  let run: ScoreNote[] = [];
  for (const n of score.notes) {
    if (n.midi !== null) {
      run.push(n);
      continue;
    }
    if (run.length) runs.push(run);
    run = [];
    runs.push([n]);
  }
  if (run.length) runs.push(run);

  let k = 0; // fallback cursor
  for (const r of runs) {
    if (r[0].midi === null) {
      out.push({ midi: null, beats: r[0].beats });
      continue;
    }
    const start = out.length;
    if (!anyLyric) {
      for (const n of r) {
        const s = fallback.length ? fallback[k++ % fallback.length] : AH;
        if (s.melisma && prev) holdPrev(n.midi!, n.beats);
        else push(n.midi!, n.beats, { ...s, phraseEnd: false });
      }
    } else if (language ? language === "ja" : r.some((n) => n.lyric && [...n.lyric.text].some(isKana))) {
      singKana(r);
    } else {
      singWords(r);
    }
    // a breath at each rest: the run's last syllable ends a phrase
    const last = out[out.length - 1];
    if (out.length > start && last.syl) last.syl = { ...last.syl, phraseEnd: true };
  }
  return out;

  function singKana(r: ScoreNote[]) {
    const syls = syllabify(r.map((n) => n.lyric?.text ?? "").join(""));
    const counts = r.map((n) => (n.lyric ? morae(n.lyric.text) : 0));
    const exact = counts.reduce((a, b) => a + b, 0) === syls.length;
    let at = 0;
    r.forEach((n, i) => {
      // when the morae don't add up (kanji, letters), a lyric note sings the next syllable
      const take = exact ? counts[i] : n.lyric ? 1 : 0;
      const mine = syls.slice(at, at + take);
      at += take;
      if (!mine.length) holdPrev(n.midi!, n.beats);
      else if (mine.length === 1 && mine[0].melisma) holdPrev(n.midi!, n.beats);
      else pushSplit(n.midi!, n.beats, mine.map((s) => (s.melisma ? hold(prev ?? AH) : s)));
    });
  }

  function singWords(r: ScoreNote[]) {
    let i = 0;
    while (i < r.length) {
      const n = r[i];
      if (!n.lyric) {
        holdPrev(n.midi!, n.beats);
        i++;
        continue;
      }
      // the word: this note to its "end" (or a "single"), with the held notes between
      const notes: ScoreNote[] = [n];
      let text = n.lyric.text;
      let j = i + 1;
      if (n.lyric.syllabic === "begin" || n.lyric.syllabic === "middle") {
        while (j < r.length) {
          const m = r[j];
          notes.push(m);
          j++;
          if (m.lyric) {
            text += m.lyric.text;
            if (m.lyric.syllabic === "end" || m.lyric.syllabic === "single") break;
          }
        }
      }
      const syls = syllabify(text.replace(/[-‐]/g, "")).filter((s) => !s.melisma).map((s) => ({ ...s, phraseEnd: false }));
      const slots = notes.filter((m) => m.lyric).length;
      let s = 0;
      let slot = 0;
      for (const m of notes) {
        if (!m.lyric || s >= syls.length) {
          holdPrev(m.midi!, m.beats);
          continue;
        }
        slot++;
        // the word's last lyric note takes every syllable left
        const take = slot === slots ? syls.length - s : 1;
        pushSplit(m.midi!, m.beats, syls.slice(s, s + take));
        s += take;
      }
      i = j;
    }
  }
}

/** Semitones to move a melody by octaves so its middle sits in a body's singing range. */
export function octaveShift(notes: { midi: number | null }[], female: number): number {
  const sung = notes
    .map((n) => n.midi)
    .filter((m): m is number => m !== null)
    .sort((a, b) => a - b);
  if (!sung.length) return 0;
  const middle = sung[Math.floor(sung.length / 2)];
  // the demo songs' middles (さくらさくら A4, 荒城の月 D5, sung an octave down by a man): A4 for a
  // woman, A3 for a man, each within a fifth or so
  const target = 57 + 12 * female;
  return 12 * Math.round((target - middle) / 12) || 0;
}
