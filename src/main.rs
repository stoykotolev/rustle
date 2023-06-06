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

// let resp = reqwest::blocking::get("https://www.nytimes.com/svc/wordle/v2/2023-06-05.json")?
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let current_date = Local::now().date_naive();
    let wordle_url = format!(
        "https://www.nytimes.com/svc/wordle/v2/{}.json",
        current_date
    );

    let resp = reqwest::blocking::get(wordle_url)?.json::<WordleData>()?;

    let word_of_the_day: Vec<char> = resp.solution.chars().collect();

    Ok(())
}
