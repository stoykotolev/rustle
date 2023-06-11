use std::io;
use std::process::Command;

use chrono::Local;
use serde::{Deserialize, Serialize};

use rodio::{source::Source, Decoder, OutputStream};
use std::fs::File;
use std::io::BufReader;

#[derive(Debug, Serialize, Deserialize)]
struct WordleData {
    id: i32,
    solution: String,
    print_date: String,
    days_since_launch: i32,
    editor: String,
}

fn get_data() -> Result<Vec<char>, Box<dyn std::error::Error>> {
    let current_date = Local::now().date_naive();
    let wordle_url = format!(
        "https://www.nytimes.com/svc/wordle/v2/{}.json",
        current_date
    );

    let resp = reqwest::blocking::get(wordle_url)?.json::<WordleData>()?;

    Ok(resp.solution.chars().collect::<Vec<char>>())
}

fn main() {
    let mut tries: i8 = 1;

    let word_of_the_day: Vec<char> = match get_data() {
        Ok(value) => value,
        Err(err) => {
            eprintln!("Error: {:?}", err);
            std::process::exit(1);
        }
    };

    println!("Please enter a 5 letter word: ");
    while tries <= 5 {
        let mut input_string = String::new();
        let mut user_guess: [String; 5] = [
            String::from("_"),
            String::from("_"),
            String::from("_"),
            String::from("_"),
            String::from("_"),
        ];

        io::stdin().read_line(&mut input_string).unwrap();

        if input_string.trim() == word_of_the_day.iter().collect::<String>().trim() {
            println!("You are correcto");
            Command::new("open")
                .arg("raycast://confetti")
                .spawn()
                .expect("The raycast confetti command to run");
            std::process::exit(0);
        }

        if input_string.trim().len() >= 6 {
            println!("Please enter a 5 letter word");
            continue;
        }

        if input_string.trim().len() < 5 {
            println!("Please enter a 5 letter word");
            continue;
        }

        for (index, char) in input_string.chars().enumerate() {
            if word_of_the_day.contains(&char) {
                if word_of_the_day[index] == char {
                    user_guess[index] = format!("\x1b[1;32m{}\x1b[0m", char);
                } else {
                    user_guess[index] = format!("\x1b[0;33m{}\x1b[0m", char)
                }
            }
        }
        println!("Guess: {}", user_guess.join("").trim());

        tries += 1;
    }

    println!("almost");
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
    std::thread::sleep(std::time::Duration::from_secs(5));
}
