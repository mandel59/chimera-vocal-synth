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
