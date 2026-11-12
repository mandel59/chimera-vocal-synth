import { expect, test } from "bun:test";
import { initEngine } from "./harness.ts";
await initEngine();
const { lyricsToSyllables, planUtterance, planPhrases, textToPhrases, DEMO_SONGS } = await import("../src/wasm/engine.ts");
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
