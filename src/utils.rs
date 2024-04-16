use std::collections::HashMap;
use std::io::{self, stdin, Cursor};
use std::process::Command;

use chrono::Local;
use serde::{Deserialize, Serialize};

use rodio::{source::Source, Decoder, OutputStream};
use std::io::BufReader;

#[derive(Debug, Serialize, Deserialize)]
pub struct WordleData<'a> {
    pub solution: &'a str,
}

type Word = Vec<char>;

pub struct Game {
    word: Word,
    state: GameState,
}

impl Game {
    pub fn new(word: Word) -> Self {
        Game {
            word,
            state: GameState::InProgress {
                guesses: Vec::new(),
            },
        }
    }

    fn change_state(&mut self, state: GameState) {
        self.state = state;
    }

    fn add_guess(&mut self, guess: Word) {
        if let GameState::InProgress { guesses } = &mut self.state {
            guesses.push(guess)
        }
    }

    fn compare_words(&self, input: Word, stdout: &mut dyn io::Write) -> Word {
        let mut new_guess: Word = vec!['\0'; 5];
        let mut matched_counts: HashMap<char, usize> = HashMap::new();

        for (indx, char) in input.iter().enumerate() {
            if indx >= 5 {
                break;
            }

            let wotd_char = &self.word[indx];

            if char == wotd_char {
                let _ = write!(stdout, "\x1b[1;32m{}\x1b[0m", char);
                new_guess[indx] = *char;
            } else if self.word.contains(char) {
                let count_in_word = self.word.iter().filter(|&x| x == char).count();
                let count_in_input = input.iter().filter(|&x| x == char).count();
                if count_in_input <= count_in_word - matched_counts.get(char).copied().unwrap_or(0)
                {
                    let _ = write!(stdout, "\x1b[0;33m{}\x1b[0m", char);
                    new_guess[indx] = *char;
                    *matched_counts.entry(*char).or_insert(0) += 1;
                } else {
                    let _ = write!(stdout, "\x1b[0;37m{}\x1b[0m", char);
                    new_guess[indx] = *char;
                }
            } else {
                let _ = write!(stdout, "\x1b[0;37m{}\x1b[0m", char);
            }
        }
        let _ = writeln!(stdout);
        new_guess
    }

    pub fn start_game(&mut self) {
        println!("Please enter a 5 letter word: ");
        loop {
            match &mut self.state {
                GameState::Won => {
                    println!("You are correcto");
                    Command::new("open")
                        .arg("raycast://confetti")
                        .stderr(std::process::Stdio::null())
                        .spawn()
                        .expect("You should have Raycast... But congratulations I guess. Download Raycast though.");
                    std::process::exit(1);
                }
                GameState::Lost => {
                    println!("almost, baka");
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
                    break;
                }
                GameState::InProgress { guesses } => {
                    // Handle the loss state
                    if guesses.len() >= 6 {
                        self.change_state(GameState::Lost);
                        continue;
                    }

                    let mut input_string = String::new();
                    stdin()
                        .read_line(&mut input_string)
                        .expect("Please enter a valid string");
                    let guess = input_string.trim().chars().collect::<Word>();

                    // Handle won state
                    if guess == self.word {
                        self.change_state(GameState::Won);
                        continue;
                    }

                    if guess.len() != 5 {
                        println!("Please enter a 5 letter word");
                        continue;
                    }
                    let new_guess = self.compare_words(guess, &mut io::stdout());
                    self.add_guess(new_guess);
                }
            }
        }
    }
}

enum GameState {
    Won,
    Lost,
    InProgress { guesses: Vec<Word> },
}

pub fn get_data() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let current_date = Local::now().date_naive();
    let wordle_url = format!(
        "https://www.nytimes.com/svc/wordle/v2/{}.json",
        current_date
    );

    let resp = reqwest::blocking::get(wordle_url)?.bytes()?;

    Ok(resp.to_vec())
}

pub fn get_word(bytes: &[u8]) -> Result<WordleData<'_>, Box<dyn std::error::Error>> {
    let data = std::str::from_utf8(bytes)?;
    let word = serde_json::from_str::<WordleData>(data)?;

    Ok(word)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_word() {
        let sample_data = "{\"solution\":\"test\"}";
        let result = get_word(sample_data.as_bytes()).unwrap();
        assert_eq!(result.solution, "test");
    }

    #[test]
    fn test_compare_words() {
        let word = vec!['s', 'p', 'e', 'l', 'l'];
        let same_input = vec!['s', 'p', 'e', 'l', 'l']; // All characters match
        let mut output = Vec::new();

        let game = Game::new(word);
        game.compare_words(same_input, &mut output);
        let parsed_success_input =
            String::from_utf8(output.clone()).expect("Couldn't parse string");
        output.clear();

        assert_eq!("\x1b[1;32ms\x1b[0m\x1b[1;32mp\x1b[0m\x1b[1;32me\x1b[0m\x1b[1;32ml\x1b[0m\x1b[1;32ml\x1b[0m", parsed_success_input.trim());

        let entirely_different_input = vec!['d', 'a', 'w', 'u', 'i'];

        game.compare_words(entirely_different_input, &mut output);
        let parsed_failed_input = String::from_utf8(output.clone()).expect("Couldn't parse string");
        assert_eq!("\u{1b}[0;37md\u{1b}[0m\u{1b}[0;37ma\u{1b}[0m\u{1b}[0;37mw\u{1b}[0m\u{1b}[0;37mu\u{1b}[0m\u{1b}[0;37mi\u{1b}[0m", parsed_failed_input.trim());
        output.clear();

        let slightly_different_input = vec!['d', 'e', 'w', 'l', 'i'];
        game.compare_words(slightly_different_input, &mut output);
        let parsed_slightly_different_input =
            String::from_utf8(output.clone()).expect("Couldn't parse string");
        assert_eq!("\u{1b}[0;37md\u{1b}[0m\u{1b}[0;33me\u{1b}[0m\u{1b}[0;37mw\u{1b}[0m\u{1b}[1;32ml\u{1b}[0m\u{1b}[0;37mi\u{1b}[0m", parsed_slightly_different_input.trim());
        output.clear();

        let slightly_different_input_with_duplicate_letters = vec!['d', 'l', 'l', 'i', 'a'];
        game.compare_words(slightly_different_input_with_duplicate_letters, &mut output);

        let parsed_slightly_different_input_with_duplicate_letters =
            String::from_utf8(output.clone()).expect("Couldn't parse string");

        assert_eq!("\x1b[0;37md\x1b[0m\x1b[0;33ml\x1b[0m\x1b[0;33ml\x1b[0m\x1b[0;37mi\x1b[0m\x1b[0;37ma\x1b[0m", parsed_slightly_different_input_with_duplicate_letters.trim());
        output.clear();
    }
}

/*  */
