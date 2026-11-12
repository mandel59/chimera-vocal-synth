//! Japanese lyrics: kana (hiragana or katakana) → sung syllables, one mora per note, as Japanese
//! songs are set. Kanji need a reading, so lyrics are written in kana.
//!  - A small ゃゅょ or ぁぃぅぇぉ joins the kana before it into one mora (きゃ, ふぁ, てぃ).
//!  - ー holds the previous vowel onto the next note (a melisma, like "-").
//!  - ん is a mora of its own, a nasal sung on its note, closed where the next stop or nasal is
//!    made (m before p b m, n before t d n ts dz ɾ, ɲ before tɕ dʑ, ŋ before k g). Before a vowel,
//!    a glide, a fricative, h, or at the end of a sung word, it uses the open nasal ɰ̃. Speech
//!    also keeps it open within a phrase, but permits a closed [ɴ] at the phrase's end (see
//!    `phrase`). Singing keeps ɰ̃ there: a closed [ɴ] held on a note radiated through the nose
//!    alone and sang 18-20 dB under a vowel, heard as a stuffed nose.
//!  - っ takes no note: it closes the mora before it on the next consonant (かった → ka-t | ta).
//!  - 𛅧 attaches a nasal coda to the preceding vowel on the same note (あ𛅧 → a-n).
//!
//! し ち じ are the alveolo-palatal ɕ tɕ dʑ (unrounded, unlike English sh ch j), う is the
//! unrounded ɯ, and が行 inside a word the nasal ŋ (鼻濁音) sung, the fricative ɣ spoken.
//!
//! Speech ([`to_phrases`]) uses the same morae, timed evenly (Japanese is mora-timed). Vowels
//! aren't devoiced (すき, です): dropping the vowel, the only voiceless vowel the planner can make,
//! was heard as a missing mora.
//!
//! Spoken words get Tokyo pitch accent: low first mora and high after it (平板, the default), or
//! with a fall marked after the accented mora by ＼ (also \ or ꜜ): は＼し 箸, はし＼ 橋, はし 端. A
//! word with a fall lowers the words after it in the phrase (downstep). Lyrics ignore the marks.

use super::Phrase;
use crate::math::js_max;
use super::{Intonation, ReleaseTiming, Segment, Syllable};

/// A kana character, or ー, that starts or continues a run of Japanese lyrics.
pub fn is_kana(c: char) -> bool {
    matches!(c, '\u{3041}'..='\u{3096}' | '\u{30A1}'..='\u{30FA}' | 'ー' | '𛅧')
}

/// Katakana → hiragana (ー and anything else unchanged).
fn hira(c: char) -> char {
    match c {
        '\u{30A1}'..='\u{30F6}' => char::from_u32(c as u32 - 0x60).unwrap_or(c),
        // ヷヸヹヺ: va vi ve vo
        'ヷ' => 'ゔ',
        _ => c,
    }
}

