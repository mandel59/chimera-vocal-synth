/**
 * MusicXML for the Sing tab: reading a score (notes, rests, ties, chords, voices, tempo, the
 * compressed .mxl) and matching its lyrics to the notes as syllables.
 */
import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { deflateRawSync, gunzipSync } from "node:zlib";
import { initEngine } from "./harness.ts";

await initEngine();
const { EnglishG2P, lyricsToSyllables } = await import("../src/wasm/engine.ts");
const { parseMusicXml, readScoreFile, singScore, morae, octaveShift } = await import("../src/show/musicxml.ts");
const { parseXml } = await import("../src/show/xml.ts");
new EnglishG2P().setDictionaryText(gunzipSync(readFileSync(new URL("../public/data/cmudict.txt.gz", import.meta.url))).toString());

type N = { p?: string; d: number; l?: string; s?: string; x?: string };
/** A note: pitch like "A4", "C#5" or none for a rest; duration in divisions; lyric and syllabic. */
function note({ p, d, l, s, x = "" }: N): string {
  const pitch = p
    ? (() => {
        const m = /^([A-G])([#b]?)(\d)$/.exec(p)!;
        const alter = m[2] === "#" ? "<alter>1</alter>" : m[2] === "b" ? "<alter>-1</alter>" : "";
        return `<pitch><step>${m[1]}</step>${alter}<octave>${m[3]}</octave></pitch>`;
      })()
    : "<rest/>";
  const lyric = l === undefined ? "" : `<lyric number="1"><syllabic>${s ?? "single"}</syllabic><text>${l}</text></lyric>`;
  return `<note>${x}${pitch}<duration>${d}</duration><voice>1</voice>${lyric}</note>`;
}
function score(measures: string[], { divisions = 2, tempo = 88, title = "Test" } = {}): string {
  const ms = measures.map((m, i) => `<measure number="${i + 1}">${i === 0 ? `<attributes><divisions>${divisions}</divisions></attributes><direction><sound tempo="${tempo}"/></direction>` : ""}${m}</measure>`);
  return `<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE score-partwise PUBLIC "-//Recordare//DTD MusicXML 4.0 Partwise//EN" "http://www.musicxml.org/dtds/partwise.dtd">
<score-partwise version="4.0"><work><work-title>${title}</work-title></work>
<part-list><score-part id="P1"><part-name>Voice</part-name></score-part></part-list>
<part id="P1">${ms.join("")}</part></score-partwise>`;
}
const notes = (...ns: N[]) => ns.map(note).join("");
const shown = (xml: string) =>
  singScore(parseMusicXml(xml), lyricsToSyllables)
    .map((n) => (n.midi === null ? "_" : `${n.syl!.melisma ? "~" : n.syl!.text}`))
    .join(" ");

describe("reading MusicXML", () => {
  test("the XML reader takes entities, CDATA, comments and a DOCTYPE with a subset", () => {
    const x = parseXml(`<?xml version="1.0"?><!DOCTYPE a [<!ENTITY x "y">]><!-- c --><a k='1 &amp; 2'>&lt;b&#x3042;<![CDATA[<c>]]><d/></a>`);
    expect(x.name).toBe("a");
    expect(x.attrs.k).toBe("1 & 2");
    expect(x.text).toBe("<bあ<c>");
    expect(x.children.map((c) => c.name)).toEqual(["d"]);
    expect(() => parseXml("<a><b></a>")).toThrow();
  });

  test("notes, rests, beats, pitch and tempo", () => {
    // 荒城の月 (Taki 1901, public domain), its first half line in D minor, then a rest
    const s = parseMusicXml(
      score([
        notes({ p: "A4", d: 2, l: "は" }, { p: "A4", d: 2, l: "る" }, { p: "D5", d: 2, l: "こ" }, { p: "E5", d: 2, l: "ー" }),
        notes({ p: "F5", d: 2, l: "ろ" }, { p: "E5", d: 2, l: "ー" }, { p: "D5", d: 2, l: "の" }, { d: 2 }),
      ]),
    );
    expect(s.title).toBe("Test");
    expect(s.bpm).toBe(88);
    // the trailing rest is dropped; a quarter is one beat at divisions 2
    expect(s.notes.map((n) => n.midi)).toEqual([69, 69, 74, 76, 77, 76, 74]);
    expect(s.notes.every((n) => n.beats === 1)).toBe(true);
  });

  test("ties join, chords sing their first note, grace notes and other voices are left out", () => {
    const tieStart = `<tie type="start"/>`;
    const tieStop = `<tie type="stop"/>`;
    const s = parseMusicXml(
      score([
        notes({ p: "C4", d: 2, l: "la", x: "" }) +
          `<note><chord/><pitch><step>E</step><octave>4</octave></pitch><duration>2</duration><voice>1</voice></note>` +
          `<note><grace/><pitch><step>B</step><octave>3</octave></pitch><voice>1</voice></note>` +
          `<note><pitch><step>D</step><octave>4</octave></pitch><duration>2</duration>${tieStart}<voice>1</voice><lyric><text>la</text></lyric></note>` +
          `<backup><duration>4</duration></backup>` +
          `<note><pitch><step>G</step><octave>3</octave></pitch><duration>4</duration><voice>2</voice></note>`,
        `<note><pitch><step>D</step><octave>4</octave></pitch><duration>4</duration>${tieStop}<voice>1</voice></note>`,
      ]),
    );
    expect(s.notes.map((n) => [n.midi, n.beats])).toEqual([
      [60, 1],
      [62, 3],
    ]);
  });

  test("a tempo change rescales the beats after it", () => {
    const xml = score([notes({ p: "C4", d: 2, l: "a" }) + `<direction><sound tempo="176"/></direction>` + notes({ p: "D4", d: 2, l: "a" })]);
    const s = parseMusicXml(xml);
    expect(s.bpm).toBe(88);
    // twice as fast: the quarter after the change lasts half a beat of the song's 88
    expect(s.notes.map((n) => n.beats)).toEqual([1, 0.5]);
  });

  test("a compressed .mxl reads through its container", async () => {
    const xml = score([notes({ p: "C4", d: 2, l: "あ" })], { title: "Zipped" });
    const container = `<?xml version="1.0"?><container><rootfiles><rootfile full-path="score/song.xml"/></rootfiles></container>`;
    const bytes = zip([
      ["mimetype", "application/vnd.recordare.musicxml", false],
      ["META-INF/container.xml", container, true],
      ["score/song.xml", xml, true],
    ]);
    const s = await readScoreFile(bytes, "song.mxl");
    expect(s.title).toBe("Zipped");
    expect(s.notes.map((n) => n.midi)).toEqual([60]);
  });

  test("not a score: a clear error", () => {
    expect(() => parseMusicXml("<html/>")).toThrow(/not a MusicXML score/);
    expect(() => parseMusicXml(`<score-timewise/>`)).toThrow(/timewise/);
  });
});

describe("lyrics on the notes", () => {
  test("kana: a mora a note, read across the phrase; a rest ends it", () => {
    const xml = score([
      notes({ p: "A4", d: 2, l: "は" }, { p: "A4", d: 2, l: "る" }, { p: "D5", d: 2, l: "こ" }, { p: "E5", d: 2, l: "ー" }),
      notes({ p: "F5", d: 2, l: "ろ" }, { p: "E5", d: 2, l: "ー" }, { p: "D5", d: 4, l: "の" }),
      notes({ d: 2 }, { p: "D5", d: 2, l: "か" }, { p: "D5", d: 2, l: "げ" }, { p: "D5", d: 2, l: "さ" }),
    ]);
    expect(shown(xml)).toBe("は る こ ~ ろ ~ の _ か げ さ");
    const sung = singScore(parseMusicXml(xml), lyricsToSyllables);
    // a breath at the rest; げ after a vowel is the sung 鼻濁音
    expect(sung[6].syl!.phraseEnd).toBe(true);
    expect(sung[9].syl!.onset.map((c) => c.ipa)).toEqual(["ŋ"]);
  });

  test("kana: ん closes on the next note's consonant, っ holds, a two-mora note splits", () => {
    const xml = score([notes({ p: "C4", d: 2, l: "さ" }, { p: "D4", d: 2, l: "ん" }, { p: "E4", d: 2, l: "ぽ" }, { p: "F4", d: 2, l: "っ" }, { p: "G4", d: 2, l: "とも" })]);
    const sung = singScore(parseMusicXml(xml), lyricsToSyllables);
    expect(sung.map((n) => n.syl!.text)).toEqual(["さ", "ん", "ぽっ", "~", "と", "も"]);
    // ん before ぽ is [m], as it would be written together
    expect(sung[1].syl!.nucleus.ipa).toBe("m");
    // とも shares its note's beat
    expect(sung.slice(4).map((n) => n.beats)).toEqual([0.5, 0.5]);
    expect(morae("きょー")).toBe(2);
  });

  test("words: a syllable a note; a held note continues the vowel", () => {
    const xml = score([
      notes({ p: "C4", d: 2, l: "Twin", s: "begin" }, { p: "C4", d: 2, l: "kle", s: "end" }, { p: "G4", d: 2, l: "twin", s: "begin" }, { p: "G4", d: 2, l: "kle", s: "end" }),
      notes({ p: "A4", d: 2, l: "lit", s: "begin" }, { p: "A4", d: 2, l: "tle", s: "end" }, { p: "G4", d: 2, l: "star" }, { p: "F4", d: 2 }),
    ]);
    expect(shown(xml)).toBe("Twin kle twin kle lit tle star ~");
  });

  test("words: the dictionary's syllables fit the word's notes", () => {
    // "every" on two notes: the dictionary reads three syllables, so the last note sings two
    const two = singScore(parseMusicXml(score([notes({ p: "C4", d: 2, l: "ev", s: "begin" }, { p: "D4", d: 2, l: "ery", s: "end" })])), lyricsToSyllables);
    expect(two.length).toBe(lyricsToSyllables("every").length);
    expect(two.reduce((a, n) => a + n.beats, 0)).toBe(2);
    // "fire" on three notes: a syllable short, the last note holds the vowel
    const three = singScore(parseMusicXml(score([notes({ p: "C4", d: 2, l: "fi", s: "begin" }, { p: "D4", d: 2, l: "-", s: "middle" }, { p: "E4", d: 2, l: "re", s: "end" })])), lyricsToSyllables);
    expect(three.length).toBe(3);
    expect(three.filter((n) => n.syl!.melisma).length).toBe(3 - lyricsToSyllables("fire").length);
  });

  test("a Japanese melisma retains the minimum vowel time", () => {
    const xml = score([notes({ p: "C4", d: 2, l: "あ" }, { p: "D4", d: 2, l: "-" })]);
    const sung = singScore(parseMusicXml(xml), lyricsToSyllables);
    expect(sung.map((n) => n.syl!.minVowel)).toEqual([0.1, 0.1]);
    expect(sung[1].syl!.melisma).toBe(true);
  });

  test("no lyrics: the typed syllables, a note each", () => {
    const xml = score([notes({ p: "C4", d: 2 }, { p: "D4", d: 2 }, { p: "E4", d: 2 })]);
    const sung = singScore(parseMusicXml(xml), lyricsToSyllables, lyricsToSyllables("la di"));
    expect(sung.map((n) => n.syl!.text)).toEqual(["la", "di", "la"]);
  });

  test("the octave moves the melody's middle into the voice", () => {
    const melody = [{ midi: 69 }, { midi: 74 }, { midi: 77 }];
    // D5 in the middle, as 荒城の月 is written: a woman sings it there, a man an octave down
    expect(octaveShift(melody, 1)).toBe(0);
    expect(octaveShift(melody, 0)).toBe(-12);
    // a tenor part written an octave up (treble clef 8vb): a man sings it an octave down
    expect(octaveShift([{ midi: 67 }, { midi: 69 }, { midi: 71 }], 0)).toBe(-12);
    expect(octaveShift([{ midi: 55 }, { midi: null }], 0)).toBe(0);
  });
});

/** A zip archive of [name, text, deflate?] entries. */
function zip(files: [string, string, boolean][]): Uint8Array {
  const enc = new TextEncoder();
  const parts: Uint8Array[] = [];
  const central: Uint8Array[] = [];
  let offset = 0;
  for (const [name, text, deflate] of files) {
    const raw = enc.encode(text);
    const data = deflate ? new Uint8Array(deflateRawSync(raw)) : raw;
    const nameBytes = enc.encode(name);
    const local = new DataView(new ArrayBuffer(30));
    local.setUint32(0, 0x04034b50, true);
    local.setUint16(8, deflate ? 8 : 0, true);
    local.setUint32(18, data.length, true);
    local.setUint32(22, raw.length, true);
    local.setUint16(26, nameBytes.length, true);
    const dir = new DataView(new ArrayBuffer(46));
    dir.setUint32(0, 0x02014b50, true);
    dir.setUint16(10, deflate ? 8 : 0, true);
    dir.setUint32(20, data.length, true);
    dir.setUint32(24, raw.length, true);
    dir.setUint16(28, nameBytes.length, true);
    dir.setUint32(42, offset, true);
    parts.push(new Uint8Array(local.buffer), nameBytes, data);
    central.push(new Uint8Array(dir.buffer), nameBytes);
    offset += 30 + nameBytes.length + data.length;
  }
  const size = central.reduce((a, b) => a + b.length, 0);
  const end = new DataView(new ArrayBuffer(22));
  end.setUint32(0, 0x06054b50, true);
  end.setUint16(8, files.length, true);
  end.setUint16(10, files.length, true);
  end.setUint32(12, size, true);
  end.setUint32(16, offset, true);
  const all = [...parts, ...central, new Uint8Array(end.buffer)];
  const out = new Uint8Array(all.reduce((a, b) => a + b.length, 0));
  let p = 0;
  for (const a of all) {
    out.set(a, p);
    p += a.length;
  }
  return out;
}
