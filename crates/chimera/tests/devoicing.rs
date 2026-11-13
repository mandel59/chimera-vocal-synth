//! Japanese vowel devoicing in speech (issue #12): which morae, and what the voice does in them.
use chimera::anatomy::human::BodyMix;
use chimera::phonetics::{
    ipa::tokenize_ipa,
    japanese,
    planner::{plan_phrases, PhraseInput, PlanOptions},
    Phrase,
};
use chimera::voice::human_voice::{HumanVoice, SourcePatch};

/// The spoken segments of a text, each devoiced vowel written with its fricative: `ɯ(s̪)`.
fn said(text: &str) -> String {
    japanese::to_phrases(text)
        .iter()
        .flat_map(|p| &p.segments)
        .map(|s| match &s.devoiced {
            Some(h) => format!("{}({h})", s.ipa),
            None => s.ipa.clone(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn high_vowels_between_voiceless_consonants_are_devoiced() {
    assert_eq!(said("すき"), "s̪ ɯ(s̪) k i");
    assert_eq!(said("あした"), "a ɕ i(ɕ) t̪ a");
    assert_eq!(said("ひと"), "ç i(ç) t̪ o");
    assert_eq!(said("ふた"), "ɸ ɯ(ɸ) t̪ a");
    assert_eq!(said("ちかい"), "tɕ i(ɕ) k a i");
    assert_eq!(said("つき"), "t̪s̪ ɯ(s̪) k i");
    // a stop releases into the fricative of the vowel's place
    assert_eq!(said("きた"), "k i(ç) t̪ a");
    assert_eq!(said("くさ"), "k ɯ() s̪ a");
    assert_eq!(said("ぷか"), "p ɯ(ɸ) k a");
    // at the phrase's end, after a fricative or affricate
    assert_eq!(said("です"), "d̪ e s̪ ɯ(s̪)");
    assert_eq!(said("あき"), "a k i");
}

#[test]
fn the_other_vowels_and_contexts_keep_their_voice() {
    // not a, e, o; not before or after a voiced sound
    assert_eq!(said("かた"), "k a t̪ a");
    assert_eq!(said("すな"), "s̪ ɯ n̪ a");
    assert_eq!(said("ずき"), "d̪z̪ ɯ k i");
    // not a long vowel, not before っ, not a palatalized onset
    assert_eq!(said("すーき"), "s̪ ɯ ɯ k i");
    assert_eq!(said("きって"), "k i t̪ t̪ e");
    assert!(!said("きゅっ").contains('('));
    // not two morae in a row
    assert_eq!(said("きくち"), "k i(ç) k ɯ tɕ i(ɕ)");
    // not the accented mora: the fall is heard on it
    assert_eq!(said("き＼た"), "k i t̪ a");
    assert_eq!(said("あし＼た"), "a ɕ i t̪ a");
    // a phrase boundary is an end, the next phrase a new start
    assert_eq!(said("です、か"), "d̪ e s̪ ɯ(s̪) k a");
}

#[test]
fn only_japanese_speech_devoices() {
    for kana in ["すき", "です", "あした", "きくち"] {
        let mut prev = None;
        for s in japanese::kana_to_syllables(kana, None) {
            assert!(s.onset.iter().chain(&s.coda).chain([&s.nucleus]).all(|c| c.devoiced.is_none()), "sung {kana}");
            prev = Some(s);
        }
        assert!(prev.is_some());
    }
    assert!(tokenize_ipa("sɯki").iter().all(|s| s.devoiced.is_none()));
}

struct Heard {
    /// mean voicing over the segment
    voice: f64,
    /// share of the segment with the tract narrower than 0.35 cm² somewhere
    narrow: f64,
    /// RMS of the output over the segment
    rms: f64,
    dur: f64,
}

/// What each planned segment of a spoken text does, by 1 ms steps.
fn heard(phrases: &[Phrase], body: BodyMix, f0: f64, rate: f64) -> Vec<(String, Heard)> {
    let input: Vec<_> = phrases
        .iter()
        .cloned()
        .map(|p| PhraseInput { segments: p.segments, intonation: Some(p.intonation), pause: Some(p.pause), rate: p.rate, register: p.register, aspiration: p.aspiration })
        .collect();
    let p = plan_phrases(&input, &PlanOptions { body, t0: 0.05, f0: Some(f0), rate: Some(rate), ..Default::default() });
    let mut v = HumanVoice::new(48000.0, body.body(), 2);
    v.set_source(&SourcePatch::f0(f0));
    v.schedule(&p.gestures);
    let mut b = [0.0f32; 48];
    let mut rows = Vec::new();
    while v.clock < p.end {
        v.render(&mut b);
        let area = v.airway.ph_areas.iter().chain(&v.airway.or_areas).copied().fold(f64::INFINITY, f64::min);
        rows.push((v.clock, v.glottis.voicing, area, b.iter().map(|x| (*x as f64).powi(2)).sum::<f64>() / 48.0));
    }
    p.segments
        .iter()
        .map(|s| {
            let r: Vec<_> = rows.iter().filter(|r| r.0 >= s.t0 && r.0 < s.t1).collect();
            let n = r.len() as f64;
            (s.ipa.clone(), Heard {
                voice: r.iter().map(|r| r.1).sum::<f64>() / n,
                narrow: r.iter().filter(|r| r.2 < 0.35).count() as f64 / n,
                rms: (r.iter().map(|r| r.3).sum::<f64>() / n).sqrt(),
                dur: s.t1 - s.t0,
            })
        })
        .collect()
}

const BODIES: [(BodyMix, f64); 3] = [(BodyMix::MALE, 110.0), (BodyMix::FEMALE, 200.0), (BodyMix::CHILD, 260.0)];

/// The mora stays: its time, a sound in it, the consonant's constriction through it, no voice,
/// and the vowel after it voices as it did. (Dropping the vowel was heard as a missing mora;
/// parting the folds over an open vowel, as an [h].)
#[test]
fn a_devoiced_mora_keeps_its_time_and_place_without_voice() {
    // (text, the devoiced segment, the next vowel)
    let cases = [("すき", 1, 3), ("あした", 2, 4), ("ひと", 1, 3), ("きた", 1, 3), ("くさ", 1, 3), ("つき", 1, 3)];
    for (body, f0) in BODIES {
        for rate in [0.7, 1.0, 1.5] {
            for (text, at, after) in cases {
                let devoiced = japanese::to_phrases(text);
                let mut voiced = devoiced.clone();
                voiced[0].segments[at].devoiced = None;
                let (d, v) = (heard(&devoiced, body, f0, rate), heard(&voiced, body, f0, rate));
                let ctx = format!("{body:?} {text} rate={rate}");
                eprintln!("{ctx}: voice {:.2} -> {:.2}, narrow {:.2} -> {:.2}, rms {:.4} -> {:.4}, next vowel {:.2} -> {:.2}", v[at].1.voice, d[at].1.voice, v[at].1.narrow, d[at].1.narrow, v[at].1.rms, d[at].1.rms, v[after].1.voice, d[after].1.voice);
                assert_eq!(d[at].1.dur, v[at].1.dur, "the mora lost time: {ctx}");
                assert!(d[at].1.voice < 0.05, "still voiced: {ctx} {}", d[at].1.voice);
                assert!(d[at].1.rms > 0.0005, "a silent mora: {ctx} {}", d[at].1.rms);
                assert!(d[after].1.voice > v[after].1.voice - 0.1, "the next vowel lost voice: {ctx}");
                // く holds nothing: う is narrow itself, between the two closures
                let narrow = if text == "くさ" { 0.6 } else { 0.8 };
                assert!(d[at].1.narrow > narrow, "the place let go: {ctx} {}", d[at].1.narrow);
            }
        }
    }
}
