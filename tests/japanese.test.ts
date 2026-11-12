import { expect, test } from "bun:test";
import { initEngine } from "./harness.ts";
await initEngine();
const { lyricsToSyllables, planUtterance, planPhrases, textToPhrases, planSong, DEMO_SONGS } = await import("../src/wasm/engine.ts");
const body = { female: 0, child: 0 };

test("afterBreath survives the JSON API and only changes terminal articulation", () => {
  const plain = [{ ipa: "a" }, { ipa: "ɴ" }];
  const marked = [{ ipa: "a" }, { ipa: "ɴ", releaseTiming: "afterBreath" as const }];
  const a = planUtterance(plain, { body, t0: 0 });
  const b = planUtterance(marked, { body, t0: 0 });
  expect(b.segments).toEqual(a.segments);
  expect(b.end).toBe(a.end);
  expect(b.gestures).not.toEqual(a.gestures);
  const phrases = textToPhrases("ほん。", "ja");
  expect(phrases[0].segments.at(-1)!.releaseTiming).toBe("afterBreath");
  expect(planPhrases(phrases, { body, t0: 0 }).gestures.length).toBeGreaterThan(0);
});

test("attached N uses one note, displays ordinary kana and reports invalid input", () => {
  expect(lyricsToSyllables("あん")).toHaveLength(2);
  const x = lyricsToSyllables("あ𛅧た");
  expect(x.map((s) => s.text)).toEqual(["あん", "た"]);
  expect(x[0].coda[0].ipa).toBe("n̪");
  for (const s of ["𛅧", "ん𛅧", "あ𛅧𛅧", "あ 𛅧", "あ𛅧っか"]) expect(() => lyricsToSyllables(s)).toThrow();
  expect(() => lyricsToSyllables("あ𛅧", "en")).toThrow();
  const demo = DEMO_SONGS.find((s) => s.name === "あんたがたどこさ")!;
  expect(lyricsToSyllables(demo.lyrics)).toHaveLength(demo.notes.length);
});


test("Japanese short vowels retain their opt-in time budget through WASM", () => {
  const syl = lyricsToSyllables("くまもとさ");
  expect(syl[3].minVowel).toBe(0.1);
  expect(syl[3].onset[0].aspirated).toBe(false);
  const beats = [0.75, 0.25, 0.75, 0.25, 1];
  const notes = syl.map((syl, i) => ({ syl, midi: 65, beats: beats[i] }));
  const p = planSong(notes, { body, bpm: 100, t0: 0.3, vibrato: 0 });
  const o = p.segments.find((s) => s.ipa === "o" && Math.abs(s.t0 - 1.35) < 1e-6)!;
  expect(o.t1 - o.t0).toBeCloseTo(0.1, 6);
  expect(p.segments.some((s) => s.ipa === "a" && Math.abs(s.t0 - 1.5) < 1e-6)).toBe(true);
  for (const text of ["hello", "안녕"]) expect(lyricsToSyllables(text).every((s) => s.minVowel === undefined)).toBe(true);
});
