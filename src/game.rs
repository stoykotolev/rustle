use std::io::{self, stdin, BufReader, Cursor};
use std::process::Command;

use rodio::{source::Source, Decoder, OutputStream};

use crate::error::RustleError;

type Word = Vec<char>;

pub struct Game {
    word: [char; 5],
    state: GameState,
}

enum GameState {
    Won,
    Lost,
    InProgress { guesses: Vec<Word> },
}

impl Game {
    /// Creates a new game from the solution word.
    ///
    /// Returns [`RustleError::InvalidWordLength`] if the word is not exactly
    /// five characters long, guaranteeing the solution can be treated as a
    /// `[char; 5]` for the rest of the game.
    pub fn new(word: Word) -> Result<Self, RustleError> {
        let word: [char; 5] = word
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

                    // Length is validated above, so this conversion cannot panic.
                    let arr: [char; 5] = guess.as_slice().try_into().expect("guess length is 5");

                    // Handle won state
                    if arr == self.word {
                        self.change_state(GameState::Won);
                        continue;
                    }

                    let states = crate::feedback::evaluate(&arr, &self.word);
                    crate::render::render_guess(&arr, &states, &mut io::stdout())
                        .expect("write to stdout");
                    self.add_guess(guess);
                }
            }
        }
    }
}
