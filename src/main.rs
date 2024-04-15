mod utils;
use utils::{get_data, get_word, start_game};

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