/// The onset consonants and the vowel of a full-size kana.
fn base(c: char) -> Option<(&'static [&'static str], &'static str)> {
    const N: &[&str] = &[];
    Some(match c {
        'あ' | 'ぁ' => (N, "a"),
        'い' | 'ぃ' | 'ゐ' => (N, "i"),
        'う' | 'ぅ' => (N, "ɯ"),
        'え' | 'ぇ' | 'ゑ' => (N, "e"),
        'お' | 'ぉ' | 'を' => (N, "o"),
        'か' | 'ゕ' => (&["k"], "a"),
        'き' => (&["k"], "i"),
        'く' => (&["k"], "ɯ"),
        'け' | 'ゖ' => (&["k"], "e"),
        'こ' => (&["k"], "o"),
        'が' => (&["g"], "a"),
        'ぎ' => (&["g"], "i"),
        'ぐ' => (&["g"], "ɯ"),
        'げ' => (&["g"], "e"),
        'ご' => (&["g"], "o"),
        'さ' => (&["s"], "a"),
        'し' => (&["ɕ"], "i"),
        'す' => (&["s"], "ɯ"),
        'せ' => (&["s"], "e"),
        'そ' => (&["s"], "o"),
        'ざ' => (&["z"], "a"),
        'じ' | 'ぢ' => (&["dʑ"], "i"),
        'ず' => (&["z"], "ɯ"),
        'ぜ' => (&["z"], "e"),
        'ぞ' => (&["z"], "o"),
        'た' => (&["t"], "a"),
        'ち' => (&["tɕ"], "i"),
        'つ' => (&["ts"], "ɯ"),
        'て' => (&["t"], "e"),
        'と' => (&["t"], "o"),
        'だ' => (&["d"], "a"),
        'づ' => (&["dz"], "ɯ"),
        'で' => (&["d"], "e"),
        'ど' => (&["d"], "o"),
        'な' => (&["n"], "a"),
        'に' => (&["n"], "i"),
        'ぬ' => (&["n"], "ɯ"),
        'ね' => (&["n"], "e"),
        'の' => (&["n"], "o"),
        'は' => (&["h"], "a"),
        'ひ' => (&["ç"], "i"),
        'ふ' => (&["ɸ"], "ɯ"),
        'へ' => (&["h"], "e"),
        'ほ' => (&["h"], "o"),
        'ば' => (&["b"], "a"),
        'び' => (&["b"], "i"),
        'ぶ' => (&["b"], "ɯ"),
        'べ' => (&["b"], "e"),
        'ぼ' => (&["b"], "o"),
        'ぱ' => (&["p"], "a"),
        'ぴ' => (&["p"], "i"),
        'ぷ' => (&["p"], "ɯ"),
        'ぺ' => (&["p"], "e"),
        'ぽ' => (&["p"], "o"),
        'ま' => (&["m"], "a"),
        'み' => (&["m"], "i"),
        'む' => (&["m"], "ɯ"),
        'め' => (&["m"], "e"),
        'も' => (&["m"], "o"),
        'や' | 'ゃ' => (&["j"], "a"),
        'ゆ' | 'ゅ' => (&["j"], "ɯ"),
        'よ' | 'ょ' => (&["j"], "o"),
        'ら' => (&["ɾ"], "a"),
        'り' => (&["ɾ"], "i"),
        'る' => (&["ɾ"], "ɯ"),
        'れ' => (&["ɾ"], "e"),
        'ろ' => (&["ɾ"], "o"),
        'わ' | 'ゎ' => (&["w"], "a"),
        'ゔ' => (&["v"], "ɯ"),
        _ => return None,
    })
}

fn is_small_y(c: char) -> bool {
    matches!(c, 'ゃ' | 'ゅ' | 'ょ')
}
fn is_small_vowel(c: char) -> bool {
    matches!(c, 'ぁ' | 'ぃ' | 'ぅ' | 'ぇ' | 'ぉ' | 'ゎ')
}

enum Mora {
    /// onset, vowel, the kana it was written with
    Sung(Vec<&'static str>, &'static str, String),
    N(String),
    AttachedN,
    Geminate,
    Hold(String),
}

fn morae(word: &str) -> Vec<Mora> {
    let cs: Vec<(char, char)> = word.chars().map(|c| (c, hira(c))).collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < cs.len() {
        let (raw, c) = cs[i];
        i += 1;
        match c {
            'ー' => out.push(Mora::Hold(raw.to_string())),
            'ん' => out.push(Mora::N(raw.to_string())),
            '𛅧' => out.push(Mora::AttachedN),
            'っ' => out.push(Mora::Geminate),
            _ => {
                let Some((onset, vowel)) = base(c) else { continue };
                let mut onset: Vec<&'static str> = onset.to_vec();
                let mut vowel = vowel;
                let mut text = raw.to_string();
                // a small kana after a full-size one joins it
                if i < cs.len() && !is_small_y(c) && !is_small_vowel(c) {
                    let (raw2, s) = cs[i];
                    if is_small_y(s) {
                        let (_, v) = base(s).unwrap_or((&[], "a"));
                        vowel = v;
                        // しゃ ちゃ じゃ ひゃ are palatal already. A palatalized velar or nasal is
                        // one consonant made at the palate, released straight into the vowel: きゃ
                        // [ca], ぎゃ [ɟa], にゃ [ɲa] (as [k] + [j] its burst all but vanished,
                        // released into the glide's narrowing). Others take a [j]: みゃ [mja].
                        match onset.last().copied() {
                            Some("ɕ" | "tɕ" | "dʑ" | "ç" | "j") => {}
                            Some("k") => *onset.last_mut().unwrap() = "c",
                            Some("g") => *onset.last_mut().unwrap() = "ɟ",
                            Some("n") => *onset.last_mut().unwrap() = "ɲ",
                            _ => onset.push("j"),
                        }
                        text.push(raw2);
                        i += 1;
                    } else if is_small_vowel(s) {
                        let (o2, v) = base(s).unwrap_or((&[], "a"));
                        if s == 'ゎ' {
                            onset.extend_from_slice(o2);
                        } else if onset.is_empty() && vowel == "ɯ" {
                            onset.push("w"); // うぃ うぇ うぉ
                        } else if onset.is_empty() && vowel == "i" && s == 'ぇ' {
                            onset.push("j"); // いぇ
                        }
                        vowel = v;
                        text.push(raw2);
                        i += 1;
                    }
                }
                out.push(Mora::Sung(onset, vowel, text));
            }
        }
    }
    out
}

