//! Japanese lyrics: kana (hiragana or katakana) → sung syllables, one mora per note, as Japanese
//! songs are set. Kanji need a reading, so lyrics are written in kana.
//!  - A small ゃゅょ or ぁぃぅぇぉ joins the kana before it into one mora (きゃ, ふぁ, てぃ).
//!  - ー holds the previous vowel onto the next note (a melisma, like "-").
//!  - ん is a mora of its own, a nasal sung on its note, placed where the next consonant is made
//!    (m before p b m, n before t d n ts ɾ, ŋ before k g, otherwise ɴ).
//!  - っ takes no note: it closes the mora before it on the next consonant (かった → ka-t | ta).
//!
//! し ち じ are the alveolo-palatal ɕ tɕ dʑ (unrounded, unlike English sh ch j), and う is the
//! unrounded ɯ.

use super::{Segment, Syllable};

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

/// ん before a consonant takes its place of articulation.
fn moraic_nasal(next: Option<&str>) -> &'static str {
    match next {
        Some("p" | "b" | "m" | "ɸ") => "m",
        Some("t" | "d" | "n" | "ts" | "dz" | "ɾ" | "s" | "z") => "n",
        Some("tɕ" | "dʑ" | "ɕ" | "j") => "ɲ",
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
    out
}
