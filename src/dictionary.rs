use std::collections::HashSet;
use std::sync::OnceLock;

const WORDS: &str = include_str!("../assets/words.txt");

fn word_set() -> &'static HashSet<&'static str> {
    static SET: OnceLock<HashSet<&'static str>> = OnceLock::new();
    SET.get_or_init(|| WORDS.lines().collect())
}

/// Returns `true` if `word` is a recognised five-letter word.
///
/// The lookup is case-insensitive. The game already lowercases guesses, so
/// the defensive lowercase here is a no-op in normal play.
pub fn is_valid_word(word: &str) -> bool {
    let lower = word.to_lowercase();
    word_set().contains(lower.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crane_valid() {
        assert!(is_valid_word("crane"));
    }

    #[test]
    fn test_spell_valid() {
        assert!(is_valid_word("spell"));
    }

    #[test]
    fn test_zzzzz_invalid() {
        assert!(!is_valid_word("zzzzz"));
    }

    #[test]
    fn test_abcde_invalid() {
        assert!(!is_valid_word("abcde"));
    }

    #[test]
    fn test_crane_uppercase_valid() {
        assert!(is_valid_word("CRANE"));
    }

    #[test]
    fn test_crane_mixed_case_valid() {
        assert!(is_valid_word("cRaNe"));
    }

    #[test]
    fn test_cat_too_short_invalid() {
        assert!(!is_valid_word("cat"));
    }

    #[test]
    fn test_toolong_invalid() {
        assert!(!is_valid_word("toolong"));
    }
}