fn seg(ipa: &str) -> Segment {
    Segment { ipa: ipa.to_string(), stress: Some(1), ..Default::default() }
}

/// The first consonant of the next sung mora, if any.
fn next_consonant(ms: &[Mora], from: usize) -> Option<&'static str> {
    ms[from..].iter().find_map(|m| match m {
        Mora::Sung(o, _, _) => Some(o.first().copied()),
        Mora::N(_) | Mora::Hold(_) => Some(None),
        Mora::Geminate | Mora::AttachedN => None,
    })?
}

/// ん before a stop, affricate or nasal closes where it does; otherwise start with an open ɰ̃.
/// Speech chooses its phrase-final closure later, after all words have been assembled.
fn moraic_nasal(next: Option<&str>) -> &'static str {
    match next {
        Some("p" | "b" | "m") => "m",
        Some("t" | "d" | "n" | "ts" | "dz" | "ɾ") => "n",
        Some("tɕ" | "dʑ" | "c" | "ɟ" | "ɲ") => "ɲ",
        Some("k" | "g") => "ŋ",
        _ => "ɰ̃",
    }
}

/// っ closes on the next consonant: an affricate's stop, a fricative held, a glottal stop before
/// a vowel.
fn geminate(next: Option<&'static str>) -> &'static str {
    match next {
        Some("tɕ" | "ts") => "t",
        Some("dʑ" | "dz") => "d",
        Some(c @ ("p" | "b" | "t" | "d" | "k" | "g" | "c" | "ɟ" | "s" | "z" | "ɕ" | "ɸ" | "ç" | "h" | "m" | "n")) => c,
        _ => "ʔ",
    }
}

/// The laminal denti-alveolar of an alveolar.
fn dental(ipa: &str) -> Option<&'static str> {
    Some(match ipa {
        "t" => "t̪",
        "d" => "d̪",
        "n" => "n̪",
        "s" => "s̪",
        "z" => "z̪",
        "ts" => "t̪s̪",
        "dz" => "d̪z̪",
        _ => return None,
    })
}

/// Explicit attached nasals must follow a vowel-bearing kana in the same word.
/// No implicit attachment across whitespace, punctuation, a nasal or a geminate closure.
pub fn validate_lyrics(text: &str) -> Result<(), String> {
    let mut previous = None;
    let mut vowel = false;
    for c in text.chars() {
        if c == '𛅧' {
            if !vowel { return Err("𛅧 needs a preceding vowel-bearing kana in the same word".into()); }
        }
        if c == 'っ' || c == 'ッ' {
            if previous == Some('𛅧') { return Err("𛅧 cannot share a syllable with a geminate closure".into()); }
        }
        vowel = base(hira(c)).is_some() || (c == 'ー' && vowel);
        previous = Some(c);
    }
    Ok(())
}

/// One run of kana → sung syllables, one per note. `prev` is the syllable sung before the run (for
/// a ー or っ at its start).
pub fn kana_to_syllables(word: &str, prev: Option<&Syllable>) -> Vec<Syllable> {
    syllables(word, prev, true)
}

