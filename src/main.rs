use std::io;
use std::process::Command;

use chrono::Local;
use serde::{Deserialize, Serialize};

use rodio::{source::Source, Decoder, OutputStream};
use std::fs::File;
use std::io::BufReader;

#[derive(Debug, Serialize, Deserialize)]
struct WordleData<'a> {
    solution: &'a str,
}

fn fail_route() {
    // Get a output stream handle to the default physical sound device
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    // Load a sound from a file, using a path relative to Cargo.toml
    let file = BufReader::new(File::open("./stupid.mp3").expect("The audio file to be present"));
    // Decode that sound file into a source
    let source = Decoder::new(file).unwrap();
    // Play the sound directly on the device
    stream_handle
        .play_raw(source.convert_samples())
        .expect("Failed to play file");
    // match {
    //     Err(e) => eprintln!("Err: {:?}", e),
    //     _ => (),
    // };
    std::thread::sleep(std::time::Duration::from_secs(1));
}

fn get_data() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let current_date = Local::now().date_naive();
    let wordle_url = format!(
        "https://www.nytimes.com/svc/wordle/v2/{}.json",
        current_date
    );

    let resp = reqwest::blocking::get(wordle_url)?.bytes()?;

    Ok(resp.to_vec())
}

fn get_word<'a>(bytes: &'a [u8]) -> Result<WordleData<'a>, Box<dyn std::error::Error>> {
    let data = std::str::from_utf8(&bytes)?;
    let word = serde_json::from_str::<WordleData>(&data)?;

    Ok(word)
}

fn start_game(word: Vec<char>) {
    let mut tries: i8 = 1;

    println!("Please enter a 5 letter word: ");

    while tries <= 5 {
        let mut input_string = String::new();

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

        for (index, char) in input_string.chars().enumerate() {
            if index >= 5 {
                break;
            }
            if word.contains(&char) {
                if word[index] == char {
                    print!("\x1b[1;32m{}\x1b[0m", char);
                } else {
                    print!("\x1b[0;33m{}\x1b[0m", char)
                }
                continue;
            }
            print!("\x1b[1;30m{}\x1b[0m", char)
        }
        println!();

        tries += 1;
    }

    println!("almost");
    fail_route();
}

fn main() {
    let bytes = match get_data() {
        Ok(value) => value,
        Err(err) => {
            eprintln!("Error: {:?}", err);
            std::process::exit(1);
        }
    };
    let word_of_the_day: Vec<char> = match get_word(&bytes) {
        Ok(value) => value.solution.chars().collect::<Vec<char>>(),
        Err(err) => {
            eprintln!("Error: {:?}", err);
            std::process::exit(1);
        }
    };

    start_game(word_of_the_day);
}
