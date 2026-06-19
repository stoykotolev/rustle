use strustle::game::Game;
use strustle::nyt::{get_data, get_word};

fn main() {
    let bytes = match get_data() {
        Ok(value) => value,
        Err(err) => {
            eprintln!("Error: {err}");
            std::process::exit(1);
        }
    };
    let word_of_the_day = match get_word(&bytes) {
        Ok(value) => value.solution.chars().collect::<Vec<char>>(),
        Err(err) => {
            eprintln!("Error: {err}");
            std::process::exit(1);
        }
    };

    let mut game = match Game::new(word_of_the_day) {
        Ok(game) => game,
        Err(err) => {
            eprintln!("Error: {err}");
            std::process::exit(1);
        }
    };
    game.start_game();
}