/// Kana → syllables, sung or spoken: they differ only in が行 inside a word (see below).
fn syllables(word: &str, prev: Option<&Syllable>, sung: bool) -> Vec<Syllable> {
    let ms = morae(word);
    let mut out: Vec<Syllable> = Vec::new();
    for (k, m) in ms.iter().enumerate() {
        match m {
            Mora::Sung(onset, vowel, text) => {
                out.push(Syllable { text: text.clone(), onset: onset.iter().map(|c| seg(c)).collect(), nucleus: seg(vowel), coda: vec![], melisma: None, word_end: false, phrase_end: false, ..Default::default() });
            }
            Mora::N(text) => {
                let n = moraic_nasal(next_consonant(&ms, k + 1));
                out.push(Syllable { text: text.clone(), onset: vec![], nucleus: seg(n), coda: vec![], melisma: None, word_end: false, phrase_end: false, ..Default::default() });
            }
            Mora::AttachedN => {
                if sung {
                    if let Some(p) = out.last_mut() {
                        p.coda.push(seg(moraic_nasal(next_consonant(&ms, k + 1))));
                        // Display ordinary kana; the supplementary character is input notation.
                        p.text.push('ん');
                    }
                }
            }
            Mora::Hold(text) => {
                if let Some(p) = out.last().or(prev) {
                    let nucleus = p.nucleus.clone();
                    out.push(Syllable { text: text.clone(), onset: vec![], nucleus, coda: vec![], melisma: Some(true), word_end: false, phrase_end: false, ..Default::default() });
                }
            }
            Mora::Geminate => {
                if let Some(p) = out.last_mut() {
                    p.coda.push(seg(geminate(next_consonant(&ms, k + 1))));
                }
            }
        }
    }
    // た だ な さ ざ つ (and ん, っ before them) are laminal denti-alveolar, not the English apical
    // alveolars: set last, so the rules above match the plain symbols. っ before ち or ぢ keeps
    // the alveolar closure, which the alveolo-palatal releases.
    for k in 0..out.len() {
        let before_alvpal = out.get(k + 1).and_then(|n| n.onset.first()).is_some_and(|c| c.ipa == "tɕ" || c.ipa == "dʑ");
        let s = &mut out[k];
        let coda = s.coda.iter_mut().filter(|_| !before_alvpal);
        for c in s.onset.iter_mut().chain(coda).chain(std::iter::once(&mut s.nucleus)) {
            if let Some(d) = dental(&c.ipa) {
                c.ipa = d.into();
            }
        }
    }
    // が行 after a vowel inside a word is not a stop: a [g] released between vowels bursts (heard
    // in あめがふる). Spoken, it is the fricative [ɣ]. Sung, it is the nasal [ŋ] (鼻濁音), as
    // 唱歌 and choral diction have it: a sung [ɣ] was weak on a short note (かげさして's げ), and a
    // nasal keeps the voice and the pitch going; ぎゃ ぎゅ ぎょ, a palatal [ɟ], become the palatal
    // nasal [ɲ]. Word-initially, and after ん or っ, it stays [g].
    for k in 1..out.len() {
        let after_vowel = out[k - 1].coda.is_empty() && matches!(out[k - 1].nucleus.ipa.as_str(), "a" | "i" | "ɯ" | "e" | "o");
        if let Some(c) = out[k].onset.first_mut().filter(|_| after_vowel) {
            match (c.ipa.as_str(), sung) {
                ("g", true) => c.ipa = "ŋ".into(),
                ("ɟ", true) => c.ipa = "ɲ".into(),
                ("g", false) => c.ipa = "ɣ".into(),
                _ => {}
            }
        }
    }
    if sung {
        for s in &mut out {
            // A 50 ms vowel can be entirely covered by stop release and the following
            // fricative's anticipatory glottal opening. Reserve an audible voiced interval
            // within the note; the singer still keeps every beat and the consonant floor.
            if matches!(s.nucleus.ipa.as_str(), "a" | "i" | "ɯ" | "e" | "o") {
                s.min_vowel = Some(0.1);
            }
            // Japanese does not use the English strongly aspirated stop series.
            for c in s.onset.iter_mut().chain(s.coda.iter_mut()) {
                if matches!(c.ipa.as_str(), "p" | "t" | "t̪" | "k" | "c" | "tɕ" | "t̪s̪") {
                    c.aspirated = Some(false);
                }
            }
        }
    }
    out
}

