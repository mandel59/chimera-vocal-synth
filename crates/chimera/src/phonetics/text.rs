//! Language selection before the existing speech and singing planners.
//! Dictionary I/O stays with the host; EnglishG2P remains English-only.

use super::english::EnglishG2P;
use super::syllables::{self, WordSource};
use super::{Phrase, Syllable, japanese};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    #[serde(rename = "en", alias = "en-US")]
    English,
    #[serde(rename = "ja", alias = "ja-JP")]
    Japanese,
}

impl Language {
    /// Compatibility for existing unlabelled inputs. Explicit selection always wins.
    pub fn detect(text: &str) -> Self {
        if japanese::has_kana(text) || needs_reading(text) { Self::Japanese } else { Self::English }
    }
}

impl std::str::FromStr for Language {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "en" | "en-US" => Ok(Self::English),
            "ja" | "ja-JP" => Ok(Self::Japanese),
            _ => Err(format!("unsupported language {value:?}; use en or ja")),
        }
    }
}

/// Kanji need a host-side dictionary reading before the kana frontend can use them.
pub fn needs_reading(text: &str) -> bool {
    text.chars().any(|c| matches!(c, '\u{3400}'..='\u{9FFF}' | '々'))
}

pub fn to_phrases(text: &str, language: Option<Language>, g: &EnglishG2P) -> Vec<Phrase> {
    match language.unwrap_or_else(|| Language::detect(text)) {
        Language::English => g.to_phrases(text),
        Language::Japanese => japanese::to_phrases(text),
    }
}

pub fn lyrics_to_syllables(text: &str, language: Option<Language>, g: &impl WordSource) -> Vec<Syllable> {
    let kana = language.unwrap_or_else(|| Language::detect(text)) == Language::Japanese;
    syllables::lyrics_with_kana(text, g, kana)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_entry_keeps_the_upstream_result() {
        let g = EnglishG2P::new();
        assert_eq!(to_phrases("Can you hear me?", Some(Language::English), &g), g.to_phrases("Can you hear me?"));
        assert_eq!(lyrics_to_syllables("hello - world", Some(Language::English), &g), syllables::lyrics_to_syllables("hello - world", &g));
        assert!(g.to_phrases("さくら").is_empty(), "EnglishG2P must not dispatch Japanese");
    }

    #[test]
    fn explicit_language_overrides_compatibility_detection() {
        let g = EnglishG2P::new();
        assert_eq!(to_phrases("さくら", None, &g), japanese::to_phrases("さくら"));
        assert!(to_phrases("さくら", Some(Language::English), &g).is_empty());
        assert!(lyrics_to_syllables("さくら", Some(Language::English), &g).is_empty());
        assert_eq!(lyrics_to_syllables("さくら", Some(Language::Japanese), &g).len(), 3);
    }

    #[test]
    fn japanese_punctuation_does_not_change_the_english_lyric_entry() {
        let g = EnglishG2P::new();
        let english = lyrics_to_syllables("hello。", Some(Language::English), &g);
        assert!(!english.last().unwrap().phrase_end);
        let japanese = lyrics_to_syllables("さくら。", Some(Language::Japanese), &g);
        assert!(japanese.last().unwrap().phrase_end);
    }

    #[test]
    fn language_codes_are_explicit_and_unknown_languages_are_errors() {
        assert_eq!("ja-JP".parse::<Language>().unwrap(), Language::Japanese);
        assert_eq!(serde_json::from_str::<Language>("\"en-US\"").unwrap(), Language::English);
        assert!("fr".parse::<Language>().is_err());
        assert!(serde_json::from_str::<Language>("\"fr\"").is_err());
        assert_eq!(Language::detect("東京"), Language::Japanese);
        assert!(needs_reading("東京"));
        assert!(!needs_reading("とうきょう"));
    }
}
