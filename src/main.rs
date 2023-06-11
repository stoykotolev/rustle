use std::io;

use chrono::Local;
use serde::{Deserialize, Serialize};

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

// let resp = reqwest::blocking::get("https://www.nytimes.com/svc/wordle/v2/2023-06-05.json")?
fn main() {
    let mut tries: i8 = 1;

    let word_of_the_day: Vec<char> = match get_data() {
        Ok(value) => value,
        Err(err) => {
            eprintln!("Error: {:?}", err);
            std::process::exit(1);
        }
    };

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
            break;
        }

        if input_string.trim().len() > 6 {
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

    if tries > 5 {
        println!("almost");
    }
}