/// Whether text has kana in it, so it is read as Japanese.
pub fn has_kana(text: &str) -> bool {
    text.chars().any(|c| is_kana(c) && c != 'ー')
}

/// Japanese tempo against the planner's (English) timing: every duration below, and the
/// consonants' from the planner's rules, is divided by it (the phrase's `rate`). Set so that a
/// sentence and its English translation take about as long at 1× (tested on eight pairs: the
/// geometric mean of the ratio is 1.0), which comes to about nine morae a second. The duration
/// ratios below were tuned slow; scaling them all keeps consonants short with the vowels, which
/// sounded clearer than shortening the vowels alone.
pub const TEMPO: f64 = 1.95;
/// Japanese speaking pitch against the voice's (English) pitch, in semitones for a male and a
/// female speaker: Japanese sets the sexes 1-3 semitones further apart than American English,
/// the men somewhat lower and the women somewhat higher (Liberman, Language Log 2007, CallHome
/// telephone speech; female speakers higher in Japanese than in English also in Ohara 1999 and
/// Muta 1995; the men's results vary between studies). Small samples, so a modest split.
pub const REGISTER: [f64; 2] = [-1.0, 1.5];
/// Japanese voiceless stops are weakly aspirated, less than English ones: this scales how fast
/// the folds close after the release (the planner keeps them apart through it). At 1 a medial か
/// had about 52 ms from release to voicing, more than half its 87 ms vowel; at 0.3, 28 ms.
pub const ASPIRATION: f64 = 0.3;
/// a spoken mora's vowel (s, before [`TEMPO`])
const MORA_VOWEL: f64 = 0.17;
/// a spoken ん (s)
const MORA_N: f64 = 0.15;
/// a っ closure (s): a mora's worth of silence
const MORA_GEMINATE: f64 = 0.09;
/// pitch of a high and a low mora, relative to the phrase's base: about four semitones apart
const PITCH_HIGH: f64 = 1.12;
const PITCH_LOW: f64 = 0.9;
/// Japanese intonation's mean against English's on the same voice, in semitones: highs and
/// lows with downstep average about 3 semitones under English's stress accents (measured on
/// eight translation pairs, see the tests), so every Japanese pitch is raised by this before
/// [`REGISTER`] sets the language apart.
const PITCH_CENTER_ST: f64 = 3.0;
/// how far each accented word lowers the ones after it in the phrase, and the floor
const DOWNSTEP: f64 = 0.88;
const DOWNSTEP_FLOOR: f64 = 0.7;

/// A pitch-accent mark: the pitch falls after the mora before it.
pub fn is_accent_mark(c: char) -> bool {
    matches!(c, '＼' | '\\' | 'ꜜ')
}

/// Tokyo accent: the pitch of mora `m` (0-based) of a word of `n` morae whose fall comes after
/// mora `nucleus` (1-based; 0 for none, 平板).
fn accent_pitch(m: usize, n: usize, nucleus: usize) -> f64 {
    let high = match nucleus {
        1 => m == 0,
        0 => m > 0 || n == 1,
        k => m > 0 && m < k,
    };
    if high { PITCH_HIGH } else { PITCH_LOW }
}

fn is_speech_punct(c: char) -> bool {
    matches!(c, '、' | '。' | '！' | '？' | ',' | '.' | '!' | '?' | '\n')
}

/// Kana text → spoken phrases, split at punctuation. Anything but kana (and ー) is skipped.
pub fn to_phrases(text: &str) -> Vec<Phrase> {
    let mut phrases = Vec::new();
    let mut part = String::new();
    for c in text.chars().chain(std::iter::once('\n')) {
        if !is_speech_punct(c) {
            part.push(c);
            continue;
        }
        let words: Vec<String> = part.split(|c: char| !is_kana(c) && !is_accent_mark(c)).filter(|w| !w.is_empty()).map(str::to_string).collect();
        part.clear();
        if let Some(p) = phrase(&words, c) {
            phrases.push(p);
        }
    }
    // the text's end is no pause of its own: the last phrase keeps the one of its punctuation
    phrases
}

