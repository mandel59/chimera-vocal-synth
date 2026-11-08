//! Japanese lyrics: kana (hiragana or katakana) → sung syllables, one mora per note, as Japanese
//! songs are set. Kanji need a reading, so lyrics are written in kana.
//!  - A small ゃゅょ or ぁぃぅぇぉ joins the kana before it into one mora (きゃ, ふぁ, てぃ).
//!  - ー holds the previous vowel onto the next note (a melisma, like "-").
//!  - ん is a mora of its own, a nasal sung on its note, closed where the next stop or nasal is
//!    made (m before p b m, n before t d n ts dz ɾ, ɲ before tɕ dʑ, ŋ before k g). Before a vowel,
//!    a glide, a fricative, h, or at the end, it doesn't close: spoken, it is the nasal vowel ɰ̃
//!    (本, 単位, 安心, 電話); sung, the closed hum ɴ, which holds a note.
//!  - っ takes no note: it closes the mora before it on the next consonant (かった → ka-t | ta).
//!
//! し ち じ are the alveolo-palatal ɕ tɕ dʑ (unrounded, unlike English sh ch j), う is the
//! unrounded ɯ, and が行 inside a word the fricative ɣ.
//!
//! Speech ([`to_phrases`]) uses the same morae, timed evenly (Japanese is mora-timed). Vowels
//! aren't devoiced (すき, です): dropping the vowel, the only voiceless vowel the planner can make,
//! was heard as a missing mora.
//!
//! Spoken words get Tokyo pitch accent: low first mora and high after it (平板, the default), or
//! with a fall marked after the accented mora by ＼ (also \ or ꜜ): は＼し 箸, はし＼ 橋, はし 端. A
//! word with a fall lowers the words after it in the phrase (downstep). Lyrics ignore the marks.

use super::english::Phrase;
use crate::math::js_max;
use super::{Intonation, Segment, Syllable};

/// A kana character, or ー, that starts or continues a run of Japanese lyrics.
pub fn is_kana(c: char) -> bool {
    matches!(c, '\u{3041}'..='\u{3096}' | '\u{30A1}'..='\u{30FA}' | 'ー')
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
                        // しゃ ちゃ じゃ ひゃ are palatal already; きゃ → kja, いゃ → ja
                        if !matches!(onset.last(), Some(&("ɕ" | "tɕ" | "dʑ" | "ç" | "j"))) {
                            onset.push("j");
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
        Mora::Geminate => None,
    })?
}

/// ん before a stop, affricate or nasal closes where it does; anywhere else it is ɴ here (the
/// sung hum; speech makes it the nasal vowel ɰ̃).
fn moraic_nasal(next: Option<&str>) -> &'static str {
    match next {
        Some("p" | "b" | "m") => "m",
        Some("t" | "d" | "n" | "ts" | "dz" | "ɾ") => "n",
        Some("tɕ" | "dʑ") => "ɲ",
        Some("k" | "g") => "ŋ",
        _ => "ɴ",
    }
}

