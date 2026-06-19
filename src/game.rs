use std::io::{self, stdin};

use crate::error::RustleError;
use crate::feedback::LetterState;

/// The number of characters in each Wordle word.
pub const WORD_LEN: usize = 5;

/// The maximum number of guesses the player is allowed.
pub const MAX_GUESSES: usize = 6;

type EvaluatedGuess = ([char; WORD_LEN], [LetterState; WORD_LEN]);

/// The outcome of applying a single line of player input via [`Game::apply_turn`].
///
/// This captures every decision the game makes about one turn *without* touching
/// stdin/stdout, so the turn logic can be exercised in unit tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnResult {
    /// The input was not applied; the payload is the user-facing reason.
    Rejected(&'static str),
    /// A valid guess was evaluated and recorded; the game continues.
    Accepted,
    /// The just-applied guess ended the game (a win or the final loss).
    GameOver,
}

/// Holds the current word and game state for a single Wordle session.
pub struct Game {
    word: [char; WORD_LEN],
    state: GameState,
}

enum GameState {
    Won { guesses: Vec<EvaluatedGuess> },
    Lost { guesses: Vec<EvaluatedGuess> },
    InProgress { guesses: Vec<EvaluatedGuess> },
}

impl Game {
    /// Creates a new game from the solution word.
    ///
    /// Returns [`RustleError::InvalidWordLength`] if the word is not exactly
    /// five characters long, guaranteeing the solution can be treated as a
    /// `[char; WORD_LEN]` for the rest of the game.
    pub fn new(word: Vec<char>) -> Result<Self, RustleError> {
        let word: [char; WORD_LEN] = word
            .as_slice()
            .try_into()
            .map_err(|_| RustleError::InvalidWordLength(word.len()))?;
        Ok(Game {
            word,
            state: GameState::InProgress {
                guesses: Vec::new(),
            },
        })
    }

    fn change_state(&mut self, state: GameState) {
        self.state = state;
    }

    /// Returns the guessed words in play order as lowercase strings.
    ///
    /// Reads the accumulated guesses out of whichever internal state the game
    /// currently holds, so it is valid both mid-game and after the loop has
    /// reached a terminal state.
    pub fn guessed_words(&self) -> Vec<String> {
        let guesses = match &self.state {
            GameState::Won { guesses }
            | GameState::Lost { guesses }
            | GameState::InProgress { guesses } => guesses,
        };
        guesses
            .iter()
            .map(|(word, _)| word.iter().collect())
            .collect()
    }

    /// Returns `true` if the game has been won.
    pub fn is_won(&self) -> bool {
        matches!(self.state, GameState::Won { .. })
    }

    /// Returns the solution word as a lowercase string.
    pub fn solution_string(&self) -> String {
        self.word.iter().collect()
    }

    fn add_evaluated_guess(&mut self, arr: [char; WORD_LEN], states: [LetterState; WORD_LEN]) {
        if let GameState::InProgress { guesses } = &mut self.state {
            guesses.push((arr, states));
        }
    }

    /// Runs the interactive game loop, returning `true` on a win and `false`
    /// on a loss.
    ///
    /// Each iteration renders the board, reads a guess from stdin, validates
    /// it, and evaluates it against the solution. The loop terminates when the
    /// player either guesses the word correctly or exhausts all
    /// [`MAX_GUESSES`] attempts.
    pub fn start_game(&mut self) -> bool {
        println!("Please enter a 5 letter word: ");
        loop {
            match &self.state {
                GameState::Won { guesses } => {
                    // Show the completed board, including the winning row, so
                    // the player sees their final guess colored green.
                    crate::render::render_board(guesses, &mut io::stdout())
                        .expect("write to stdout");
                    println!("You are correcto");
                    crate::celebrate::confetti();
                    return true;
                }
                GameState::Lost { guesses } => {
                    // Show the full final board before revealing the answer.
                    crate::render::render_board(guesses, &mut io::stdout())
                        .expect("write to stdout");
                    let solution: String = self.word.iter().collect();
                    println!("almost, baka. The word is actually {solution}");
                    crate::celebrate::play_sad_sound();
                    return false;
                }
                GameState::InProgress { guesses } => {
                    // Render the full board before prompting.
                    crate::render::render_board(guesses, &mut io::stdout())
                        .expect("write to stdout");

                    let mut input_string = String::new();
                    stdin()
                        .read_line(&mut input_string)
                        .expect("Please enter a valid string");

                    // A rejection just prints the reason and re-prompts; an
                    // accepted guess or game-over falls through to the next loop
                    // iteration, which renders/handles the new state.
                    if let TurnResult::Rejected(msg) = self.apply_turn(&input_string) {
                        println!("{msg}");
                    }
                }
            }
        }
    }