fn phrase(words: &[String], delim: char) -> Option<Phrase> {
    // the phrase's morae, each with the index of the word it ends and its pitch
    let mut syls: Vec<(Syllable, Option<usize>, f64)> = Vec::new();
    let mut said = Vec::new();
    let mut step = 1.0;
    for w in words {
        let mut s = syllables(w, syls.last().map(|x| &x.0), false);
        if s.is_empty() {
            continue;
        }
        // a lone ー only lengthens the word before: no word of its own
        if s.iter().all(|x| x.melisma == Some(true)) {
            let p = syls.last().map_or(1.0, |x| x.2);
            syls.extend(s.drain(..).map(|x| (x, None, p)));
            continue;
        }
        // the fall comes after the morae before the first mark
        let nucleus = w.find(is_accent_mark).map_or(0, |i| syllables(&w[..i], None, false).len());
        let n = s.len();
        let center = crate::math::pow(2.0, PITCH_CENTER_ST / 12.0);
        syls.extend(s.drain(..).enumerate().map(|(i, x)| (x, (i + 1 == n).then_some(said.len()), center * step * accent_pitch(i, n, nucleus))));
        said.push(w.chars().filter(|c| !is_accent_mark(*c)).collect::<String>());
        if nucleus > 0 {
            step = js_max(DOWNSTEP_FLOOR, step * DOWNSTEP);
        }
    }
    if syls.is_empty() {
        return None;
    }
    // A spoken phrase may end in a closed nasal. Use [ɴ] as a simple default articulation,
    // not a claim that every Japanese final /N/ is uvular: its closure varies with the speaker
    // and preceding vowel (Maekawa, doi:10.1017/S0025100321000050). Do this after joining words
    // so a dictionary's accent-phrase boundary alone cannot close the ん of e.g. ほん を.
    // An explicit long-vowel continuation keeps the open nasal, as singing does.
    if let Some((s, _, _)) = syls.last_mut() {
        if s.nucleus.ipa == "ɰ̃" && s.melisma != Some(true) {
            s.nucleus.ipa = "ɴ".into();
            s.nucleus.release_timing = Some(ReleaseTiming::AfterBreath);
        }
    }
    let mut segs: Vec<Segment> = Vec::new();
    for (s, word_end, pitch) in &syls {
        if s.melisma == Some(true) {
            // ー is a mora of its own: the vowel goes on for a mora's vowel, at its own pitch
            // (キョ＼ー falls inside), so a long vowel is twice its short one. At 0.1 s it was 1.6
            // times, and ソード's oo was heard short.
            segs.push(Segment { ipa: s.nucleus.ipa.clone(), dur: Some(MORA_VOWEL), stress: Some(1), pitch: Some(*pitch), release_timing: s.nucleus.release_timing, ..Default::default() });
        } else {
            let vowel = matches!(s.nucleus.ipa.as_str(), "a" | "i" | "ɯ" | "e" | "o");
            for c in &s.onset {
                segs.push(Segment { ipa: c.ipa.clone(), stress: Some(0), ..Default::default() });
            }
            segs.push(Segment { ipa: s.nucleus.ipa.clone(), dur: Some(if vowel { MORA_VOWEL } else { MORA_N }), stress: Some(1), pitch: Some(*pitch), release_timing: s.nucleus.release_timing, ..Default::default() });
            for c in &s.coda {
                segs.push(Segment { ipa: c.ipa.clone(), dur: Some(MORA_GEMINATE), stress: Some(0), ..Default::default() });
            }
        }
        if word_end.is_some()
            && let Some(last) = segs.last_mut()
        {
            last.word_end = Some(true);
        }
    }
    let pause = match delim {
        '。' | '！' | '？' | '.' | '!' | '?' => 0.35,
        '、' | ',' => 0.18,
        _ => 0.3,
    };
    let intonation = if matches!(delim, '？' | '?') { Intonation::Question } else { Intonation::Statement };
    Some(Phrase { segments: segs, intonation, pause, words: said, rate: Some(TEMPO), register: Some(REGISTER), aspiration: Some(ASPIRATION) })
}
