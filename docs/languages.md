# Language selection

The fork's Japanese and browser MusicXML features are ported onto upstream
`b6969d5a63d5`, retaining its Russian lyrics, IPA notices, mood tunes, video recording and scramble judge
(original port base `1d3ea496cad0`, originally `eed4a5ff2695ad151df2210d012b2a4ab584f099`), from `japanese-lyrics` at
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

Japanese speech uses a closed uvular nasal `ɴ` for a final `ん` at the end of
a phrase, including before a punctuation pause. This is a simple default,
not a model of the variation with the preceding vowel and speaker
([Maekawa](https://doi.org/10.1017/S0025100321000050)). Accent-phrase word
boundaries alone do not trigger it: `ほん を` keeps an open `ɰ̃`. Existing
assimilation before consonants is retained. Singing and an explicit `ー`
continuation keep the open nasal for sustained notes; the quietness of a
held `ɴ` does not determine the articulation of ordinary speech.

Common-Japanese kana merge じ/ぢ and ず/づ: either spelling takes the same contextual
onset rule. This front end defaults to affricates at a word/run start and after a nasal or
geminate, and fricatives after a vowel (including its long mark): dʑ/ʑ for じ/ぢ and
laminal denti-alveolar d̪z̪/z̪ for ず/づ and the other z-row vowels. The choice is made
before nasal/geminate assimilation. Original spelling is retained for display. This is a
simplified deterministic default, not a categorical claim about every speaker or a model
of dialects that distinguish four sounds. Shared IPA inventory and typed IPA do not change.

Spoken voiced-affricate onsets explicitly use the shared inventory's inherent duration
before the Japanese phrase rate is applied. Otherwise English's 50 ms /dʒ/-style Klatt
value shortens Japanese /dʑ/ again to 25.6 ms, leaving its planned release nearly closed.
The existing 120 ms inventory value gives 61.5 ms at the default rate, without changing
the shared sound, English, typed IPA or singing. The physical release and following
vowel are covered in `tests/voiced_affricates.rs`; perceptual quality remains issue #8.

`render::say` keeps its English signature for the unmodified TSP judge. CLI
`render text` selects a language (`--lang en|ja`, or compatibility detection)
and uses the same rendering body after any dictionary preparation. Its
speaking pitches, 50 ms plan origin and 300 ms tail remain upstream's.
`render notes` accepts an optional `"language": "en" | "ja"` in its melody JSON.
The judge's alignment and scoring code are unchanged.

The original Japanese engine, dictionary integration and MusicXML implementation were
co-authored with Claude Opus 5.5 (see `6c721e5`, `88afee8`, `79dcc200`). Work
without a model attribution in the prior history used Codex, as confirmed by the repository
owner. The present port and integration use GPT-6.1 Sol in Codex. Later phonetics
commits `3ee34ff7` and `bcfd487a` retain their recorded Claude Fable 5.2 provenance.

## Singing and scores

Mood tunes still fit the player's own words, with saved tempo, key and language. Original
demos and loaded scores share the same playback, stop/cancellation and video hooks; their
temporary lyrics do not replace the saved player lyrics.

The browser MusicXML reader produces notes and rests; its existing lyric
alignment feeds `Syllable` values to `plan_song`. Explicit language selection
also controls this alignment. The CLI MusicXML converter remains upstream's
separate English tool. Live keys, CLI and browser song assembly continue to
use their existing planning APIs.

This port does not introduce a new prosody representation, language-span
model, source-map IDs, score model, shared whole-song compiler, resource
registry or diagnostic schema. Speech does not analyze mixed-language spans.
Lyrics retain upstream's English/Korean/Russian script router; Japanese selection adds a kana
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
