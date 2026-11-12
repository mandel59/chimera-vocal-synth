# Language selection

The fork's Japanese and browser MusicXML features are ported onto upstream
`1d3ea496cad0` (originally `eed4a5ff2695ad151df2210d012b2a4ab584f099`), from `japanese-lyrics` at
`79dcc2000a5b6a5c3e573416657b5ab953f9f0a0` (issue #5).

`phonetics::text` selects English or Japanese before the existing planners:

```text
text -> Phrase / Segment -> plan_phrases -> Plan -> HumanVoice
lyrics -> Syllable       -> plan_song    -> Plan -> HumanVoice
```

Select `Language::English` / `Language::Japanese` in Rust, or `"en"` / `"ja"` in
the JSON and TypeScript interfaces. `en-US` and `ja-JP` are accepted by Rust's
parser and serde. An omitted selection uses the existing kana/kanji heuristic
at this entry. It is compatibility behavior, not mixed-language analysis.
Unknown language codes are errors.

`EnglishG2P` retains the upstream English behavior. `Phrase` lives beside
`Segment` and is re-exported from `english` for existing callers. Japanese
supplies explicit durations, mora pitches and optional phrase rate, register
and aspiration; absent settings preserve the upstream English behavior.
CLI and WASM call this language entry. The WASM facade passes `PhraseInput`
without reconstructing and potentially losing its fields.

## Readings and rendering

The core has no dictionary I/O. The English dictionary keeps its upstream
host-side loading and unknown-word fallback. The Japanese core reads kana.
The CLI's optional `jdic` feature uses jpreprocess/naist-jdic to turn kanji
text into kana with accent marks before calling the same Japanese frontend.
Hand-marked readings are retained. The browser does not load that dictionary
and reports a kanji-reading error rather than silently dropping the kanji.

`render::say` keeps its English signature for the unmodified TSP judge. CLI
`render text` selects a language (`--lang en|ja`, or compatibility detection)
and uses the same rendering body after any dictionary preparation. Its
speaking pitches, 50 ms plan origin and 300 ms tail remain upstream's.
`render notes` accepts an optional `"language": "en" | "ja"` in its melody JSON.
The judge's alignment and scoring code are unchanged.

## Singing and scores

The browser MusicXML reader produces notes and rests; its existing lyric
alignment feeds `Syllable` values to `plan_song`. Explicit language selection
also controls this alignment. The CLI MusicXML converter remains upstream's
separate English tool. Live keys, CLI and browser song assembly continue to
use their existing planning APIs.

This port does not introduce a new prosody representation, language-span
model, source-map IDs, score model, shared whole-song compiler, resource
registry or diagnostic schema. Speech does not analyze mixed-language spans.
Lyrics retain upstream's English/Korean script router; Japanese selection adds a kana
frontend, including long-vowel continuations across words. Shared IPA sounds use upstream's
definitions. UI character checks are only replaced where the explicit language
is already available. Analysis and planning remain outside audio rendering;
the gesture layout and AudioWorklet are unchanged.

## Checks

`tests/language.test.ts` checks language selection and timing, register,
aspiration and mora pitch through the WASM boundary. The fork's Japanese,
dictionary, MusicXML and lyric-display regressions are retained. English
audio/plans are compared with upstream, Japanese with the original fork.
Run the repository checks in AGENTS.md; `cargo test -p chimera-cli --features
jdic` additionally checks the dictionary path.