    /// Applies one line of raw player input to the game and reports the outcome.
    ///
    /// This is the pure core of a turn: it performs no I/O. It parses the input,
    /// validates it against the allowed-guess list, evaluates an accepted guess
    /// against the solution, records it, and transitions the game state.
    ///
    /// The solution is always an accepted guess, even if it is missing from the
    /// bundled (and possibly stale) word lists, so the game can never be made
    /// unwinnable.
    ///
    /// Returns:
    /// - [`TurnResult::Rejected`] with a user-facing message if the input is not
    ///   a five-letter word or is not in the allowed-guess list. Nothing is
    ///   recorded.
    /// - [`TurnResult::GameOver`] if the recorded guess won the game or used up
    ///   the final attempt.
    /// - [`TurnResult::Accepted`] otherwise.
    ///
    /// Calling this once the game has already reached a terminal state is a
    /// no-op that returns [`TurnResult::GameOver`].
    pub fn apply_turn(&mut self, raw_input: &str) -> TurnResult {
        if !matches!(self.state, GameState::InProgress { .. }) {
            return TurnResult::GameOver;
        }

        let arr = match parse_guess(raw_input) {
            Ok(arr) => arr,
            Err(msg) => return TurnResult::Rejected(msg),
        };

        // A guess is accepted if it is in `answers ∪ allowed`, *or* if it
        // exactly matches the solution. The bundled lists are a frozen snapshot
        // and can lag the live NYT answer rotation, so the solution itself is
        // not guaranteed to appear in them. Without this guard a solution that
        // is absent from the lists could never be typed, making that day's game
        // unwinnable.
        if arr != self.word {
            let word_str: String = arr.iter().collect();
            if !crate::dictionary::is_allowed_guess(&word_str) {
                return TurnResult::Rejected("Not in word list");
            }
        }

        let states = crate::feedback::evaluate(&arr, &self.word);
        self.add_evaluated_guess(arr, states);

        // Decide the next state based on the just-added guess, moving the
        // accumulated guesses into the terminal state.
        if let GameState::InProgress { guesses } = &mut self.state {
            if arr == self.word {
                let guesses = std::mem::take(guesses);
                self.change_state(GameState::Won { guesses });
                return TurnResult::GameOver;
            } else if guesses.len() >= MAX_GUESSES {
                let guesses = std::mem::take(guesses);
                self.change_state(GameState::Lost { guesses });
                return TurnResult::GameOver;
            }
        }

        TurnResult::Accepted
    }
}

fn parse_guess(raw: &str) -> Result<[char; WORD_LEN], &'static str> {
    let lower: String = raw.trim().to_lowercase();
    let chars: Vec<char> = lower.chars().collect();
    if chars.len() != WORD_LEN {
        return Err("Please enter a 5-letter word");
    }
    if !chars.iter().all(|c| c.is_ascii_alphabetic()) {
        return Err("Guess must contain only letters (a-z)");
    }
    Ok(chars.try_into().expect("length checked above"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_guess_valid() {
        assert_eq!(parse_guess("CRANE"), Ok(['c', 'r', 'a', 'n', 'e']));
    }

    #[test]
    fn test_parse_guess_digit() {
        assert_eq!(
            parse_guess("cr4ne"),
            Err("Guess must contain only letters (a-z)")
        );
    }

    #[test]
    fn test_parse_guess_special_char() {
        assert_eq!(
            parse_guess("cr@ne"),
            Err("Guess must contain only letters (a-z)")
        );
    }

    #[test]
    fn test_parse_guess_too_long() {
        assert_eq!(parse_guess("toolong"), Err("Please enter a 5-letter word"));
    }

    #[test]
    fn test_parse_guess_too_short() {
        assert_eq!(parse_guess("abc"), Err("Please enter a 5-letter word"));
    }

    fn game_with_solution(word: &str) -> Game {
        Game::new(word.chars().collect()).expect("test solution must be 5 chars")
    }

    #[test]
    fn test_apply_turn_rejects_non_word() {
        let mut game = game_with_solution("crane");
        assert_eq!(
            game.apply_turn("zzzzz\n"),
            TurnResult::Rejected("Not in word list")
        );
        // A rejected guess is never recorded.
        assert!(game.guessed_words().is_empty());
    }

    #[test]
    fn test_apply_turn_rejects_malformed_input() {
        let mut game = game_with_solution("crane");
        assert_eq!(
            game.apply_turn("abc\n"),
            TurnResult::Rejected("Please enter a 5-letter word")
        );
        assert!(game.guessed_words().is_empty());
    }

    #[test]
    fn test_apply_turn_accepts_valid_word() {
        let mut game = game_with_solution("crane");
        assert_eq!(game.apply_turn("spell\n"), TurnResult::Accepted);
        // Exactly one evaluated guess is appended.
        assert_eq!(game.guessed_words(), vec!["spell".to_string()]);
        assert!(!game.is_won());
    }

    #[test]
    fn test_apply_turn_solution_wins() {
        let mut game = game_with_solution("crane");
        assert_eq!(game.apply_turn("crane\n"), TurnResult::GameOver);
        assert!(game.is_won());
        assert_eq!(game.guessed_words(), vec!["crane".to_string()]);
    }

    #[test]
    fn test_apply_turn_accepts_solution_absent_from_word_lists() {
        // "zzzzz" is not in answers.txt or allowed.txt, but if NYT ever serves a
        // solution missing from the bundled (frozen) lists, the player must
        // still be able to type and win with it. Otherwise the day is unwinnable.
        assert!(!crate::dictionary::is_allowed_guess("zzzzz"));
        let mut game = game_with_solution("zzzzz");
        assert_eq!(game.apply_turn("zzzzz\n"), TurnResult::GameOver);
        assert!(game.is_won());
        assert_eq!(game.guessed_words(), vec!["zzzzz".to_string()]);
    }

    #[test]
    fn test_apply_turn_six_wrong_guesses_loses() {
        let mut game = game_with_solution("crane");
        // Five valid, wrong guesses keep the game in progress.
        for guess in ["spell", "audio", "ghost", "lucky", "joker"] {
            assert_eq!(game.apply_turn(guess), TurnResult::Accepted);
        }
        assert!(!game.is_won());
        // The sixth wrong-but-valid guess exhausts the attempts and loses.
        assert_eq!(game.apply_turn("month"), TurnResult::GameOver);
        assert!(!game.is_won());
        assert_eq!(game.guessed_words().len(), MAX_GUESSES);
    }
}
