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
    assert_eq!(show("さくら"), "さ:s̪.a. く:k.ɯ. ら:ɾ.a.");
    assert_eq!(show("しちつふひ"), "し:ɕ.i. ち:tɕ.i. つ:t̪s̪.ɯ. ふ:ɸ.ɯ. ひ:ç.i.");
}

#[test]
fn katakana_reads_as_hiragana() {
    assert_eq!(show("サクラ"), show("さくら").replace('さ', "サ").replace('く', "ク").replace('ら', "ラ"));
}

#[test]
fn small_kana_join_the_mora_before() {
    assert_eq!(show("きゃしゅちょ"), "きゃ:c.a. しゅ:ɕ.ɯ. ちょ:tɕ.o.");
    // a palatalized velar or nasal is one palatal consonant; labials take a glide
    assert_eq!(show("ぎょにゃみゅりょ"), "ぎょ:ɟ.o. にゃ:ɲ.a. みゅ:mj.ɯ. りょ:ɾj.o.");
    assert_eq!(show("ふぁてぃうぇ"), "ふぁ:ɸ.a. てぃ:t̪.i. うぇ:w.e.");
}

#[test]
fn moraic_nasal_takes_the_next_place() {
    assert_eq!(show("さんぽ"), "さ:s̪.a. ん:.m. ぽ:p.o.");
    assert_eq!(show("かんだ"), "か:k.a. ん:.n̪. だ:d̪.a.");
    assert_eq!(show("りんご"), "り:ɾ.i. ん:.ŋ. ご:g.o.");
    assert_eq!(show("ほん"), "ほ:h.o. ん:.ɰ̃.");
    // a fricative, glide or vowel after it: no closure, the nasal vowel (sung as spoken)
    assert_eq!(show("あんしん"), "あ:.a. ん:.ɰ̃. し:ɕ.i. ん:.ɰ̃.");
    assert_eq!(show("でんわ"), "で:d̪.e. ん:.ɰ̃. わ:w.a.");
}

#[test]
fn spoken_n_without_a_closure_is_a_nasal_vowel() {
    // asked 2026-11-09: the ん of かもしれません should be a nasal vowel, not a closed [ɴ]
    assert_eq!(spoken("せん。"), "s̪ e ɰ̃");
    assert_eq!(spoken("たんい"), "t̪ a ɰ̃ i");
    assert_eq!(spoken("さんぽ"), "s̪ a m p o");
}

#[test]
fn sokuon_closes_the_mora_before_and_takes_no_note() {
    assert_eq!(show("かった"), "か:k.a.t̪ た:t̪.a.");
    assert_eq!(show("まっちゃ"), "ま:m.a.t ちゃ:tɕ.a.");
    assert_eq!(show("あっ"), "あ:.a.ʔ");
}

#[test]
fn long_mark_holds_the_vowel() {
    assert_eq!(show("そらー"), "そ:s̪.o. ら:ɾ.a. ~ー:.a.");
    // across a space it still holds the vowel before
    assert_eq!(show("そら ー"), "そ:s̪.o. ら:ɾ.a. ~ー:.a.");
}

#[test]
fn japanese_punctuation_ends_phrases() {
    assert_eq!(show("さくら、さくら。"), "さ:s̪.a. く:k.ɯ. ら:ɾ.a.| さ:s̪.a. く:k.ɯ. ら:ɾ.a.|");
}

#[test]
fn kanji_are_skipped_and_english_still_reads() {
    assert_eq!(show("桜さくら"), "さ:s̪.a. く:k.ɯ. ら:ɾ.a.");
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
    assert_eq!(spoken("きのう"), "k i n̪ o ɯ");
    assert_eq!(spoken("さくら、さくら。"), "s̪ a k ɯ ɾ a | s̪ a k ɯ ɾ a");
}

#[test]
fn spoken_vowels_keep_their_voice() {
    // dropping a devoiced vowel was heard as a missing mora (see japanese.rs)
    assert_eq!(spoken("すき"), "s̪ ɯ k i");
    assert_eq!(spoken("です。"), "d̪ e s̪ ɯ");
}

