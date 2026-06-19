use std::collections::HashSet;
use std::sync::OnceLock;

/// The canonical Wordle *answers* list (historically `La`): every word that can
/// be a solution.
const ANSWERS: &str = include_str!("../assets/answers.txt");

/// The *allowed guesses* list (historically `Ta`), unioned with the answers and
/// the project's legacy combined word list. See `assets/README.md`.
const ALLOWED: &str = include_str!("../assets/allowed.txt");

/// The set of words that may be the solution.
fn answer_set() -> &'static HashSet<&'static str> {
    static SET: OnceLock<HashSet<&'static str>> = OnceLock::new();
    SET.get_or_init(|| ANSWERS.lines().collect())
}

/// The set of words accepted as a guess: `ALLOWED ∪ ANSWERS`.
///
/// The answers are folded in unconditionally so that any solution is always a
/// legal guess, regardless of how the on-disk files are partitioned.
fn allowed_set() -> &'static HashSet<&'static str> {
    static SET: OnceLock<HashSet<&'static str>> = OnceLock::new();
    SET.get_or_init(|| ALLOWED.lines().chain(ANSWERS.lines()).collect())
}

/// Returns `true` if `word` may be entered as a guess.
///
/// A guess is accepted iff it is in `answers ∪ allowed`. The lookup is
/// case-insensitive; the game already lowercases guesses, so the defensive
/// lowercase here is a no-op in normal play.
pub fn is_allowed_guess(word: &str) -> bool {
    let lower = word.to_lowercase();
    allowed_set().contains(lower.as_str())
}

/// Returns `true` if `word` is a possible solution (a member of the answers
/// list).
///
/// The lookup is case-insensitive. Because the answers are a subset of the
/// allowed guesses, every answer also satisfies [`is_allowed_guess`].
pub fn is_answer(word: &str) -> bool {
    let lower = word.to_lowercase();
    answer_set().contains(lower.as_str())
}

/// Returns `true` if `word` is a recognised five-letter word.
///
/// Thin alias for [`is_allowed_guess`], kept for backwards compatibility with
/// existing callers that asked "is this a real word I can guess?".
pub fn is_valid_word(word: &str) -> bool {
    is_allowed_guess(word)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_allowed_guess_accepts_known_answer() {
        // "crane" is in the answers list, hence a valid guess.
        assert!(is_allowed_guess("crane"));
    }

    #[test]
    fn is_allowed_guess_accepts_guess_only_word() {
        // "aahed" is in the allowed-guesses list but is never an answer.
        assert!(is_allowed_guess("aahed"));
    }

    #[test]
    fn is_allowed_guess_rejects_non_word() {
        assert!(!is_allowed_guess("zzzzz"));
    }

    #[test]
    fn is_answer_true_for_answer() {
        assert!(is_answer("crane"));
    }

    #[test]
    fn is_answer_false_for_guess_only_word() {
        // "aahed" is an allowed guess but not a possible solution.
        assert!(!is_answer("aahed"));
        assert!(is_allowed_guess("aahed"));
    }

    #[test]
    fn case_insensitive() {
        assert!(is_allowed_guess("CRANE"));
        assert!(is_allowed_guess("cRaNe"));
        assert!(is_answer("CRANE"));
    }

    #[test]
    fn is_valid_word_delegates_to_is_allowed_guess() {
        assert!(is_valid_word("crane"));
        assert!(is_valid_word("aahed"));
        assert!(!is_valid_word("zzzzz"));
    }

    /// Every line of `answers.txt` is exactly five lowercase ASCII letters.
    #[test]
    fn answers_asset_integrity() {
        for line in ANSWERS.lines() {
            assert_eq!(line.len(), 5, "answer {line:?} is not five chars");
            assert!(
                line.chars().all(|c| c.is_ascii_alphabetic()),
                "answer {line:?} has a non-alphabetic char"
            );
            assert!(
                line.chars().all(|c| c.is_ascii_lowercase()),
                "answer {line:?} is not lowercase"
            );
        }
    }

    /// Every line of `allowed.txt` is exactly five lowercase ASCII letters.
    #[test]
    fn allowed_asset_integrity() {
        for line in ALLOWED.lines() {
            assert_eq!(line.len(), 5, "allowed word {line:?} is not five chars");
            assert!(
                line.chars().all(|c| c.is_ascii_alphabetic()),
                "allowed word {line:?} has a non-alphabetic char"
            );
            assert!(
                line.chars().all(|c| c.is_ascii_lowercase()),
                "allowed word {line:?} is not lowercase"
            );
        }
    }

    /// Lower-bound count checks so an accidentally truncated asset fails the
    /// build rather than silently shrinking the word set.
    #[test]
    fn count_sanity() {
        let answers = answer_set().len();
        let allowed = allowed_set().len();
        assert!(
            (2_000..=2_600).contains(&answers),
            "answer count {answers} outside expected range"
        );
        assert!(
            allowed >= 7_000,
            "allowed count {allowed} unexpectedly small"
        );
        // The allowed set is a strict superset of the answers.
        assert!(allowed > answers);
    }

    /// Every word in the bundled answers list is also an accepted guess
    /// (`answers ⊆ allowed_set`). Note this only covers the *bundled* lists; the
    /// live NYT solution can still fall outside them, which is why
    /// [`crate::game::Game::apply_turn`] keeps a solution-always-accepted guard.
    #[test]
    fn superset_invariant() {
        for answer in ANSWERS.lines() {
            assert!(
                is_allowed_guess(answer),
                "answer {answer:?} is not an allowed guess"
            );
        }
    }

    #[test]
    fn known_answers_present() {
        for word in ["crane", "aback", "zonal", "spell", "audio", "ghost"] {
            assert!(is_answer(word), "{word:?} should be a known answer");
        }
    }
}
