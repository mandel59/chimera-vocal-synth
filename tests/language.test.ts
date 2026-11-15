import { describe, expect, test } from "bun:test";
import { initEngine } from "./harness.ts";

await initEngine();
const { EnglishG2P, textToPhrases, lyricsToSyllables, planPhrases, planSong } = await import("../src/wasm/engine.ts");
const { call } = await import("../src/wasm/chimera.ts");

describe("language entry over WASM", () => {
  test("Japanese selection keeps Russian prepositions and cross-script vowel holds", () => {
    const mixed = lyricsToSyllables("さ в лесу́ 빵야 ー star", "ja");
    expect(mixed.map((s) => s.text)).toEqual(["さ", "в ле", "су́", "빵", "야", "ー", "star"]);
    expect(mixed[1].onset[0].ipa).toBe("v");
    expect(mixed[5].nucleus).toEqual(mixed[4].nucleus);
    expect(lyricsToSyllables("в、 лесу́", "ja")[0].text).toBe("ле");
    expect(lyricsToSyllables("в あ лесу́", "ja")[1].text).toBe("ле");
  });

  test("Japanese selection preserves the upstream Korean lyric frontend", () => {
    const korean = lyricsToSyllables("빵야", "en");
    expect(korean.map((s) => s.text)).toEqual(["빵", "야"]);
    const mixed = lyricsToSyllables("さ 빵야 ー", "ja");
    expect(mixed.map((s) => s.text)).toEqual(["さ", "빵", "야", "ー"]);
    expect(mixed.slice(1, 3)).toEqual(korean);
    expect(mixed[3].nucleus).toEqual(korean[1].nucleus);
    expect(mixed[3].melisma).toBe(true);
  });

  test("English keeps its upstream entry, Japanese is selected outside EnglishG2P", () => {
    const g = new EnglishG2P();
    expect(textToPhrases("Can you hear me?", "en")).toEqual(g.toPhrases("Can you hear me?"));
    expect(g.toPhrases("さくら")).toEqual([]);
    expect(textToPhrases("さくら", "en")).toEqual([]);
    expect(textToPhrases("さくら")).toEqual(textToPhrases("さくら", "ja"));
    expect(lyricsToSyllables("さくら", "ja").map((s) => s.text)).toEqual(["さ", "く", "ら"]);
    expect(lyricsToSyllables("さくら", "en")).toEqual([]);
  });

  test("speech timing, register, aspiration and mora pitch reach the planner", () => {
    const phrases = textToPhrases("か＼さ。", "ja");
    const p = phrases[0];
    expect(p.rate).toBeGreaterThan(1);
    expect(p.register).toBeDefined();
    expect(p.aspiration).toBeLessThan(1);
    expect(p.segments.some((s) => s.pitch !== undefined)).toBe(true);
    const options = { body: { female: 0, child: 0 }, t0: 0.05, f0: 112 };
    const actual = planPhrases(phrases, options);
    const slower = planPhrases([{ ...p, rate: 1 }], options);
    const neutralPitch = planPhrases([{ ...p, register: [0, 0] }], options);
    const aspirated = planPhrases([{ ...p, aspiration: 1 }], options);
    expect(actual.end).toBeLessThan(slower.end);
    const f0 = (plan: typeof actual) => plan.gestures.find((g) => g.label === "pitch")!.targets.f0!;
    expect(f0(actual)).toBeLessThan(f0(neutralPitch));
    const release = (plan: typeof actual) => plan.gestures.find((g) => g.label === "aspiration")!.rampOut;
    expect(release(actual)).toBeLessThan(release(aspirated));
    expect(actual.gestures.filter((g) => g.label === "pitch").length).toBe(2);
  });

  test("unsupported languages and unavailable kanji readings report errors", () => {
    expect(() => call("textPhrases", { text: "bonjour", language: "fr" })).toThrow("language");
    expect(() => textToPhrases("東京", "ja")).toThrow("kana");
    expect(() => lyricsToSyllables("桜", "ja")).toThrow("kana");
  });
});


test("common Japanese merges yotsugana spellings over WASM", () => {
  const phrases = textToPhrases("じ。ず。ぢ。づ。", "ja");
  expect(phrases.map((p) => p.segments.map((s) => s.ipa))).toEqual([
    ["dʑ", "i"], ["d̪z̪", "ɯ"], ["dʑ", "i"], ["d̪z̪", "ɯ"],
  ]);
  expect(phrases[0].segments).toEqual(phrases[2].segments);
  expect(phrases[1].segments).toEqual(phrases[3].segments);
  for (const [a, b] of [["あじ", "あぢ"], ["あず", "あづ"], ["かんじ", "かんぢ"], ["かんず", "かんづ"]]) {
    expect(textToPhrases(a, "ja")[0].segments).toEqual(textToPhrases(b, "ja")[0].segments);
    expect(lyricsToSyllables(a, "ja").map((s) => [s.onset, s.nucleus, s.coda])).toEqual(
      lyricsToSyllables(b, "ja").map((s) => [s.onset, s.nucleus, s.coda]),
    );
  }
});