#[test]
fn a_long_mark_is_a_mora_of_its_own() {
    assert_eq!(spoken("かー"), "k a a");
    // and carries its own pitch: 今日 キョ＼ー falls inside the long vowel
    assert_eq!(accents("きょ＼ーわ"), "HLL");
}

#[test]
fn the_pitch_falls_on_a_moraic_nasal() {
    // asked 2026-11-09: in しれませ＼ん the fall must reach the ん, which the planner skipped
    // (pitch targets went on vowels only)
    use chimera::control::score::ControlKey;
    use chimera::phonetics::planner::{PlanOptions, plan_utterance};
    let p = &EnglishG2P::new().to_phrases("ませ＼ん")[0];
    assert_eq!(p.segments.last().unwrap().ipa, "ɰ̃");
    let plan = plan_utterance(&p.segments, &PlanOptions { f0: Some(100.0), ..Default::default() });
    let targets: Vec<f64> = plan.gestures.iter().filter_map(|g| g.targets.get(ControlKey::F0)).collect();
    assert_eq!(targets.len(), 3, "one per mora: {targets:?}");
    assert!(targets[2] < targets[1], "ん lower than せ: {targets:?}");
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

#[test]
fn japanese_alveolars_are_laminal_dental_and_english_ones_are_not() {
    // asked 2026-11-09: Japanese s and t should sit where Japanese makes them, not at the
    // English apical alveolar place
    assert_eq!(spoken("たださなざつ"), "t̪ a d̪ a s̪ a n̪ a z̪ a t̪s̪ ɯ");
    // ん before た closes with it; っ before ち keeps the alveolar closure that ち releases
    assert_eq!(show("ほんと"), "ほ:h.o. ん:.n̪. と:t̪.o.");
    assert_eq!(show("まっちゃ"), "ま:m.a.t ちゃ:tɕ.a.");
    assert!(!spoken("stand tall").contains('\u{32a}'));
}

// ---------------------------------------------------------------- pitch accent

/// "H"/"L" per spoken mora (each vowel or ん): above or below halfway between the phrase's
/// highest and lowest pitch
fn accents(text: &str) -> String {
    let p = &EnglishG2P::new().to_phrases(text)[0];
    let pitch: Vec<f64> = p.segments.iter().filter_map(|s| s.pitch).collect();
    let mid = (pitch.iter().cloned().fold(f64::MIN, f64::max) + pitch.iter().cloned().fold(f64::MAX, f64::min)) / 2.0;
    pitch.iter().map(|x| if *x > mid { 'H' } else { 'L' }).collect()
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
    let inputs: Vec<PhraseInput> = EnglishG2P::new().to_phrases(text).into_iter().map(|p| PhraseInput { segments: p.segments, intonation: Some(p.intonation), pause: Some(p.pause), rate: p.rate, register: p.register, aspiration: p.aspiration }).collect();
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

/// dB of the 2 ms RMS of the first difference (a crude high-pass: a release burst is broadband, a
/// voiced vowel has little energy up there) in 2 ms steps over `t0..t1`
fn hp_peaks(x: &[f32], t0: f64, t1: f64) -> Vec<f64> {
    let n = (0.002 * FS) as usize;
    let (a, b) = ((t0 * FS) as usize, ((t1 * FS) as usize).min(x.len() - 1));
    (a..b).step_by(n).map(|i| {
        let e: f64 = (i..(i + n).min(x.len() - 1)).map(|j| ((x[j + 1] - x[j]) as f64).powi(2)).sum::<f64>() / n as f64;
        10.0 * (e + 1e-20).log10()
    }).collect()
}

/// How far the loudest 2 ms of high-passed sound around the release of the consonant `ipa`
/// (10 ms either side) rises over the following vowel's middle (its median 2 ms).
fn release_excess(text: &str, ipa: &str) -> Vec<(String, f64)> {
    let mut out = vec![];
    for (body, name, f0) in [(BODY_MALE, "male", 112.0), (BODY_FEMALE, "female", 205.0)] {
        let (plan, x) = speak(text, body, name, f0);
        let segs = &plan.segments;
        for (i, s) in segs.iter().enumerate().filter(|(_, s)| s.ipa == ipa) {
            let Some(v) = segs.get(i + 1) else { continue };
            let burst = hp_peaks(&x, s.t1 - 0.01, s.t1 + 0.01).into_iter().fold(f64::MIN, f64::max);
            let mut vowel = hp_peaks(&x, v.t0 + 0.3 * (v.t1 - v.t0), v.t0 + 0.7 * (v.t1 - v.t0));
            vowel.sort_by(|a, b| a.partial_cmp(b).unwrap());
            out.push((format!("{name} {ipa}{} at {:.2} s", v.ipa, s.t1), burst - vowel[vowel.len() / 2]));
        }
    }
    out
}

#[test]
fn ga_is_a_fricative_only_after_a_vowel_in_a_word() {
    assert_eq!(show("がっこう"), "が:g.a.k こ:k.o. う:.ɯ.");
    assert_eq!(show("りんご"), "り:ɾ.i. ん:.ŋ. ご:g.o."); // after ん: [ŋg]
    assert_eq!(show("かがみ"), "か:k.a. が:ɣ.a. み:m.i.");
    assert_eq!(show("あめが ぎんが"), "あ:.a. め:m.e. が:ɣ.a. ぎ:g.i. ん:.ŋ. が:g.a.");
}

#[test]
fn ga_between_vowels_has_no_strong_release_burst() {
    // heard 2026-11-09: the が of あめがふる has a sharp plosive release burst; between vowels
    // Japanese /g/ is a weak stop or the fricative [ɣ] (or the nasal [ŋ], 鼻濁音)
    let r = release_excess("あめがふる。", "ɣ");
    println!("{r:?}");
    assert_eq!(r.len(), 2, "one per voice");
    assert!(r.iter().all(|(_, e)| *e < 10.0), "{r:?}");
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
    let inputs: Vec<PhraseInput> = EnglishG2P::new().to_phrases(text).into_iter().map(|p| PhraseInput { segments: p.segments, intonation: Some(p.intonation), pause: Some(p.pause), rate: p.rate, register: p.register, aspiration: p.aspiration }).collect();
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
    let inputs: Vec<PhraseInput> = g.to_phrases(text).into_iter().map(|p| PhraseInput { segments: p.segments, intonation: Some(p.intonation), pause: Some(p.pause), rate: p.rate, register: p.register, aspiration: p.aspiration }).collect();
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

/// The burst of the first stop `ipa` in a text: dB of the high-passed sound in the 3 ms after
/// the tract really opens (from the voice's telemetry), absolute, so it doesn't move with the
/// vowel after it.
fn burst_db(text: &str, ipa: &str, body: HumanBody, name: &str, f0: f64) -> f64 {
    use chimera::voice::human_voice::VoiceTelemetry;
    let inputs: Vec<PhraseInput> = EnglishG2P::new().to_phrases(text).into_iter().map(|p| PhraseInput { segments: p.segments, intonation: Some(p.intonation), pause: Some(p.pause), rate: p.rate, register: p.register, aspiration: p.aspiration }).collect();
    let plan = plan_phrases(&inputs, &chimera::phonetics::planner::PlanOptions { body: name.into(), t0: 0.05, f0: Some(f0), ..Default::default() });
    let mut v = HumanVoice::new(FS, body, 2);
    v.set_source(&SourcePatch::f0(f0));
    v.schedule(&plan.gestures);
    let mut tele = VoiceTelemetry::default();
    let (mut x, mut buf, mut opens) = (Vec::<f32>::new(), vec![0.0f32; 24], vec![]);
    while (x.len() as f64) / FS < plan.end + 0.1 {
        v.render(&mut buf);
        x.extend_from_slice(&buf);
        v.telemetry(&mut tele);
        opens.push((x.len(), tele.or_areas.iter().chain(tele.ph_areas.iter()).fold(f32::INFINITY, |a, b| a.min(*b))));
    }
    let s = plan.segments.iter().find(|s| s.ipa == ipa).unwrap();
    let c = opens.iter().position(|(n, o)| *n as f64 / FS > s.t0 && *o < 0.01).expect("a closure");
    let r = opens[c..].iter().find(|(_, o)| *o > 0.02).expect("a release").0;
    let n = (0.003 * FS) as usize;
    10.0 * ((r..r + n).map(|j| ((x[j + 1] - x[j]) as f64).powi(2)).sum::<f64>() / n as f64 + 1e-20).log10()
}

#[test]
fn kyo_has_an_audible_burst() {
    // heard 2026-11-09: the き of きょ＼ーわ at the start had a weak burst. As [k] + [j] the
    // release went into the glide's narrowing and all but vanished (male about 50 dB under こ);
    // as one palatal [c] it releases into the vowel. The male velar burst is itself strong (as
    // が's was), so the palatal may stay some dB under it.
    for (body, name, f0) in [(BODY_MALE, "male", 112.0), (BODY_FEMALE, "female", 205.0)] {
        let (kyo, ko) = (burst_db("きょーわ。", "c", body, name, f0), burst_db("こーわ。", "k", body, name, f0));
        println!("{name}: きょ {kyo:.1} dB, こ {ko:.1} dB");
        assert!(kyo > ko - 12.0, "{name}: きょ {kyo:.1} dB against こ {ko:.1} dB");
    }
}

#[test]
fn japanese_is_spoken_in_its_own_register() {
    // asked 2026-11-09: Japanese and English are spoken at different average pitches. Japanese
    // sets the sexes further apart: men a little lower, women a little higher.
    use chimera::control::score::ControlKey;
    let first_f0 = |inputs: &[PhraseInput], body: &str| {
        let plan = plan_phrases(inputs, &chimera::phonetics::planner::PlanOptions { body: body.into(), t0: 0.0, f0: Some(200.0), ..Default::default() });
        plan.gestures.iter().filter_map(|g| g.targets.get(ControlKey::F0)).next().unwrap()
    };
    let ja: Vec<PhraseInput> = EnglishG2P::new().to_phrases("さかな。").into_iter().map(|p| PhraseInput { segments: p.segments, intonation: Some(p.intonation), pause: Some(p.pause), rate: p.rate, register: p.register, aspiration: p.aspiration }).collect();
    let plain: Vec<PhraseInput> = ja.iter().cloned().map(|p| PhraseInput { register: None, ..p }).collect();
    let [m, f] = chimera::phonetics::japanese::REGISTER;
    for (body, st) in [("male", m), ("female", f)] {
        let ratio = first_f0(&ja, body) / first_f0(&plain, body);
        assert!((ratio - 2f64.powf(st / 12.0)).abs() < 1e-9, "{body}: {ratio}");
    }
    assert!(m < 0.0 && f > 0.0);
    // English brings no register
    assert!(EnglishG2P::new().to_phrases("Hello.").iter().all(|p| p.register.is_none()));
}

/// Mean pitch of a text's plan in semitones against the voice's f0 (the pitch targets weighted
/// by how long each holds), its language's register left out.
fn mean_pitch_st(g: &EnglishG2P, text: &str, body: &str) -> f64 {
    use chimera::control::score::ControlKey;
    let inputs: Vec<PhraseInput> = g.to_phrases(text).into_iter().map(|p| PhraseInput { segments: p.segments, intonation: Some(p.intonation), pause: Some(p.pause), rate: p.rate, register: None, aspiration: p.aspiration }).collect();
    let plan = plan_phrases(&inputs, &chimera::phonetics::planner::PlanOptions { body: body.into(), t0: 0.0, f0: Some(100.0), ..Default::default() });
    let (mut sum, mut w) = (0.0, 0.0);
    for g in &plan.gestures {
        if let Some(f) = g.targets.get(ControlKey::F0) {
            let d = g.t1 - g.t0;
            sum += d * 12.0 * (f / 100.0).log2();
            w += d;
        }
    }
    sum / w
}

/// The eight translation pairs, the Japanese as the dictionary reads and accents it
/// (`chimera render text` with `--features jdic`).
const PAIRS: [(&str, &str); 8] = [
    ("It is raining.", "ア＼メガ フ＼ッテ イマ＼ス、"),
    ("Good morning. How are you?", "オハヨーゴザイマ＼ス、オゲ＼ンキデスカ？"),
    ("The weather is nice today.", "キョ＼ーワ イ＼イ テ＼ンキデスネ、"),
    ("I want to drink a cup of tea.", "オチャヲ イッパイ ノミタ＼イデス、"),
    ("Where is the station?", "エ＼キワ ド＼コデスカ？"),
    ("Thank you very much for your help.", "テツダ＼ッテ クレテ ホントーニ＼ アリ＼ガトー、"),
    ("My younger brother is playing in the park.", "オトート＼ガ コーエンデ アソンデ イマ＼ス、"),
    ("Please wait a moment.", "スコ＼シ マ＼ッテ クダサ＼イ、"),
];

#[test]
fn japanese_and_english_intonation_sit_at_the_same_mean_pitch() {
    // Each language's register (REGISTER) is an offset from English; that only holds if the
    // two intonation models average to the same pitch on the same voice. English lifts its
    // stressed syllables, Japanese alternates high and low with downstep.
    let g = english_with_dictionary();
    let n = PAIRS.len() as f64;
    let gaps: Vec<(String, f64)> = ["male", "female"]
        .iter()
        .map(|body| {
            let en = PAIRS.iter().map(|(e, _)| mean_pitch_st(g, e, body)).sum::<f64>() / n;
            let ja = PAIRS.iter().map(|(_, j)| mean_pitch_st(g, j, body)).sum::<f64>() / n;
            println!("{body}: English {en:+.2} st, Japanese {ja:+.2} st");
            (body.to_string(), ja - en)
        })
        .collect();
    assert!(gaps.iter().all(|(_, d)| d.abs() < 0.5), "Japanese minus English: {gaps:?}");
}

/// Voice onset time (ms) of each voiceless stop `ipa` in a text: from the release (the tract
/// opening past 0.05 cm² after its closure) to the folds voicing again, from the voice's
/// telemetry.
fn vot_ms(text: &str, ipa: &str) -> Vec<(String, f64)> {
    use chimera::voice::human_voice::VoiceTelemetry;
    let mut out = vec![];
    for (body, name, f0) in [(BODY_MALE, "male", 112.0), (BODY_FEMALE, "female", 205.0)] {
        let inputs: Vec<PhraseInput> = EnglishG2P::new().to_phrases(text).into_iter().map(|p| PhraseInput { segments: p.segments, intonation: Some(p.intonation), pause: Some(p.pause), rate: p.rate, register: p.register, aspiration: p.aspiration }).collect();
        let plan = plan_phrases(&inputs, &chimera::phonetics::planner::PlanOptions { body: name.into(), t0: 0.05, f0: Some(f0), ..Default::default() });
        let mut v = HumanVoice::new(FS, body, 2);
        v.set_source(&SourcePatch::f0(f0));
        v.schedule(&plan.gestures);
        let mut tele = VoiceTelemetry::default();
        let mut buf = vec![0.0f32; 48];
        let mut track = vec![];
        let mut t = 0.0;
        while t < plan.end + 0.1 {
            v.render(&mut buf);
            t += 48.0 / FS;
            v.telemetry(&mut tele);
            let open = tele.or_areas.iter().chain(tele.ph_areas.iter()).fold(f32::INFINITY, |a, b| a.min(*b));
            track.push((t, open, tele.voicing));
        }
        for s in plan.segments.iter().filter(|s| s.ipa == ipa) {
            let closed = track.iter().position(|(t, o, _)| *t > s.t0 && *o < 0.01);
            let Some(c) = closed else { continue };
            let Some(r) = track[c..].iter().position(|(_, o, _)| *o > 0.05).map(|i| c + i) else { continue };
            let Some(vo) = track[r..].iter().position(|(_, _, v)| *v > 0.5).map(|i| r + i) else { continue };
            out.push((format!("{name} {ipa} at {:.2} s", s.t0), (track[vo].0 - track[r].0) * 1000.0));
        }
    }
    out
}

#[test]
fn japanese_voiceless_stops_are_weakly_aspirated() {
    // asked 2026-11-09 about the か of 降るかも: its folds stayed apart 45 ms after the release,
    // eating half the vowel. Japanese medial voiceless stops are weakly aspirated (about 20-35 ms
    // for /k/, less for /p t/).
    let r = vot_ms("ふるかも。", "k");
    println!("{r:?}");
    assert_eq!(r.len(), 2, "one per voice");
    assert!(r.iter().all(|(_, ms)| (15.0..=35.0).contains(ms)), "{r:?}");
}

#[test]
fn the_japanese_demo_song_has_a_mora_per_note() {
    let song = chimera::phonetics::songs::DEMO_SONGS.iter().find(|s| s.name == "きらきらぼし").expect("the demo song");
    let syls = lyrics_to_syllables(song.lyrics, &word);
    assert_eq!(syls.len(), song.notes.len(), "{:?}", syls.iter().map(|s| s.text.as_str()).collect::<Vec<_>>());
    // a breath at each line's end: six phrases
    assert_eq!(syls.iter().filter(|s| s.phrase_end).count(), 6);
}

#[test]
fn sakura_sakura_has_a_mora_per_note() {
    let song = chimera::phonetics::songs::DEMO_SONGS.iter().find(|s| s.name == "さくらさくら").expect("the demo song");
    let syls = lyrics_to_syllables(song.lyrics, &word);
    assert_eq!(syls.len(), song.notes.len(), "{:?}", syls.iter().map(|s| s.text.as_str()).collect::<Vec<_>>());
    assert_eq!(syls.iter().filter(|s| s.phrase_end).count(), 9, "a breath after each of the nine phrases");
    assert!(!song.lyrics.contains('は'), "the particle is written わ, as sung");
}

use chimera::phonetics::sing::{SongOptions, SungNote, midi_to_hz, plan_song};

/// dB RMS of the last `secs` of the final held note of `lyrics` sung on `notes`.
fn held_end_db(lyrics: &str, notes: &[[f64; 2]], body: HumanBody, name: &str) -> f64 {
    let syl = lyrics_to_syllables(lyrics, &word);
    let notes: Vec<SungNote> = notes.iter().zip(&syl).map(|(n, s)| SungNote { midi: n[0], beats: n[1], syl: s.clone() }).collect();
    let mut v = HumanVoice::new(FS, body, 2);
    v.set_source(&SourcePatch { f0: Some(midi_to_hz(notes[0].midi)), voiced: Some(true), pressure: Some(9.0), tension: Some(0.55), ..Default::default() });
    let plan = plan_song(&notes, &SongOptions { body: name.into(), bpm: 76.0, t0: 0.3, vibrato: 35.0 });
    v.schedule(&plan.gestures);
    let mut out = vec![0.0f32; (FS * (plan.end + 0.3)).round() as usize];
    for chunk in out.chunks_mut(128) {
        v.render(chunk);
    }
    let end = (FS * (plan.end - 0.4)) as usize;
    let x = &out[end - (FS * 1.0) as usize..end];
    10.0 * (x.iter().map(|&s| (s as f64).powi(2)).sum::<f64>() / x.len() as f64).log10()
}

#[test]
fn a_held_final_n_is_sung_out_loud() {
    // asked 2026-11-09: the last ん of さくらさくら (みにゆかん) sounded stuffed and inaudible. It
    // was a closed [ɴ], 18 dB under the same note sung on あ; the nasal vowel ɰ̃ sings 6-8 under.
    let notes = [[65.0, 1.0], [65.0, 1.0], [64.0, 3.0]];
    for (body, name, octave) in [(BODY_MALE, "male", 0.0), (BODY_FEMALE, "female", 12.0)] {
        let notes = notes.map(|[m, b]| [m + octave, b]);
        let n = held_end_db("ゆかん。", &notes, body, name);
        let a = held_end_db("ゆかあ。", &notes, body, name);
        assert!(a - n < 12.0, "{name}: ん {n:.1} dB, あ {a:.1} dB");
    }
}
