//! Japanese and Russian require different context from the shared lyrics router.
use chimera::phonetics::english::EnglishG2P;
use chimera::phonetics::ipa::skipped_ipa;
use chimera::phonetics::sing::{SongOptions, SungNote, plan_song};
use chimera::phonetics::syllables::lyrics_with_kana;
use chimera::phonetics::{Segment, Syllable};

#[test]
fn japanese_selection_keeps_russian_words_and_their_prepositions() {
    let g = EnglishG2P::new();
    let syllables = lyrics_with_kana("さ в лесу́ 빵야 ー star", &g, true);
    assert_eq!(
        syllables
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>(),
        ["さ", "в ле", "су\u{301}", "빵", "야", "ー", "star"]
    );
    assert_eq!(syllables[5].nucleus, syllables[4].nucleus);
    assert_eq!(syllables[5].melisma, Some(true));
    assert_eq!(syllables[1].onset[0].ipa, "v");
}

#[test]
fn russian_word_context_stops_at_japanese_words_and_punctuation() {
    let g = EnglishG2P::new();
    for (text, first) in [("в、 лесу́", 0), ("в あ лесу́", 1)] {
        let syllables = lyrics_with_kana(text, &g, true);
        assert_eq!(syllables[first].text, "ле", "{text}");
        assert_eq!(syllables[first].onset[0].ipa, "l", "{text}");
    }
    let syllables = lyrics_with_kana("あ ー", &g, true);
    assert_eq!(syllables[1].nucleus, syllables[0].nucleus);
}

#[test]
fn explicit_sung_consonant_length_and_voiced_release_are_both_retained() {
    let notes = [SungNote {
        midi: 60.0,
        beats: 1.0,
        syl: Syllable {
            onset: vec![Segment {
                ipa: "dʑ".into(),
                dur: Some(0.18),
                release_abduction: Some(0.15),
                ..Default::default()
            }],
            nucleus: Segment {
                ipa: "a".into(),
                ..Default::default()
            },
            ..Default::default()
        },
    }];
    let plan = plan_song(
        &notes,
        &SongOptions {
            bpm: 60.0,
            t0: 0.5,
            ..Default::default()
        },
    );
    let onset = plan.segments.iter().find(|s| s.ipa == "dʑ").unwrap();
    assert!((onset.t1 - onset.t0 - 0.18).abs() < 1e-9);
    assert!(
        plan.gestures
            .iter()
            .any(|g| g.label_str() == "voiced release")
    );
}

#[test]
fn japanese_ipa_diacritics_are_sounds_rather_than_ignored_marks() {
    assert!(skipped_ipa("t̪ d̪ n̪ s̪ z̪ t̪s̪ d̪z̪ ɰ̃ t͡ɕ d͡ʑ").is_empty());
}