test("Japanese voiced affricates keep their inherent duration before the phrase rate", () => {
  const phrases = textToPhrases("じ。", "ja");
  const consonant = phrases[0].segments.find((s) => s.ipa === "dʑ")!;
  expect(consonant.dur).toBe(0.12);
  const p = planPhrases(phrases, { body: { female: 0, child: 0 }, t0: 0.05 });
  const span = p.segments.find((s) => s.ipa === "dʑ")!;
  expect(span.t1 - span.t0).toBeCloseTo(0.12 / phrases[0].rate!, 6);
  expect(span.t1 - span.t0).toBeGreaterThan(0.05);
  expect(lyricsToSyllables("じ", "ja")[0].onset[0].dur).toBeUndefined();
});

test("Japanese b keeps its requested vowel-body timing through the WASM facade", () => {
  const phrases = textToPhrases("あび", "ja");
  expect(phrases[0].segments.find((s) => s.ipa === "b")!.vowelTransition).toBe("afterRelease");
  const legacy = phrases.map((p) => ({ ...p, segments: p.segments.map(({ vowelTransition, ...s }) => s) }));
  const actual = planPhrases(phrases, { body: { female: 0, child: 0 }, t0: 0.05 });
  const before = planPhrases(legacy, { body: { female: 0, child: 0 }, t0: 0.05 });
  expect(actual.segments).toEqual(before.segments);
  expect(actual.end).toBe(before.end);
  const body = actual.gestures.find((g) => g.label === "i" && g.targets.tbY !== undefined)!;
  expect(body.rampIn).toBe(0);
  expect(before.gestures.find((g) => g.label === "i")!.rampIn).toBeGreaterThan(0);
  expect(lyricsToSyllables("あび", "ja")[1].onset[0].vowelTransition).toBe("afterRelease");
});

test("Japanese sung b carries its pressure guard across words and melisma without moving beats", () => {
  const syls = lyricsToSyllables("あ ー び ぶ べ ぼ", "ja");
  expect(syls.flatMap((s) => s.onset).map((s) => s.vowelTransition)).toEqual([
    "afterRelease", "afterRelease", undefined, undefined,
  ]);
  const notes = syls.map((syl) => ({ syl, midi: 60, beats: 0.5 }));
  const options = { body: { female: 0, child: 0 }, bpm: 100, t0: 0.3, vibrato: 35 };
  const actual = planSong(notes, options);
  const before = planSong(notes.map((n) => ({ ...n, syl: { ...n.syl, onset: n.syl.onset.map(({ vowelTransition, ...s }) => s) } })), options);
  expect(actual.segments).toEqual(before.segments);
  const vi = actual.segments.find((s) => s.ipa === "i")!;
  const body = actual.gestures.find((g) => g.label === "i" && g.targets.tbY !== undefined)!;
  expect(body.t0).toBeCloseTo(vi.t0 + 0.015, 8);
  expect(body.rampIn).toBe(0);
  const vo = actual.segments[actual.segments.length - 1];
  const bo = actual.gestures.find((g) => g.label === "o" && g.t0 === vo.t0)!;
  expect(bo.targets.tbY).toBeDefined();
  expect(bo.targets.lipAp).toBeDefined();
  expect(bo.rampIn).toBeGreaterThan(0);
});

test("Japanese be-bo keeps vowel overlap while bi-bu retains the pressure guard", () => {
  const phrases = textToPhrases("ばびぶべぼ", "ja");
  expect(phrases[0].segments.filter((s) => s.ipa === "b").map((s) => s.vowelTransition)).toEqual([
    undefined, "afterRelease", "afterRelease", undefined, undefined,
  ]);
  const plan = planPhrases(phrases, { body: { female: 0, child: 0 }, t0: 0.05 });
  const vowel = plan.segments[plan.segments.length - 1];
  const body = plan.gestures.find((g) => g.label === "o" && g.t0 === vowel.t0 && g.targets.tbY !== undefined)!;
  expect(body.targets.lipAp).toBeDefined();
  expect(body.rampIn).toBeGreaterThan(0);
});
