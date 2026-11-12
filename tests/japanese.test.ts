import { expect, test } from "bun:test";
import { initEngine } from "./harness.ts";
await initEngine();
const { planUtterance, planPhrases, textToPhrases } = await import("../src/wasm/engine.ts");
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
