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
    let mut tries: i8 = 0;

    let word_of_the_day: Vec<char> = match get_data() {
        Ok(value) => value,
        Err(err) => {
            eprintln!("Error: {:?}", err);
            std::process::exit(1);
        }
    };

    println!(
        "This is the word for the day: {:?}",
        word_of_the_day.iter().collect::<String>()
    );
}
