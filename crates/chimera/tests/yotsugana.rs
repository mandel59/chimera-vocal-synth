//! Spelling must not split the common-Japanese じ/ぢ and ず/づ series.
use chimera::phonetics::{english::EnglishG2P, japanese, text};

fn spoken(s: &str) -> Vec<Vec<String>> {
    japanese::to_phrases(s)
        .iter()
        .map(|p| p.segments.iter().map(|s| s.ipa.clone()).collect())
        .collect()
}

fn plan(s: &str) -> serde_json::Value {
    use chimera::phonetics::planner::{plan_phrases, PhraseInput};
    let input = japanese::to_phrases(s)
        .into_iter()
        .map(|p| PhraseInput {
            segments: p.segments,
            intonation: Some(p.intonation),
            pause: Some(p.pause),
            rate: p.rate,
            register: p.register,
            aspiration: p.aspiration,
        })
        .collect::<Vec<_>>();
    serde_json::to_value(plan_phrases(&input, &Default::default())).unwrap()
}

#[test]
fn isolated_yotsugana_share_the_same_two_onsets() {
    assert_eq!(
        spoken("じ。ず。ぢ。づ。"),
        [
            vec!["dʑ", "i"],
            vec!["d̪z̪", "ɯ"],
            vec!["dʑ", "i"],
            vec!["d̪z̪", "ɯ"]
        ]
    );
    assert_eq!(spoken("ジ。ズ。ヂ。ヅ。"), spoken("じ。ず。ぢ。づ。"));
}

#[test]
fn affrication_depends_on_the_same_context_for_both_spellings() {
    for (a, b, want) in [
        ("あじ", "あぢ", vec!["a", "ʑ", "i"]),
        ("あず", "あづ", vec!["a", "z̪", "ɯ"]),
        ("あーじ", "あーぢ", vec!["a", "a", "ʑ", "i"]),
        ("あーず", "あーづ", vec!["a", "a", "z̪", "ɯ"]),
        ("かんじ", "かんぢ", vec!["k", "a", "ɲ", "dʑ", "i"]),
        ("かんず", "かんづ", vec!["k", "a", "n̪", "d̪z̪", "ɯ"]),
        ("あっじ", "あっぢ", vec!["a", "d", "dʑ", "i"]),
        ("あっず", "あっづ", vec!["a", "d̪", "d̪z̪", "ɯ"]),
    ] {
        assert_eq!(spoken(a), vec![want], "{a}");
        assert_eq!(spoken(a), spoken(b), "{a} / {b}");
        assert_eq!(plan(a), plan(b), "{a} / {b}: gestures and timing");
        // Singing uses the same rule and keeps the input spelling for display.
        let x = text::lyrics_to_syllables(a, None, &EnglishG2P::new()).unwrap();
        let y = text::lyrics_to_syllables(b, None, &EnglishG2P::new()).unwrap();
        assert_eq!(x.len(), y.len());
        for (a, b) in x.iter().zip(y) {
            assert_eq!(a.onset, b.onset);
            assert_eq!(a.nucleus, b.nucleus);
            assert_eq!(a.coda, b.coda);
        }
    }
}

#[test]
fn palatalized_and_dental_series_keep_their_places_and_boundaries() {
    assert_eq!(spoken("じゃ。ぢゃ。"), [vec!["dʑ", "a"], vec!["dʑ", "a"]]);
    assert_eq!(
        spoken("あじゃ。あぢゃ。"),
        [vec!["a", "ʑ", "a"], vec!["a", "ʑ", "a"]]
    );
    assert_eq!(spoken("ざぜぞ"), [vec!["d̪z̪", "a", "z̪", "e", "z̪", "o"]]);
    assert_eq!(spoken("あ ず"), [vec!["a", "d̪z̪", "ɯ"]]);
    assert_eq!(spoken("んーず"), [vec!["ɰ̃", "ɰ̃", "d̪z̪", "ɯ"]]);
    let g = EnglishG2P::new();
    let x = text::lyrics_to_syllables("あ𛅧ず", None, &g).unwrap();
    assert_eq!(x[0].coda[0].ipa, "n̪");
    assert_eq!(x[1].onset[0].ipa, "d̪z̪");
}
