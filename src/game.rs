use std::io::{self, stdin, BufReader, Cursor};
use std::process::Command;

use rodio::{source::Source, Decoder, OutputStream};

use crate::error::RustleError;
use crate::feedback::LetterState;

pub const WORD_LEN: usize = 5;
pub const MAX_GUESSES: usize = 6;

type EvaluatedGuess = ([char; WORD_LEN], [LetterState; WORD_LEN]);

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

    fn add_evaluated_guess(&mut self, arr: [char; WORD_LEN], states: [LetterState; WORD_LEN]) {
        if let GameState::InProgress { guesses } = &mut self.state {
            guesses.push((arr, states));
        }
    }

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
                    Command::new("open")
                        .arg("raycast://confetti")
                        .stderr(std::process::Stdio::null())
                        .spawn()
                        .expect("You should have Raycast... But congratulations I guess. Download Raycast though.")
                        .wait()
                        .ok();
                    return true;
                }
                GameState::Lost { guesses } => {
                    // Show the full final board before revealing the answer.
                    crate::render::render_board(guesses, &mut io::stdout())
                        .expect("write to stdout");
                    let solution: String = self.word.iter().collect();
                    println!("almost, baka. The word is actually {solution}");
                    // Get a output stream handle to the default physical sound device
                    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

                    // Load the audio file at compile time as bytes,
                    // to be able to use regardless of current
                    // directory
                    let audio_file = include_bytes!("../assets/stupid.mp3");
                    let audio_cursor = Cursor::new(audio_file);
                    let audio_buffer = BufReader::new(audio_cursor);

                    // Decode that sound file into a source
                    let source = Decoder::new(audio_buffer).unwrap();

                    // Play the sound directly on the device
                    stream_handle
                        .play_raw(source.convert_samples())
                        .expect("Failed to play file");
                    std::thread::sleep(std::time::Duration::from_secs(1));
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

                    let arr = match parse_guess(&input_string) {
                        Ok(arr) => arr,
                        Err(msg) => {
                            println!("{msg}");
                            continue;
                        }
                    };

                    // The solution is always accepted, even if it is missing
                    // from the bundled word list (which may be stale relative
                    // to the live NYT answer). Otherwise the correct guess
                    // could be rejected and the game made unwinnable.
                    if !is_accepted_guess(&arr, &self.word) {
                        println!("Not in word list");
                        continue;
                    }

                    let states = crate::feedback::evaluate(&arr, &self.word);
                    self.add_evaluated_guess(arr, states);

                    // Decide the next state based on the just-added guess,
                    // moving the accumulated guesses into the terminal state.
                    if let GameState::InProgress { guesses } = &mut self.state {
                        if arr == self.word {
                            let guesses = std::mem::take(guesses);
                            self.change_state(GameState::Won { guesses });
                        } else if guesses.len() >= MAX_GUESSES {
                            let guesses = std::mem::take(guesses);
                            self.change_state(GameState::Lost { guesses });
                        }
                    }
                }
            }
        }
    }
}

/// Returns `true` if `guess` may be entered as a guess.
///
/// A guess is accepted if it is a recognised word *or* if it exactly matches
/// the solution. The latter guarantees the correct answer is never blocked by
/// a stale bundled word list, which would otherwise make the game unwinnable.
fn is_accepted_guess(guess: &[char; WORD_LEN], solution: &[char; WORD_LEN]) -> bool {
    if guess == solution {
        return true;
    }
    let word_str: String = guess.iter().collect();
    crate::dictionary::is_valid_word(&word_str)
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

    #[test]
    fn test_accepted_guess_dictionary_word() {
        let guess = ['c', 'r', 'a', 'n', 'e'];
        let solution = ['s', 'p', 'e', 'l', 'l'];
        assert!(is_accepted_guess(&guess, &solution));
    }

    #[test]
    fn test_accepted_guess_rejects_non_word() {
        let guess = ['z', 'z', 'z', 'z', 'z'];
        let solution = ['c', 'r', 'a', 'n', 'e'];
        assert!(!is_accepted_guess(&guess, &solution));
    }

    #[test]
    fn test_accepted_guess_solution_not_in_dictionary() {
        // The solution itself is accepted even when it is not a recognised
        // dictionary word, so a stale word list can never block a win.
        let solution = ['z', 'z', 'z', 'z', 'z'];
        assert!(!crate::dictionary::is_valid_word("zzzzz"));
        assert!(is_accepted_guess(&solution, &solution));
    }
}
