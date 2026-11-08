//! Japanese lyrics: kana → one syllable per mora.
use chimera::phonetics::Segment;
use chimera::phonetics::syllables::lyrics_to_syllables;

// Japanese lyrics never reach the English dictionary
fn word(_w: &str, _reduce: bool) -> Vec<Segment> {
    Vec::new()
}

/// "text:onset.nucleus.coda" per syllable, "~" marking a melisma
fn show(text: &str) -> String {
    let ipa = |xs: &[Segment]| xs.iter().map(|x| x.ipa.as_str()).collect::<String>();
    lyrics_to_syllables(text, &word)
        .iter()
        .map(|s| {
            let m = if s.melisma == Some(true) { "~" } else { "" };
            let p = if s.phrase_end { "|" } else { "" };
            format!("{m}{}:{}.{}.{}{p}", s.text, ipa(&s.onset), s.nucleus.ipa, ipa(&s.coda))
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn one_syllable_per_kana() {
    assert_eq!(show("さくら"), "さ:s.a. く:k.ɯ. ら:ɾ.a.");
    assert_eq!(show("しちつふひ"), "し:ɕ.i. ち:tɕ.i. つ:ts.ɯ. ふ:ɸ.ɯ. ひ:ç.i.");
}

#[test]
fn katakana_reads_as_hiragana() {
    assert_eq!(show("サクラ"), show("さくら").replace('さ', "サ").replace('く', "ク").replace('ら', "ラ"));
}

#[test]
fn small_kana_join_the_mora_before() {
    assert_eq!(show("きゃしゅちょ"), "きゃ:kj.a. しゅ:ɕ.ɯ. ちょ:tɕ.o.");
    assert_eq!(show("ふぁてぃうぇ"), "ふぁ:ɸ.a. てぃ:t.i. うぇ:w.e.");
}

#[test]
fn moraic_nasal_takes_the_next_place() {
    assert_eq!(show("さんぽ"), "さ:s.a. ん:.m. ぽ:p.o.");
    assert_eq!(show("かんだ"), "か:k.a. ん:.n. だ:d.a.");
    assert_eq!(show("りんご"), "り:ɾ.i. ん:.ŋ. ご:g.o.");
    assert_eq!(show("ほん"), "ほ:h.o. ん:.ɴ.");
}

#[test]
fn sokuon_closes_the_mora_before_and_takes_no_note() {
    assert_eq!(show("かった"), "か:k.a.t た:t.a.");
    assert_eq!(show("まっちゃ"), "ま:m.a.t ちゃ:tɕ.a.");
    assert_eq!(show("あっ"), "あ:.a.ʔ");
}

#[test]
fn long_mark_holds_the_vowel() {
    assert_eq!(show("そらー"), "そ:s.o. ら:ɾ.a. ~ー:.a.");
    // across a space it still holds the vowel before
    assert_eq!(show("そら ー"), "そ:s.o. ら:ɾ.a. ~ー:.a.");
}

#[test]
fn japanese_punctuation_ends_phrases() {
    assert_eq!(show("さくら、さくら。"), "さ:s.a. く:k.ɯ. ら:ɾ.a.| さ:s.a. く:k.ɯ. ら:ɾ.a.|");
}

#[test]
fn kanji_are_skipped_and_english_still_reads() {
    assert_eq!(show("桜さくら"), "さ:s.a. く:k.ɯ. ら:ɾ.a.");
    let en = |w: &str, _r: bool| if w == "hi" { vec![Segment { ipa: "h".into(), ..Default::default() }, Segment { ipa: "aɪ".into(), ..Default::default() }] } else { vec![] };
    let s = lyrics_to_syllables("hi さ", &en);
    assert_eq!(s.iter().map(|s| s.text.as_str()).collect::<Vec<_>>(), ["hi", "さ"]);
}

// ---------------------------------------------------------------- speech
use chimera::phonetics::english::EnglishG2P;

/// a spoken text's phonemes, phrases split by "|"
fn spoken(text: &str) -> String {
    EnglishG2P::new().to_phrases(text).iter().map(|p| p.segments.iter().map(|s| s.ipa.as_str()).collect::<Vec<_>>().join(" ")).collect::<Vec<_>>().join(" | ")
}

#[test]
fn kana_text_is_read_as_japanese() {
    assert_eq!(spoken("きのう"), "k i n o ɯ");
    assert_eq!(spoken("さくら、さくら。"), "s a k ɯ ɾ a | s a k ɯ ɾ a");
}

#[test]
fn spoken_vowels_keep_their_voice() {
    // dropping a devoiced vowel was heard as a missing mora (see japanese.rs)
    assert_eq!(spoken("すき"), "s ɯ k i");
    assert_eq!(spoken("です。"), "d e s ɯ");
}

#[test]
fn a_long_mark_lengthens_the_spoken_vowel() {
    let plain = &EnglishG2P::new().to_phrases("か")[0].segments[1];
    let long = &EnglishG2P::new().to_phrases("かー")[0].segments[1];
    assert!(long.dur.unwrap() > plain.dur.unwrap());
}

#[test]
fn spoken_words_match_their_word_ends() {
    for text in ["きょうわ いい てんき。", "すき、です？", "がっこう ー"] {
        for p in EnglishG2P::new().to_phrases(text) {
            let ends = p.segments.iter().filter(|s| s.word_end == Some(true)).count();
            assert_eq!(ends, p.words.len(), "{text}");
        }
    }
    let q = EnglishG2P::new().to_phrases("すき、です？");
    assert_eq!(q.len(), 2);
    assert_eq!(q[1].intonation, chimera::phonetics::Intonation::Question);
}

#[test]
fn english_text_is_still_english() {
    assert!(!spoken("hello").contains('ɯ'));
}

// ---------------------------------------------------------------- pitch accent

/// "H"/"L" per spoken mora (each vowel or ん), compared to the phrase's first word's middle
fn accents(text: &str) -> String {
    let p = &EnglishG2P::new().to_phrases(text)[0];
    p.segments
        .iter()
        .filter_map(|s| s.pitch)
        .map(|x| if x > 1.0 { 'H' } else { 'L' })
        .collect()
}

#[test]
fn words_are_flat_unless_a_fall_is_marked() {
    assert_eq!(accents("はし"), "LH"); // 端
    assert_eq!(accents("は＼し"), "HL"); // 箸
    assert_eq!(accents("はし＼"), "LH"); // 橋: the fall comes on what follows
    assert_eq!(accents("さか＼な"), "LHL");
    assert_eq!(accents("さかな"), "LHH");
    // \ and ꜜ mark it too, and the mark is not part of the word
    assert_eq!(accents("は\\し"), "HL");
    assert_eq!(accents("はꜜし"), "HL");
    assert_eq!(EnglishG2P::new().to_phrases("は＼し")[0].words, ["はし"]);
}

#[test]
fn an_accented_word_steps_the_next_one_down() {
    let p = &EnglishG2P::new().to_phrases("あ＼め さかな")[0];
    let pitch: Vec<f64> = p.segments.iter().filter_map(|s| s.pitch).collect();
    // あ＼め H L, then さかな L H H a step lower
    assert!(pitch[3] < pitch[0] && pitch[3] > pitch[2], "{pitch:?}");
    let flat = &EnglishG2P::new().to_phrases("あめ さかな")[0];
    let flat: Vec<f64> = flat.segments.iter().filter_map(|s| s.pitch).collect();
    assert_eq!(flat[3], flat[1], "a flat word doesn't step down");
}

#[test]
fn lyrics_ignore_accent_marks() {
    assert_eq!(show("は＼し"), "は:h.a. し:ɕ.i.");
}

#[test]
fn the_planner_follows_a_given_pitch() {
    use chimera::control::score::ControlKey;
    use chimera::phonetics::planner::{PlanOptions, plan_utterance};
    let peak = |text: &str| {
        let p = &EnglishG2P::new().to_phrases(text)[0];
        let plan = plan_utterance(&p.segments, &PlanOptions { f0: Some(100.0), ..Default::default() });
        // the pitch target on the first vowel's gesture
        plan.gestures.iter().filter_map(|g| g.targets.get(ControlKey::F0)).next().unwrap()
    };
    assert!(peak("は＼し") > peak("はし"), "{} vs {}", peak("は＼し"), peak("はし"));
}

// ---------------------------------------------------------------- artifacts heard in あめがふる
use chimera::anatomy::human::{BODY_FEMALE, BODY_MALE, HumanBody};
use chimera::phonetics::planner::{PhraseInput, plan_phrases};
use chimera::voice::human_voice::{HumanVoice, SourcePatch};

const FS: f64 = 48000.0;

fn speak(text: &str, body: HumanBody, name: &str, f0: f64) -> (chimera::phonetics::Plan, Vec<f32>) {
    let inputs: Vec<PhraseInput> = EnglishG2P::new().to_phrases(text).into_iter().map(|p| PhraseInput { segments: p.segments, intonation: Some(p.intonation), pause: Some(p.pause), rate: p.rate }).collect();
    let plan = plan_phrases(&inputs, &chimera::phonetics::planner::PlanOptions { body: name.into(), t0: 0.05, f0: Some(f0), ..Default::default() });
    let mut v = HumanVoice::new(FS, body, 2);
    v.set_source(&SourcePatch::f0(f0));
    v.schedule(&plan.gestures);
    let mut out = vec![0.0f32; (FS * (plan.end + 0.3)).round() as usize];
    for chunk in out.chunks_mut(128) {
        v.render(chunk);
    }
    (plan, out)
}

/// How long (ms) the sound around the consonant `ipa` stays 20 dB under the following vowel's
/// middle, in 2 ms RMS frames: the closure a listener hears.
fn closure_ms(text: &str, ipa: &str) -> Vec<(String, f64)> {
    let n = (0.002 * FS) as usize;
    let rms = |x: &[f32], t0: f64, t1: f64| -> Vec<f64> {
        let (a, b) = ((t0 * FS) as usize, ((t1 * FS) as usize).min(x.len()));
        (a..b).step_by(n).map(|i| 10.0 * ((i..(i + n).min(x.len())).map(|j| (x[j] as f64).powi(2)).sum::<f64>() / n as f64 + 1e-20).log10()).collect()
    };
    let mut out = vec![];
    for (body, name, f0) in [(BODY_MALE, "male", 112.0), (BODY_FEMALE, "female", 205.0)] {
        let (plan, x) = speak(text, body, name, f0);
        let segs = &plan.segments;
        for (i, s) in segs.iter().enumerate().filter(|(_, s)| s.ipa == ipa) {
            let Some(v) = segs.get(i + 1) else { continue };
            let mut vowel = rms(&x, v.t0 + 0.3 * (v.t1 - v.t0), v.t0 + 0.7 * (v.t1 - v.t0));
            vowel.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let floor = vowel[vowel.len() / 2] - 20.0;
            let low = rms(&x, s.t0 - 0.04, v.t0 + 0.04).iter().filter(|e| **e < floor).count();
            out.push((format!("{name} {ipa}{} at {:.2} s", v.ipa, s.t0), 2.0 * low as f64));
        }
    }
    out
}

#[test]
fn ru_is_a_tap_not_a_stop() {
    // heard 2026-11-09: the る of あめがふる sounds like ドゥ. What tells a tap from [d] is
    // mostly the closure's length: a tap touches for 10-30 ms, a [d] holds 50 ms or more.
    let r = closure_ms("あめがふる。", "ɾ");
    println!("{r:?}");
    assert_eq!(r.len(), 2, "one per voice");
    assert!(r.iter().all(|(_, ms)| *ms <= 30.0), "{r:?}");
}

/// Morae per second of a spoken sentence at 1× (each vowel, ん and っ is a mora; a ー adds one).
fn morae_per_second(text: &str) -> f64 {
    let inputs: Vec<PhraseInput> = EnglishG2P::new().to_phrases(text).into_iter().map(|p| PhraseInput { segments: p.segments, intonation: Some(p.intonation), pause: Some(p.pause), rate: p.rate }).collect();
    let morae = text.chars().filter(|c| chimera::phonetics::japanese::is_kana(*c) && !"ゃゅょぁぃぅぇぉゎ".contains(*c)).count();
    let plan = plan_phrases(&inputs, &chimera::phonetics::planner::PlanOptions { body: "female".into(), t0: 0.0, f0: Some(200.0), ..Default::default() });
    let speech: f64 = plan.segments.iter().map(|s| s.t1 - s.t0).sum();
    morae as f64 / speech
}

fn english_with_dictionary() -> &'static EnglishG2P {
    static G: std::sync::OnceLock<EnglishG2P> = std::sync::OnceLock::new();
    G.get_or_init(|| {
        use std::io::Read;
        let path = format!("{}/../../public/data/cmudict.txt.gz", env!("CARGO_MANIFEST_DIR"));
        let gz = std::fs::read(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
        let mut text = String::new();
        flate2::read::GzDecoder::new(&gz[..]).read_to_string(&mut text).unwrap();
        let mut g = EnglishG2P::new();
        g.load_dict(&text);
        g
    })
}

/// Seconds from the first sound to the last at 1×, pauses between phrases included.
fn spoken_length(g: &EnglishG2P, text: &str) -> f64 {
    let inputs: Vec<PhraseInput> = g.to_phrases(text).into_iter().map(|p| PhraseInput { segments: p.segments, intonation: Some(p.intonation), pause: Some(p.pause), rate: p.rate }).collect();
    let plan = plan_phrases(&inputs, &chimera::phonetics::planner::PlanOptions { body: "female".into(), t0: 0.0, f0: Some(200.0), ..Default::default() });
    plan.segments.last().unwrap().t1 - plan.segments[0].t0
}

#[test]
fn japanese_takes_as_long_as_the_same_words_in_english() {
    // asked 2026-11-09: at 1× a sentence and its translation take about as long, so the tempo
    // of each language feels the same at any rate
    let pairs = [
        ("It is raining.", "あめが ふって います。"),
        ("Good morning. How are you?", "おはよう ございます。おげんき ですか？"),
        ("The weather is nice today.", "きょうわ いい てんき ですね。"),
        ("I want to drink a cup of tea.", "おちゃを いっぱい のみたいです。"),
        ("Where is the station?", "えきわ どこ ですか？"),
        ("Thank you very much for your help.", "てつだって くれて ほんとうに ありがとう。"),
        ("My younger brother is playing in the park.", "おとうとが こうえんで あそんで います。"),
        ("Please wait a moment.", "すこし まって ください。"),
    ];
    let g = english_with_dictionary();
    let mut log_sum = 0.0;
    for (en, ja) in pairs {
        let (e, j) = (spoken_length(g, en), spoken_length(g, ja));
        println!("{:.2} s / {:.2} s = {:.2}  ({:.1} morae/s)  {en} / {ja}", j, e, j / e, morae_per_second(ja));
        log_sum += (j / e).ln();
    }
    let ratio = (log_sum / pairs.len() as f64).exp();
    println!("geometric mean Japanese / English: {ratio:.3}");
    assert!((0.9..=1.1).contains(&ratio), "Japanese takes {ratio:.2}× as long as English");
}