/// っ closes on the next consonant: an affricate's stop, a fricative held, a glottal stop before
/// a vowel.
fn geminate(next: Option<&'static str>) -> &'static str {
    match next {
        Some("tɕ" | "ts") => "t",
        Some("dʑ" | "dz") => "d",
        Some(c @ ("p" | "b" | "t" | "d" | "k" | "g" | "s" | "z" | "ɕ" | "ɸ" | "ç" | "h" | "m" | "n")) => c,
        _ => "ʔ",
    }
}

/// One run of kana → syllables, one per note. `prev` is the syllable sung before the run (for a
/// ー or っ at its start).
pub fn kana_to_syllables(word: &str, prev: Option<&Syllable>) -> Vec<Syllable> {
    let ms = morae(word);
    let mut out: Vec<Syllable> = Vec::new();
    for (k, m) in ms.iter().enumerate() {
        match m {
            Mora::Sung(onset, vowel, text) => {
                out.push(Syllable { text: text.clone(), onset: onset.iter().map(|c| seg(c)).collect(), nucleus: seg(vowel), coda: vec![], melisma: None, word_end: false, phrase_end: false });
            }
            Mora::N(text) => {
                let n = moraic_nasal(next_consonant(&ms, k + 1));
                out.push(Syllable { text: text.clone(), onset: vec![], nucleus: seg(n), coda: vec![], melisma: None, word_end: false, phrase_end: false });
            }
            Mora::Hold(text) => {
                if let Some(p) = out.last().or(prev) {
                    let nucleus = p.nucleus.clone();
                    out.push(Syllable { text: text.clone(), onset: vec![], nucleus, coda: vec![], melisma: Some(true), word_end: false, phrase_end: false });
                }
            }
            Mora::Geminate => {
                if let Some(p) = out.last_mut() {
                    p.coda.push(seg(geminate(next_consonant(&ms, k + 1))));
                }
            }
        }
    }
    // が行 after a vowel inside a word is a fricative [ɣ], not a stop: a [g] released between
    // vowels bursts (heard in あめがふる). Word-initially, and after ん or っ, it stays [g].
    for k in 1..out.len() {
        let after_vowel = out[k - 1].coda.is_empty() && matches!(out[k - 1].nucleus.ipa.as_str(), "a" | "i" | "ɯ" | "e" | "o");
        if let Some(c) = out[k].onset.first_mut().filter(|c| c.ipa == "g" && after_vowel) {
            c.ipa = "ɣ".into();
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
/// a spoken mora's vowel (s, before [`TEMPO`])
const MORA_VOWEL: f64 = 0.17;
/// a spoken ん (s)
const MORA_N: f64 = 0.15;
/// what ー adds to the vowel before it (s)
const MORA_LONG: f64 = 0.1;
/// a っ closure (s): a mora's worth of silence
const MORA_GEMINATE: f64 = 0.09;
/// pitch of a high and a low mora, relative to the phrase's base: about four semitones apart
const PITCH_HIGH: f64 = 1.12;
const PITCH_LOW: f64 = 0.9;
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
        let mut s = kana_to_syllables(w, syls.last().map(|x| &x.0));
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
        let nucleus = w.find(is_accent_mark).map_or(0, |i| kana_to_syllables(&w[..i], None).len());
        let n = s.len();
        syls.extend(s.drain(..).enumerate().map(|(i, x)| (x, (i + 1 == n).then_some(said.len()), step * accent_pitch(i, n, nucleus))));
        said.push(w.chars().filter(|c| !is_accent_mark(*c)).collect::<String>());
        if nucleus > 0 {
            step = js_max(DOWNSTEP_FLOOR, step * DOWNSTEP);
        }
    }
    if syls.is_empty() {
        return None;
    }
    let mut segs: Vec<Segment> = Vec::new();
    for (s, word_end, pitch) in &syls {
        if s.melisma == Some(true) {
            if let Some(v) = segs.iter_mut().rev().find(|x| x.stress == Some(1)) {
                v.dur = Some(v.dur.unwrap_or(MORA_VOWEL) + MORA_LONG);
            }
        } else {
            let vowel = matches!(s.nucleus.ipa.as_str(), "a" | "i" | "ɯ" | "e" | "o");
            for c in &s.onset {
                segs.push(Segment { ipa: c.ipa.clone(), stress: Some(0), ..Default::default() });
            }
            // a spoken ん that doesn't close is a nasal vowel, not the sung hum
            let ipa = if s.nucleus.ipa == "ɴ" { "ɰ̃".to_string() } else { s.nucleus.ipa.clone() };
            segs.push(Segment { ipa, dur: Some(if vowel { MORA_VOWEL } else { MORA_N }), stress: Some(1), pitch: Some(*pitch), ..Default::default() });
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
    Some(Phrase { segments: segs, intonation, pause, words: said, rate: Some(TEMPO) })
}
