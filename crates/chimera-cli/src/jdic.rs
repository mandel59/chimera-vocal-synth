//! Japanese text with kanji → the kana markup the engine speaks, with each accent phrase's pitch
//! accent: OpenJTalk's front end (jpreprocess, a Rust rewrite) and its dictionary (naist-jdic,
//! bundled at build time). Built with `--features jdic`.
//!
//! 雨が降る → "あ＼めが ふる": each accent phrase becomes a space-separated word, its fall marked
//! with ＼ after the accented mora (flat phrases have none), and 、 and ？ end breath groups.
//! The engine does the rest (`chimera::phonetics::japanese`): morae, Tokyo pitch, downstep.

use jpreprocess::kind::JPreprocessDictionaryKind;
use jpreprocess::{DefaultTokenizer, JPreprocess, SystemDictionaryConfig};
use std::sync::OnceLock;

fn jpreprocess() -> Result<&'static JPreprocess<DefaultTokenizer>, String> {
    static J: OnceLock<JPreprocess<DefaultTokenizer>> = OnceLock::new();
    if let Some(j) = J.get() {
        return Ok(j);
    }
    let system = SystemDictionaryConfig::Bundled(JPreprocessDictionaryKind::NaistJdic).load().map_err(|e| format!("naist-jdic: {e}"))?;
    Ok(J.get_or_init(|| JPreprocess::with_dictionaries(system, None)))
}

/// Whether the text is Japanese the dictionary should read: kanji or kana, and no accent marks
/// typed by hand (those are kept as written).
pub fn wants(text: &str) -> bool {
    let japanese = text.chars().any(|c| matches!(c, '\u{3041}'..='\u{30FA}' | '\u{3400}'..='\u{9FFF}' | '々'));
    japanese && !text.chars().any(chimera::phonetics::japanese::is_accent_mark)
}

/// Text → kana markup with accent marks (see the module docs).
pub fn annotate(text: &str) -> Result<String, String> {
    let j = jpreprocess()?;
    let mut njd = j.text_to_njd(text).map_err(|e| e.to_string())?;
    njd.preprocess();
    let mut out = String::new();
    // the accent phrase being built: its kana, accent (0: flat) and morae so far
    let mut phrase = String::new();
    let (mut accent, mut morae) = (0usize, 0usize);
    let flush = |out: &mut String, phrase: &mut String| {
        if !phrase.is_empty() {
            if !out.is_empty() && !out.ends_with(['、', '？']) {
                out.push(' ');
            }
            out.push_str(phrase);
            phrase.clear();
        }
    };
    for node in &njd.nodes {
        let pron = node.get_pron();
        if pron.is_touten() || pron.is_question() {
            flush(&mut out, &mut phrase);
            out.push(if pron.is_question() { '？' } else { '、' });
            continue;
        }
        if !matches!(node.get_chain_flag(), Some(true)) {
            flush(&mut out, &mut phrase);
            accent = pron.accent();
            morae = 0;
        }
        for m in pron.moras() {
            // a devoiced mora is marked ’: the engine keeps its voice (see japanese.rs)
            phrase.push_str(m.to_string().trim_end_matches('’'));
            morae += 1;
            if morae == accent {
                phrase.push('＼');
            }
        }
    }
    flush(&mut out, &mut phrase);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn homophones_get_their_own_accents() {
        // 箸 HL, 橋 LH with the particle low, 端 LH with the particle high
        assert_eq!(annotate("箸で橋を渡る。端に寄る。").unwrap(), "ハ＼シデ ハシ＼ヲ ワタル、ハシニ ヨル、");
    }

    #[test]
    fn particles_join_their_accent_phrase_and_questions_stay() {
        assert_eq!(annotate("雨が降る。").unwrap(), "ア＼メガ フ＼ル、");
        assert_eq!(annotate("お元気ですか？").unwrap(), "オゲ＼ンキデスカ？");
    }

    #[test]
    fn hand_marked_text_is_kept() {
        assert!(wants("雨が降る"));
        assert!(wants("あめがふる"));
        assert!(!wants("あ＼めが ふる"));
        assert!(!wants("Hello"));
    }
}
