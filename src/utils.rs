use std::io::{self, Cursor};
use std::process::Command;

use chrono::Local;
use serde::{Deserialize, Serialize};

use rodio::{source::Source, Decoder, OutputStream};
use std::io::BufReader;

#[derive(Debug, Serialize, Deserialize)]
pub struct WordleData<'a> {
    pub solution: &'a str,
}

pub fn fail_route() {
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

pub fn start_game(word: Vec<char>) {
    let mut tries: i8 = 1;

    println!("Please enter a 5 letter word: ");

    while tries <= 5 {
        let mut input_string = String::new();
        let curr_try = word.clone();

        io::stdin().read_line(&mut input_string).unwrap();

        if input_string.trim() == word.iter().collect::<String>().trim() {
            println!("You are correcto");
            Command::new("open")
                .arg("raycast://confetti")
                .stderr(std::process::Stdio::null())
                .spawn()
                .expect(
                "You should have Raycast... But congratulations I guess. Download Raycast though.",
            );
            std::process::exit(1);
        }

        if input_string.trim().len() != 5 {
            println!("Please enter a 5 letter word");
            continue;
        }

        compare_words(input_string, curr_try);

        println!();
        tries += 1;
    }

    println!("almost");
    fail_route();
}

pub fn compare_words(input: String, mut wotd: Vec<char>) {
    let orgiginal_wotd = wotd.clone();

    for (i, c) in input.chars().enumerate() {
        if i >= 5 {
            break;
        }

        let target_char = wotd.get(i).unwrap();
        if c == *target_char {
            print!("\x1b[1;32m{}\x1b[0m", c);
            wotd[i] = '\0';
        } else if orgiginal_wotd.contains(&c)
            && orgiginal_wotd.contains(&c)
            && input.chars().filter(|&x| x == c).count()
                <= orgiginal_wotd.iter().filter(|&&x| x == c).count()
        {
            print!("\x1b[0;33m{}\x1b[0m", c);
            wotd[i] = '\0';
        } else {
            print!("\x1b[0;37m{}\x1b[0m", c);
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

    #[test]
    fn test_duplicate_letters() {}
}

/* struct Game {
  word: [char; 5];
  state: GameState
}

enum GameState {
  Won,
  Lost,
  InProgress {
    guesses: Vec<Guess>
  }
}

struct Guess([char; 5]);

impl Guess {
  pub fn new(value: &str) -> Result<Self, ()> {
    Self(value.chars().collect::<Vec<_>>().map_err(|_| ()))
  }
} */
