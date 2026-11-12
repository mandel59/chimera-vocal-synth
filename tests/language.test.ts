import { describe, expect, test } from "bun:test";
import { initEngine } from "./harness.ts";

await initEngine();
const { EnglishG2P, textToPhrases, lyricsToSyllables, planPhrases } = await import("../src/wasm/engine.ts");
const { call } = await import("../src/wasm/chimera.ts");

describe("language entry over WASM", () => {
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
