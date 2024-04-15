use std::io::{stdin, Cursor};
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
    pub word: Word,
    state: GameState,
}

impl Game {
    pub fn new(word: Vec<char>) -> Self {
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
                    if guess.len() != 5 {
                        println!("Please enter a 5 letter word");
                        continue;
                    }

                    self.add_guess(guess);
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

pub fn compare_words(input: String, mut wotd: Vec<char>) {
    let orgiginal_wotd = wotd.clone();

    for (indx, char) in input.chars().enumerate() {
        if indx >= 5 {
            break;
        }

        let target_char = wotd.get(indx).unwrap();
        if char == *target_char {
            print!("\x1b[1;32m{}\x1b[0m", char);
            wotd[indx] = '\0';
        } else if orgiginal_wotd.contains(&char)
            && orgiginal_wotd.contains(&char)
            && input.chars().filter(|&x| x == char).count()
                <= orgiginal_wotd.iter().filter(|&&x| x == char).count()
        {
            print!("\x1b[0;33m{}\x1b[0m", char);
            wotd[indx] = '\0';
        } else {
            print!("\x1b[0;37m{}\x1b[0m", char);
        }
    }
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
}

/*  */
